use std::ops::Deref;

use thiserror::Error;
use ultraviolet::Vec2;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use yew::prelude::*;

pub mod array;
pub mod buffer;
pub mod frame;
pub mod framebuffer;
pub mod renderbuffer;
pub mod shader;
pub mod texture;

#[derive(Error, Debug)]
pub enum GlError {
    #[error("failed to allocate WebGL object")]
    Allocate,
    #[error("failed to attach to WebGL context")]
    Attachment(&'static str),
    #[error("missing WebGL extension")]
    ExtensionMissing,
    #[error("incomplete framebuffer")]
    Framebuffer,
    #[error("failed to compile shader: {0}")]
    ShaderCompile(String),
    #[error("failed to generate texture")]
    Texture,
    #[error("failed to link shader program: {0}")]
    ProgramLink(String),
    #[error("failed to bind vertex array attribute: {0}")]
    BindAttribute(String),
    #[error("failed to pass uniform value: {0}")]
    Uniform(String),
}

mod ctx {
    use std::{cell::RefCell, rc::Rc};

    use homotopy_common::{declare_idx, dense::DenseVec};
    use web_sys::WebGl2RenderingContext;

    use super::Result;

    declare_idx! {
        pub struct GlCtxHook = usize;
    }

    type ResizeHook = Box<dyn Fn(u32, u32) -> Result<()>>;

    pub struct GlCtxInner {
        ctx: WebGl2RenderingContext,
        resize_hooks: RefCell<DenseVec<GlCtxHook, ResizeHook>>,
    }

    #[derive(Clone)]
    pub struct GlCtxHandle(Rc<GlCtxInner>);

    impl GlCtxHandle {
        #[inline]
        pub(super) fn new(ctx: WebGl2RenderingContext) -> Self {
            Self(Rc::new(GlCtxInner {
                ctx,
                resize_hooks: Default::default(),
            }))
        }

        #[allow(clippy::inline_always)]
        #[inline(always)]
        pub(super) fn with_gl<F, T>(&self, f: F) -> T
        where
            F: FnOnce(&WebGl2RenderingContext) -> T,
        {
            f(&self.0.ctx)
        }

        #[inline]
        pub(super) fn remove_resize_hook(&self, hook: GlCtxHook) {
            self.0.resize_hooks.borrow_mut().remove(hook);
        }

        #[inline]
        pub(super) fn install_resize_hook<F>(&self, f: F) -> GlCtxHook
        where
            F: Fn(u32, u32) -> super::Result<()> + 'static,
        {
            self.0
                .resize_hooks
                .borrow_mut()
                .push(Box::new(f) as Box<dyn Fn(u32, u32) -> Result<()>>)
        }

        pub(super) fn run_hooks(&self, width: u32, height: u32) -> Result<()> {
            for hook in self.0.resize_hooks.borrow().values() {
                hook(width, height)?;
            }

            Ok(())
        }
    }
}

use ctx::{GlCtxHandle, GlCtxHook};

pub type Result<T> = std::result::Result<T, GlError>;

pub struct GlCtx {
    ctx: GlCtxHandle,
    canvas: HtmlCanvasElement,
    width: u32,
    height: u32,
    pixel_ratio: f64,
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

        // We need this for deferred shading
        webgl_ctx
            .get_extension("EXT_color_buffer_float")
            .map_err(|_| GlError::ExtensionMissing)?;

        Ok(Self {
            ctx: GlCtxHandle::new(webgl_ctx),
            width: canvas.width(),
            height: canvas.height(),
            canvas,
            pixel_ratio: 1.,
        })
    }

    fn resize_to(&mut self, width: u32, height: u32) -> Result<()> {
        if width != self.canvas.width() || height != self.canvas.height() {
            self.canvas.set_width(width);
            self.canvas.set_height(height);

            self.width = width;
            self.height = height;

            self.ctx.run_hooks(width, height)?;
        }

        self.ctx
            .with_gl(|gl| gl.viewport(0, 0, width as i32, height as i32));

        Ok(())
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

    #[inline]
    fn ctx_handle(&self) -> GlCtxHandle {
        self.ctx.clone()
    }

    #[inline]
    #[must_use]
    pub fn to_ndc(&self, v: Vec2) -> Vec2 {
        2. * (Vec2::new(v.x, -v.y) / self.size()) + Vec2::new(-1., 1.)
    }

    #[inline]
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    #[must_use]
    pub const fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    #[inline]
    #[must_use]
    pub fn aspect_ratio(&self) -> f32 {
        (self.width as f32) / (self.height as f32)
    }
}

impl Deref for GlCtx {
    type Target = GlCtxHandle;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}
