use web_sys::{WebGl2RenderingContext, WebGlRenderbuffer};

use super::{framebuffer::Attachable, GlCtx, GlCtxHandle, GlError, Result};

pub struct Renderbuffer {
    ctx: GlCtxHandle,
    webgl_renderbuffer: WebGlRenderbuffer,
}

#[derive(Default)]
pub struct RenderbufferOpts {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl Renderbuffer {
    fn alloc(ctx: &GlCtx, opts: &RenderbufferOpts) -> Result<Self> {
        let webgl_renderbuffer = ctx
            .with_gl(WebGl2RenderingContext::create_renderbuffer)
            .ok_or(GlError::Allocate)?;

        let renderbuffer = Self {
            ctx: ctx.ctx_handle(),
            webgl_renderbuffer,
        };

        renderbuffer.bind(|| {
            ctx.with_gl(|gl| {
                // NOTE could pass format as a renderbuffer option
                gl.renderbuffer_storage(
                    WebGl2RenderingContext::RENDERBUFFER,
                    WebGl2RenderingContext::DEPTH_COMPONENT16,
                    // FIXME(@doctorn) this needs to be adaptive
                    opts.width.unwrap_or_else(|| ctx.width()) as i32,
                    opts.height.unwrap_or_else(|| ctx.height()) as i32,
                );
            });
        });

        Ok(renderbuffer)
    }

    #[inline]
    pub(super) fn bind<F, U>(&self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        self.ctx.with_gl(|gl| {
            gl.bind_renderbuffer(
                WebGl2RenderingContext::RENDERBUFFER,
                Some(&self.webgl_renderbuffer),
            );
            let result = f();
            gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);
            result
        })
    }
}

impl Attachable for Renderbuffer {
    #[inline]
    unsafe fn attach(&self, gl: &WebGl2RenderingContext, target: u32) {
        gl.framebuffer_renderbuffer(
            WebGl2RenderingContext::FRAMEBUFFER,
            target,
            WebGl2RenderingContext::RENDERBUFFER,
            Some(&self.webgl_renderbuffer),
        );
    }
}

impl Drop for Renderbuffer {
    #[inline]
    fn drop(&mut self) {
        self.ctx
            .with_gl(|gl| gl.delete_renderbuffer(Some(&self.webgl_renderbuffer)));
    }
}

impl GlCtx {
    #[inline]
    pub fn mk_renderbuffer_with_opts(&self, opts: &RenderbufferOpts) -> Result<Renderbuffer> {
        Renderbuffer::alloc(self, opts)
    }

    #[inline]
    pub fn mk_renderbuffer(&self) -> Result<Renderbuffer> {
        self.mk_renderbuffer_with_opts(&Default::default())
    }
}
