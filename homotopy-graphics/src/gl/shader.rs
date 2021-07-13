use std::rc::Rc;

use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

use super::{GlCtx, GlError, Result};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ShaderKind {
    Vertex = WebGl2RenderingContext::VERTEX_SHADER as isize,
    Fragment = WebGl2RenderingContext::FRAGMENT_SHADER as isize,
}

struct UntypedShader {
    ctx: WebGl2RenderingContext,
    webgl_shader: WebGlShader,
    kind: ShaderKind,
}

#[derive(Clone)]
pub struct VertexShader(Rc<UntypedShader>);
#[derive(Clone)]
pub struct FragmentShader(Rc<UntypedShader>);

struct ProgramData {
    ctx: WebGl2RenderingContext,
    webgl_program: WebGlProgram,
    vertex_shader: VertexShader,
    fragment_shader: FragmentShader,
}

pub struct Program(Rc<ProgramData>);

impl UntypedShader {
    fn compile<S: AsRef<str>>(ctx: &GlCtx, kind: ShaderKind, src: S) -> Result<Self> {
        let allocated = ctx
            .webgl_ctx
            .create_shader(kind as u32)
            .ok_or(GlError::Allocate)?;

        let shader = Self {
            ctx: ctx.webgl_ctx.clone(),
            webgl_shader: allocated,
            kind,
        };

        // Set shader source
        shader.ctx.shader_source(&shader.webgl_shader, src.as_ref());
        // Attempt to compile the shader
        shader.ctx.compile_shader(&shader.webgl_shader);

        // Check compilation was successful
        if shader
            .ctx
            .get_shader_parameter(&shader.webgl_shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or_default()
        {
            // If so, store shader data and move on
            Ok(shader)
        } else {
            // And then try to get an error log
            Err(GlError::ShaderCompile(
                shader
                    .ctx
                    .get_shader_info_log(&shader.webgl_shader)
                    .unwrap_or_else(|| "unknown shader compilation failure".to_owned()),
            ))
        }
    }

    #[inline(always)]
    fn into_vertex_shader(self) -> VertexShader {
        assert_eq!(self.kind, ShaderKind::Vertex);
        VertexShader(Rc::new(self))
    }

    #[inline(always)]
    fn into_fragment_shader(self) -> FragmentShader {
        assert_eq!(self.kind, ShaderKind::Fragment);
        FragmentShader(Rc::new(self))
    }
}

impl Drop for UntypedShader {
    fn drop(&mut self) {
        self.ctx.delete_shader(Some(&self.webgl_shader));
    }
}

impl VertexShader {
    pub fn compile<S>(ctx: &GlCtx, src: S) -> Result<VertexShader>
    where
        S: AsRef<str>,
    {
        Ok(UntypedShader::compile(ctx, ShaderKind::Vertex, src)?.into_vertex_shader())
    }
}

impl FragmentShader {
    pub fn compile<S>(ctx: &GlCtx, src: S) -> Result<FragmentShader>
    where
        S: AsRef<str>,
    {
        Ok(UntypedShader::compile(ctx, ShaderKind::Fragment, src)?.into_fragment_shader())
    }
}

impl Program {
    pub fn link(
        ctx: &GlCtx,
        vertex_shader: &VertexShader,
        fragment_shader: &FragmentShader,
    ) -> Result<Program> {
        let allocated = ctx.webgl_ctx.create_program().ok_or(GlError::Allocate)?;
        let program = ProgramData {
            ctx: ctx.webgl_ctx.clone(),
            webgl_program: allocated,
            vertex_shader: vertex_shader.clone(),
            fragment_shader: fragment_shader.clone(),
        };

        // Attach shaders and link
        program.ctx.attach_shader(
            &program.webgl_program,
            &program.fragment_shader.0.webgl_shader,
        );
        program.ctx.attach_shader(
            &program.webgl_program,
            &program.vertex_shader.0.webgl_shader,
        );
        program.ctx.link_program(&program.webgl_program);

        // Check linking was successful
        if program
            .ctx
            .get_program_parameter(&program.webgl_program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or_default()
        {
            // If so, store program data and move on
            Ok(Program(Rc::new(program)))
        } else {
            // Otherwise, try to get an error log
            Err(GlError::ProgramLink(
                program
                    .ctx
                    .get_program_info_log(&program.webgl_program)
                    .unwrap_or_else(|| "unknown program link failure".to_owned()),
            ))
        }
    }

    #[inline(always)]
    pub(super) fn bind<F, U>(&self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        self.0.ctx.use_program(Some(&self.0.webgl_program));
        let result = f();
        self.0.ctx.use_program(None);
        result
    }
}

impl Drop for ProgramData {
    fn drop(&mut self) {
        self.ctx.delete_program(Some(&self.webgl_program));
    }
}
