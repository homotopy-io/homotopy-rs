use thiserror::Error;
use ultraviolet::Vec2;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use yew::prelude::*;

pub mod array;
pub mod buffer;
pub mod frame;
pub mod shader;

#[derive(Error, Debug)]
pub enum GlError {
    #[error("failed to attach to WebGL context")]
    Attachment(&'static str),
    #[error("failed to allocate WebGL object")]
    Allocate,
    #[error("failed to compile shader: {0}")]
    ShaderCompile(String),
    #[error("failed to link shader program: {0}")]
    ProgramLink(String),
    #[error("failed to bind vertex array attribute: {0}")]
    BindAttribute(String),
    #[error("failed to pass uniform value: {0}")]
    Uniform(String),
}

pub type Result<T> = std::result::Result<T, GlError>;

pub struct GlCtx {
    webgl_ctx: WebGl2RenderingContext,
    canvas: HtmlCanvasElement,

    width: u32,
    height: u32,
}

impl GlCtx {
    #[allow(clippy::map_err_ignore)]
    pub fn attach(node_ref: &NodeRef) -> Result<Self> {
        let canvas = node_ref
            .cast::<HtmlCanvasElement>()
            .ok_or(GlError::Attachment(
                "supplied node ref does not point to a canvas element",
            ))?;

        let webgl_ctx = if let Ok(Some(obj)) = canvas.get_context("webgl2") {
            obj.dyn_into::<WebGl2RenderingContext>().map_err(|_| {
                GlError::Attachment("failed to cast WebGL context to a rendering context")
            })?
        } else {
            return Err(GlError::Attachment(
                "failed to get WebGL context for canvas",
            ));
        };

        webgl_ctx.enable(WebGl2RenderingContext::DEPTH_TEST);

        Ok(Self {
            webgl_ctx,
            width: canvas.width(),
            height: canvas.height(),
            canvas,
        })
    }

    fn resize_to(&mut self, width: u32, height: u32) {
        if width != self.canvas.width() || height != self.canvas.height() {
            self.canvas.set_width(width);
            self.canvas.set_height(height);

            self.width = width;
            self.height = height;
        }

        self.webgl_ctx.viewport(0, 0, width as i32, height as i32);
    }

    fn resize_to_fit(&mut self) {
        let width = self.canvas.client_width();
        let height = self.canvas.client_height();

        self.resize_to(width as u32, height as u32);
    }

    #[inline]
    pub fn to_ndc(&self, v: Vec2) -> Vec2 {
        2. * (Vec2::new(v.x, -v.y) / self.size()) + Vec2::new(-1., 1.)
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    #[inline]
    pub fn aspect_ratio(&self) -> f32 {
        (self.width as f32) / (self.height as f32)
    }
}
