use std::{collections::HashMap, rc::Rc};

use ultraviolet::{Mat4, Vec2, Vec3};
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

use super::{GlCtx, GlCtxHandle, GlError, Result};

#[macro_export]
macro_rules! program {
    (
        $ctx:expr,
        $vertex:expr,
        $fragment:expr,
        {$($attribute:ident),*$(,)*},
        {$($uniform:ident),*$(,)*}$(,)*
    ) => {{
        $ctx.mk_program(
            include_str!($vertex),
            include_str!($fragment),
            vec![$(stringify!($attribute)),*],
            vec![$(stringify!($uniform)),*],
        )
    }}
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ShaderKind {
    Vertex = WebGl2RenderingContext::VERTEX_SHADER as isize,
    Fragment = WebGl2RenderingContext::FRAGMENT_SHADER as isize,
}

struct UntypedShader {
    ctx: GlCtxHandle,
    webgl_shader: WebGlShader,
    kind: ShaderKind,
}

#[derive(Clone)]
struct VertexShader(Rc<UntypedShader>);
#[derive(Clone)]
struct FragmentShader(Rc<UntypedShader>);

struct ProgramData {
    ctx: GlCtxHandle,

    vertex_shader: VertexShader,
    fragment_shader: FragmentShader,
    attributes: HashMap<&'static str, u32>,
    uniforms: HashMap<&'static str, WebGlUniformLocation>,

    webgl_program: WebGlProgram,
}

#[derive(Clone)]
pub struct Program(Rc<ProgramData>);

impl<'ctx> UntypedShader {
    fn compile<S: AsRef<str>>(ctx: &'ctx GlCtx, kind: ShaderKind, src: S) -> Result<Self> {
        let allocated = ctx
            .with_gl(|gl| gl.create_shader(kind as u32))
            .ok_or(GlError::Allocate)?;

        let shader = Self {
            ctx: ctx.ctx_handle(),
            webgl_shader: allocated,
            kind,
        };

        let compilation_success = ctx.with_gl(|gl| {
            // Set shader source
            gl.shader_source(&shader.webgl_shader, src.as_ref());
            // Attempt to compile the shader
            gl.compile_shader(&shader.webgl_shader);
            gl.get_shader_parameter(&shader.webgl_shader, WebGl2RenderingContext::COMPILE_STATUS)
        });

        // Check compilation was successful
        if compilation_success.as_bool().unwrap_or_default() {
            // If so, store shader data and move on
            Ok(shader)
        } else {
            // Otherwise try to get an error log
            let log = ctx.with_gl(|gl| gl.get_shader_info_log(&shader.webgl_shader));
            Err(GlError::ShaderCompile(log.unwrap_or_else(|| {
                "unknown shader compilation failure".to_owned()
            })))
        }
    }

    #[inline]
    fn into_vertex_shader(self) -> VertexShader {
        assert_eq!(self.kind, ShaderKind::Vertex);
        VertexShader(Rc::new(self))
    }

    #[inline]
    fn into_fragment_shader(self) -> FragmentShader {
        assert_eq!(self.kind, ShaderKind::Fragment);
        FragmentShader(Rc::new(self))
    }
}

impl<'ctx> Drop for UntypedShader {
    #[inline]
    fn drop(&mut self) {
        self.ctx
            .with_gl(|gl| gl.delete_shader(Some(&self.webgl_shader)));
    }
}

impl VertexShader {
    #[inline]
    fn compile<S>(ctx: &GlCtx, src: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        Ok(UntypedShader::compile(ctx, ShaderKind::Vertex, src)?.into_vertex_shader())
    }
}

impl FragmentShader {
    #[inline]
    fn compile<S>(ctx: &GlCtx, src: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        Ok(UntypedShader::compile(ctx, ShaderKind::Fragment, src)?.into_fragment_shader())
    }
}

