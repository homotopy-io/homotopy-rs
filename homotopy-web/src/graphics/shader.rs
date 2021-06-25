use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

// use homotopy_core::declare_idx;

use super::GraphicsCtx;

// TODO(@doctorn)
// declare_idx! {
//     pub struct Shader = usize;
// }

pub type Shader = ();

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShaderKind {
    Vert = WebGlRenderingContext::VERTEX_SHADER as isize,
    Frag = WebGlRenderingContext::FRAGMENT_SHADER as isize,
}

pub struct ShaderData {
    kind: ShaderKind,
    webgl_shader: WebGlShader,
}

pub type Program = ();

pub struct ProgramData {
    webgl_program: WebGlProgram,
}

impl GraphicsCtx {
    pub fn compile_shader<S: AsRef<str>>(&self, kind: ShaderKind, source: S) -> Result<Shader, ()> {
        // Allocate shader
        let webgl_shader = self
            .webgl_ctx
            .create_shader(kind as u32)
            .ok_or_else(|| ())?; // TODO(@doctorn)

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
            // TODO(@doctorn)
            // Otherwise, try to get an error log
            Err(())
        }
    }

    pub fn link_program(&self, vert: Shader, frag: Shader) -> Result<Program, ()> {
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
        let webgl_program = self.webgl_ctx.create_program().ok_or_else(|| ())?; // TODO(@doctorn)

        // Attach shaders and link
        self.webgl_ctx
            .attach_shader(&webgl_program, &self.shaders[vert].webgl_shader);
        self.webgl_ctx
            .attach_shader(&webgl_program, &self.shaders[frag].webgl_shader);
        self.webgl_ctx.link_program(&webgl_program);

        // Check linking was successful
        if self.webgl_ctx.get_program_parameter(&webgl_program, WebGlRenderingContext::LINK_STATUS).as_bool().unwrap_or_default() {
            // If so, store program data and move on
            Ok(self.programs.push(ProgramData { webgl_program }))
        } else {
            // TODO(@doctorn)
            // Otherwise, try to get an error log
            Err(())
        }
    }
}
