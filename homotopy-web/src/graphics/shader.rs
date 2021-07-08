use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

use homotopy_core::declare_idx;

use super::{Bindable, GraphicsCtx, GraphicsError, GraphicsObject, Result};

pub mod attrib;
pub mod uniform;

declare_idx! {
    pub struct VertexShader = usize;

    pub struct FragmentShader = usize;

    pub struct Program = usize;
}

pub(super) struct ShaderData {
    webgl_shader: WebGlShader,
}

pub(super) struct ProgramData {
    webgl_program: WebGlProgram,
}

impl GraphicsObject for VertexShader {
    type Carrier = WebGlShader;
    type Data = ShaderData;

    #[inline]
    fn alloc_carrier(ctx: &mut GraphicsCtx) -> Result<Self::Carrier> {
        ctx.webgl_ctx
            .create_shader(WebGl2RenderingContext::VERTEX_SHADER)
            .ok_or(GraphicsError::Allocate)
    }

    #[inline]
    fn dealloc_carrier(self, ctx: &mut GraphicsCtx) {
        ctx.webgl_ctx.delete_shader(Some(ctx.carrier_for(self)));
    }

    #[inline]
    fn get_data(self, ctx: &GraphicsCtx) -> &Self::Data {
        &ctx.vert_shaders[self]
    }

    #[inline]
    fn get_carrier(self, ctx: &GraphicsCtx) -> &Self::Carrier {
        &ctx.vert_shaders[self].webgl_shader
    }
}

impl GraphicsObject for FragmentShader {
    type Carrier = WebGlShader;
    type Data = ShaderData;

    #[inline]
    fn alloc_carrier(ctx: &mut GraphicsCtx) -> Result<Self::Carrier> {
        ctx.webgl_ctx
            .create_shader(WebGl2RenderingContext::FRAGMENT_SHADER)
            .ok_or(GraphicsError::Allocate)
    }

    #[inline]
    fn dealloc_carrier(self, ctx: &mut GraphicsCtx) {
        ctx.webgl_ctx.delete_shader(Some(ctx.carrier_for(self)));
    }

    #[inline]
    fn get_data(self, ctx: &GraphicsCtx) -> &Self::Data {
        &ctx.frag_shaders[self]
    }

    #[inline]
    fn get_carrier(self, ctx: &GraphicsCtx) -> &Self::Carrier {
        &ctx.frag_shaders[self].webgl_shader
    }
}

impl GraphicsObject for Program {
    type Carrier = WebGlProgram;
    type Data = ProgramData;

    #[inline]
    fn alloc_carrier(ctx: &mut GraphicsCtx) -> Result<Self::Carrier> {
        ctx.webgl_ctx
            .create_program()
            .ok_or(GraphicsError::Allocate)
    }

    #[inline]
    fn dealloc_carrier(self, ctx: &mut GraphicsCtx) {
        ctx.webgl_ctx.delete_program(Some(ctx.carrier_for(self)));
    }

    #[inline]
    fn get_data(self, ctx: &GraphicsCtx) -> &Self::Data {
        &ctx.programs[self]
    }

    #[inline]
    fn get_carrier(self, ctx: &GraphicsCtx) -> &Self::Carrier {
        &ctx.programs[self].webgl_program
    }
}

impl Bindable for Program {
    #[inline]
    fn bind(self, ctx: &GraphicsCtx) {
        ctx.webgl_ctx.use_program(Some(ctx.carrier_for(self)));
    }

    #[inline]
    fn release(self, ctx: &GraphicsCtx) {
        ctx.webgl_ctx.use_program(None);
    }
}

impl GraphicsCtx {
    pub fn mk_vertex_shader<S: AsRef<str>>(&mut self, source: S) -> Result<VertexShader> {
        let allocated = self.alloc::<VertexShader>()?;
        let webgl_shader = self.compile_shader(allocated, source)?;

        Ok(self.vert_shaders.push(ShaderData { webgl_shader }))
    }

    pub fn mk_fragment_shader<S: AsRef<str>>(&mut self, source: S) -> Result<FragmentShader> {
        let allocated = self.alloc::<FragmentShader>()?;
        let webgl_shader = self.compile_shader(allocated, source)?;
        Ok(self.frag_shaders.push(ShaderData { webgl_shader }))
    }

    fn compile_shader<S: AsRef<str>>(
        &mut self,
        webgl_shader: WebGlShader,
        source: S,
    ) -> Result<WebGlShader> {
        // Set shader source
        self.webgl_ctx.shader_source(&webgl_shader, source.as_ref());
        // Attempt to compile the shader
        self.webgl_ctx.compile_shader(&webgl_shader);

        // Check compilation was successful
        if self
            .webgl_ctx
            .get_shader_parameter(&webgl_shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or_default()
        {
            // If so, store shader data and move on
            Ok(webgl_shader)
        } else {
            // Quickly delete shader on error
            self.webgl_ctx.delete_shader(Some(&webgl_shader));
            // And then try to get an error log
            Err(GraphicsError::ShaderCompile(
                self.webgl_ctx
                    .get_shader_info_log(&webgl_shader)
                    .unwrap_or_else(|| "unknown shader compilation failure".to_owned()),
            ))
        }
    }

    pub fn mk_program(&mut self, vert: VertexShader, frag: FragmentShader) -> Result<Program> {
        // Allocate program
        let webgl_program = self.alloc::<Program>()?;

        // Attach shaders and link
        self.webgl_ctx
            .attach_shader(&webgl_program, self.carrier_for(vert));
        self.webgl_ctx
            .attach_shader(&webgl_program, self.carrier_for(frag));
        self.webgl_ctx.link_program(&webgl_program);

        // Check linking was successful
        if self
            .webgl_ctx
            .get_program_parameter(&webgl_program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or_default()
        {
            // If so, store program data and move on
            Ok(self.programs.push(ProgramData { webgl_program }))
        } else {
            // Quickly delete program on error
            self.webgl_ctx.delete_program(Some(&webgl_program));
            // Otherwise, try to get an error log
            Err(GraphicsError::ProgramLink(
                self.webgl_ctx
                    .get_program_info_log(&webgl_program)
                    .unwrap_or_else(|| "unknown program link failure".to_owned()),
            ))
        }
    }
}
