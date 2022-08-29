use thiserror::Error;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

#[derive(Error, Debug)]
pub enum TwoDError {
    #[error("failed to attach to 2D context")]
    Attachment(&'static str),
    #[error("failed to rescale canvas")]
    Rescale,
}

pub struct TwoDCtx {
    pub ctx: CanvasRenderingContext2d,
    pub canvas: HtmlCanvasElement,
    pub width: u32,
    pub height: u32,
    pixel_ratio: f64,
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

        Ok(Self {
            ctx,
            width: canvas.width(),
            height: canvas.height(),
            canvas,
            pixel_ratio: 1.,
        })
    }

    fn resize_to(&mut self, width: u32, height: u32) -> Result<()> {
        let x_ratio = (width != self.canvas.width()).then(|| {
            let ratio = f64::from(width) / f64::from(self.canvas.width());
            self.canvas.set_width(width);
            self.width = width;
            ratio
        });
        let y_ratio = (height != self.canvas.height()).then(|| {
            let ratio = f64::from(height) / f64::from(self.canvas.height());
            self.canvas.set_height(height);
            self.height = height;
            ratio
        });

        if x_ratio.is_some() || y_ratio.is_some() {
            self.ctx
                .scale(x_ratio.unwrap_or(1.), y_ratio.unwrap_or(1.))
                .map_err(|_err| TwoDError::Rescale)
        } else {
            Ok(())
        }
    }

    fn resize_to_fit(&mut self) -> Result<()> {
        let correct = |x| f64::ceil(f64::from(x) * self.pixel_ratio) as u32;
        let width = correct(self.canvas.client_width());
        let height = correct(self.canvas.client_height());

        self.resize_to(width, height)
    }

    pub fn set_pixel_ratio(&mut self, pixel_ratio: f64) -> Result<()> {
        if (pixel_ratio - self.pixel_ratio).abs() > f64::EPSILON {
            self.pixel_ratio = pixel_ratio;
            self.resize_to_fit()?;
        }

        Ok(())
    }
}
