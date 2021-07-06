use yew::prelude::*;

use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlRenderingContext};

use thiserror::Error;

use homotopy_core::idx::IdxVec;

mod util;

pub mod frame;
pub mod geom;
pub mod shader;
mod vertex_buffer;

use self::shader::{Program, ProgramData, Shader, ShaderData};
use self::vertex_buffer::{VertexBuffer, VertexBufferData};

pub mod buffer {
    pub use super::vertex_buffer::VertexBuffer;
}

#[derive(Error, Debug)]
pub enum GraphicsError {
    #[error("failed to attach to WebGL context")]
    Attachment(&'static str),
    #[error("failed to allocate buffer")]
    BufferAllocate,
    #[error("failed to compile shader: {0}")]
    ShaderCompile(String),
    #[error("failed to link shader program: {0}")]
    ProgramLink(String),
}

pub type Result<T> = std::result::Result<T, GraphicsError>;

pub struct GraphicsCtx {
    webgl_ctx: WebGlRenderingContext,
    shaders: IdxVec<Shader, ShaderData>,
    programs: IdxVec<Program, ProgramData>,
    vertex_buffers: IdxVec<VertexBuffer, VertexBufferData>,
}

impl GraphicsCtx {
    pub fn attach(node_ref: NodeRef) -> Result<Self> {
        let canvas = node_ref.cast::<HtmlCanvasElement>().ok_or_else(|| {
            GraphicsError::Attachment("supplied node ref does not point to a canvas element")
        })?;

        let webgl_ctx = if let Ok(Some(obj)) = canvas.get_context("webgl") {
            obj.dyn_into::<WebGlRenderingContext>().map_err(|_| {
                GraphicsError::Attachment("failed to cast WebGL context to a rendering context")
            })?
        } else {
            return Err(GraphicsError::Attachment(
                "failed to get WebGL context for canvas",
            ));
        };

        Ok(Self {
            webgl_ctx,
            shaders: IdxVec::new(),
            programs: IdxVec::new(),
            vertex_buffers: IdxVec::new(),
        })
    }
}