impl Program {
    fn link(
        ctx: &GlCtx,
        vertex_shader: &VertexShader,
        fragment_shader: &FragmentShader,
        attributes: Vec<&'static str>,
        uniforms: Vec<&'static str>,
    ) -> Result<Self> {
        let attributes = {
            let mut map = HashMap::new();

            for (i, attribute) in attributes.into_iter().enumerate() {
                if map.insert(attribute, i as u32).is_some() {
                    return Err(GlError::ProgramLink(format!(
                        "duplicate attribute '{}'",
                        attribute
                    )));
                }
            }

            map
        };

        let allocated = ctx
            .with_gl(WebGl2RenderingContext::create_program)
            .ok_or(GlError::Allocate)?;

        let mut program = ProgramData {
            ctx: ctx.ctx_handle(),
            vertex_shader: vertex_shader.clone(),
            fragment_shader: fragment_shader.clone(),
            attributes,
            uniforms: HashMap::new(),
            webgl_program: allocated,
        };

        let linkage_success = ctx.with_gl(|gl| {
            // Attach shaders and link
            gl.attach_shader(
                &program.webgl_program,
                &program.fragment_shader.0.webgl_shader,
            );
            gl.attach_shader(
                &program.webgl_program,
                &program.vertex_shader.0.webgl_shader,
            );

            // Bind the attribute locations
            for (attribute, i) in &program.attributes {
                gl.bind_attrib_location(&program.webgl_program, *i, attribute);
            }

            gl.link_program(&program.webgl_program);

            gl.get_program_parameter(&program.webgl_program, WebGl2RenderingContext::LINK_STATUS)
        });

        // Check linking was successful
        if linkage_success.as_bool().unwrap_or_default() {
            // If so, get uniform locations, store program data, and move on
            for uniform in uniforms {
                let loc = ctx
                    .with_gl(|gl| gl.get_uniform_location(&program.webgl_program, uniform))
                    .ok_or_else(|| {
                        GlError::Uniform(format!(
                            "couldn't get the location of the uniform '{}'",
                            uniform,
                        ))
                    })?;

                if program.uniforms.insert(uniform, loc).is_some() {
                    return Err(GlError::ProgramLink(format!(
                        "duplicate uniform '{}'",
                        uniform,
                    )));
                }
            }

            Ok(Self(Rc::new(program)))
        } else {
            // Otherwise, try to get an error log
            let log = ctx.with_gl(|gl| gl.get_program_info_log(&program.webgl_program));
            Err(GlError::ProgramLink(log.unwrap_or_else(|| {
                "unknown program link failure".to_owned()
            })))
        }
    }

    #[inline]
    pub(super) fn uniforms(
        &self,
    ) -> impl Iterator<Item = (&'static str, &WebGlUniformLocation)> + '_ {
        self.0.uniforms.iter().map(|(k, v)| (*k, v))
    }

    #[inline]
    pub(super) fn has_uniform(&self, name: &'static str) -> bool {
        self.0.uniforms.contains_key(name)
    }

    #[inline]
    pub(super) fn bind<F, U>(&self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        self.0.ctx.with_gl(|gl| {
            gl.use_program(Some(&self.0.webgl_program));
            let result = f();
            gl.use_program(None);
            result
        })
    }
}

impl Drop for ProgramData {
    #[inline]
    fn drop(&mut self) {
        self.ctx
            .with_gl(|gl| gl.delete_program(Some(&self.webgl_program)));
    }
}

impl GlCtx {
    #[inline]
    pub fn mk_program<S, T>(
        &self,
        vertex_src: S,
        fragment_src: T,
        attributes: Vec<&'static str>,
        uniforms: Vec<&'static str>,
    ) -> Result<Program>
    where
        S: AsRef<str>,
        T: AsRef<str>,
    {
        Program::link(
            self,
            &VertexShader::compile(self, vertex_src)?,
            &FragmentShader::compile(self, fragment_src)?,
            attributes,
            uniforms,
        )
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe trait Uniformable: 'static {
    fn uniform(&self, ctx: &GlCtx, loc: &WebGlUniformLocation);
}

unsafe impl Uniformable for bool {
    #[inline]
    fn uniform(&self, ctx: &GlCtx, loc: &WebGlUniformLocation) {
        ctx.with_gl(|gl| gl.uniform1i(Some(loc), *self as i32));
    }
}

unsafe impl Uniformable for i32 {
    #[inline]
    fn uniform(&self, ctx: &GlCtx, loc: &WebGlUniformLocation) {
        ctx.with_gl(|gl| gl.uniform1i(Some(loc), *self));
    }
}

unsafe impl Uniformable for f32 {
    #[inline]
    fn uniform(&self, ctx: &GlCtx, loc: &WebGlUniformLocation) {
        ctx.with_gl(|gl| gl.uniform1f(Some(loc), *self));
    }
}

unsafe impl Uniformable for Vec2 {
    #[inline]
    fn uniform(&self, ctx: &GlCtx, loc: &WebGlUniformLocation) {
        ctx.with_gl(|gl| gl.uniform2f(Some(loc), self.x, self.y));
    }
}

unsafe impl Uniformable for Vec3 {
    #[inline]
    fn uniform(&self, ctx: &GlCtx, loc: &WebGlUniformLocation) {
        ctx.with_gl(|gl| gl.uniform3f(Some(loc), self.x, self.y, self.z));
    }
}

unsafe impl Uniformable for Mat4 {
    #[inline]
    fn uniform(&self, ctx: &GlCtx, loc: &WebGlUniformLocation) {
        ctx.with_gl(|gl| gl.uniform_matrix4fv_with_f32_array(Some(loc), false, self.as_array()));
    }
}
