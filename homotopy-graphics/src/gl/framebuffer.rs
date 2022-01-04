use web_sys::{WebGl2RenderingContext, WebGlFramebuffer};

use super::{GlCtx, GlCtxHandle, GlError, Result};

pub struct Framebuffer {
    ctx: GlCtxHandle,
    webgl_framebuffer: WebGlFramebuffer,
}

impl Framebuffer {
    fn alloc(ctx: &GlCtx) -> Result<Self> {
        let webgl_framebuffer = ctx
            .with_gl(WebGl2RenderingContext::create_framebuffer)
            .ok_or(GlError::Allocate)?;

        Ok(Self {
            ctx: ctx.ctx_handle(),
            webgl_framebuffer,
        })
    }
}

impl Drop for Framebuffer {
    #[inline]
    fn drop(&mut self) {
        self.ctx
            .with_gl(|gl| gl.delete_framebuffer(Some(&self.webgl_framebuffer)));
    }
}

impl GlCtx {
    #[inline]
    pub fn mk_framebuffer(&self) -> Result<Framebuffer> {
        Framebuffer::alloc(self)
    }
}
