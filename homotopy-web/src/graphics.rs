use web_sys::WebGlRenderingContext;

// use homotopy_core::idx::IdxVec;

mod shader;

pub struct GraphicsCtx {
    webgl_ctx: WebGlRenderingContext,
    // TODO(@doctorn)
    shaders: IdxVec<shader::Shader, shader::ShaderData>,
    programs: IdxVec<shader::Program, shader::ProgramData>,
}
