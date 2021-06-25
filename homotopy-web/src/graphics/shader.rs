use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

use homotopy_core::declare_idx;

use super::{GraphicsCtx, GraphicsError, Result};

declare_idx! {
    pub struct Shader = usize;

    pub struct Program = usize;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShaderKind {
    Vert = WebGlRenderingContext::VERTEX_SHADER as isize,
    Frag = WebGlRenderingContext::FRAGMENT_SHADER as isize,
}

pub struct ShaderData {
    kind: ShaderKind,
    webgl_shader: WebGlShader,
}

pub struct ProgramData {
    webgl_program: WebGlProgram,
}

impl ProgramData {
    #[inline]
    pub fn underlying_program(&self) -> &WebGlProgram {
        &self.webgl_program
    }
}

impl GraphicsCtx {
    pub fn compile_shader<S: AsRef<str>>(&mut self, kind: ShaderKind, source: S) -> Result<Shader> {
        // Allocate shader
        let webgl_shader = self
            .webgl_ctx
            .create_shader(kind as u32)
            .ok_or_else(|| GraphicsError::ShaderCompile("could not allocate shader".to_owned()))?;

        // Set shader source
        self.webgl_ctx.shader_source(&webgl_shader, source.as_ref());
        // Attempt to compile the shader
        self.webgl_ctx.compile_shader(&webgl_shader);

        // Check compilation was successful
        if self
            .webgl_ctx
            .get_shader_parameter(&webgl_shader, WebGlRenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or_default()
        {
            // If so, store shader data and move on
            Ok(self.shaders.push(ShaderData { kind, webgl_shader }))
        } else {
            // Otherwise, try to get an error log
            Err(GraphicsError::ShaderCompile(
                self.webgl_ctx
                    .get_shader_info_log(&webgl_shader)
                    .unwrap_or_else(|| "unknown shader compilation failure".to_owned()),
            ))
        }
    }

    pub fn link_program(&mut self, vert: Shader, frag: Shader) -> Result<Program> {
        // We're not using the type system to enforce that the shader kinds match up with WebGL's
        // expectations, so assert that they do here instead
        assert_eq!(
            self.shaders[vert].kind,
            ShaderKind::Vert,
            "non-vertex shader used as a vertex shader"
        );
        assert_eq!(
            self.shaders[frag].kind,
            ShaderKind::Frag,
            "non-fragment shader used as a fragment shader"
        );

        // Allocate program
        let webgl_program = self
            .webgl_ctx
            .create_program()
            .ok_or_else(|| GraphicsError::ProgramLink("could not allocate program".to_owned()))?;

        // Attach shaders and link
        self.webgl_ctx
            .attach_shader(&webgl_program, &self.shaders[vert].webgl_shader);
        self.webgl_ctx
            .attach_shader(&webgl_program, &self.shaders[frag].webgl_shader);
        self.webgl_ctx.link_program(&webgl_program);

        // Check linking was successful
        if self
            .webgl_ctx
            .get_program_parameter(&webgl_program, WebGlRenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or_default()
        {
            // If so, store program data and move on
            Ok(self.programs.push(ProgramData { webgl_program }))
        } else {
            // Otherwise, try to get an error log
            Err(GraphicsError::ProgramLink(
                self.webgl_ctx
                    .get_program_info_log(&webgl_program)
                    .unwrap_or_else(|| "unknown program link failure".to_owned()),
            ))
        }
    }
}
