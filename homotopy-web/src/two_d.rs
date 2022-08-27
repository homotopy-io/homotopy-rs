use thiserror::Error;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

#[derive(Error, Debug)]
pub enum TwoDError {
    #[error("failed to attach to 2D context")]
    Attachment(&'static str),
}

pub struct TwoDCtx {
    pub ctx: CanvasRenderingContext2d,
    pub canvas: HtmlCanvasElement,
}

pub type Result<T> = std::result::Result<T, TwoDError>;

impl TwoDCtx {
    pub fn attach(node_ref: &NodeRef) -> Result<Self> {
        let canvas = node_ref
            .cast::<HtmlCanvasElement>()
            .ok_or(TwoDError::Attachment(
                "supplied node ref does not point to a canvas element",
            ))?;
        let ctx = if let Ok(Some(obj)) = canvas.get_context("2d") {
            obj.dyn_into::<CanvasRenderingContext2d>().map_err(|_err| {
                TwoDError::Attachment("failed to cast 2D context to a rendering context")
            })?
        } else {
            return Err(TwoDError::Attachment("failed to get 2D context for canvas"));
        };

        Ok(Self { ctx, canvas })
    }
}
