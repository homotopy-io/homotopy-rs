use yew::prelude::*;

use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use thiserror::Error;

pub mod attribute;

pub mod array;
pub mod buffer;
pub mod frame;
pub mod geom;
pub mod shader;

#[derive(Error, Debug)]
pub enum GraphicsError {
    #[error("failed to attach to WebGL context")]
    Attachment(&'static str),
    #[error("failed to allocate WebGL object")]
    Allocate,
    #[error("failed to compile shader: {0}")]
    ShaderCompile(String),
    #[error("failed to link shader program: {0}")]
    ProgramLink(String),
}

pub type Result<T> = std::result::Result<T, GraphicsError>;

pub struct GraphicsCtx {
    webgl_ctx: WebGl2RenderingContext,
}

impl GraphicsCtx {
    pub fn attach(node_ref: NodeRef) -> Result<Self> {
        let canvas = node_ref.cast::<HtmlCanvasElement>().ok_or_else(|| {
            GraphicsError::Attachment("supplied node ref does not point to a canvas element")
        })?;

        let webgl_ctx = if let Ok(Some(obj)) = canvas.get_context("webgl2") {
            obj.dyn_into::<WebGl2RenderingContext>().map_err(|_| {
                GraphicsError::Attachment("failed to cast WebGL context to a rendering context")
            })?
        } else {
            return Err(GraphicsError::Attachment(
                "failed to get WebGL context for canvas",
            ));
        };

        Ok(Self { webgl_ctx })
    }
}
