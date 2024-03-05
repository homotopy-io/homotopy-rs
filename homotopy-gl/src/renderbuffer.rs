use web_sys::{WebGl2RenderingContext, WebGlRenderbuffer};

use super::{framebuffer::Attachable, GlCtx, GlCtxHandle, GlCtxHook, GlError, Result};

pub struct Renderbuffer {
    ctx: GlCtxHandle,
    webgl_renderbuffer: WebGlRenderbuffer,
    hook: Option<GlCtxHook>,
}

#[derive(Default, Clone)]
pub struct RenderbufferOpts {
    pub dimensions: Option<(u32, u32)>,
}

impl RenderbufferOpts {
    #[allow(clippy::unused_self)]
    fn resize(&self, gl: &WebGl2RenderingContext, width: u32, height: u32) {
        // NOTE(@doctorn) could pass format as a renderbuffer option
        gl.renderbuffer_storage(
            WebGl2RenderingContext::RENDERBUFFER,
            WebGl2RenderingContext::DEPTH_COMPONENT16,
            width as i32,
            height as i32,
        );
    }

    fn cfg(&self, gl: &WebGl2RenderingContext, width: u32, height: u32) {
        let (width, height) = self.dimensions.unwrap_or((width, height));
        self.resize(gl, width, height);
    }

    fn build_resize_hook(
        &self,
        ctx: GlCtxHandle,
        webgl_renderbuffer: WebGlRenderbuffer,
    ) -> impl Fn(u32, u32) -> Result<()> {
        let opts = self.clone();
        move |width, height| {
            ctx.with_gl(|gl| {
                gl.bind_renderbuffer(
                    WebGl2RenderingContext::RENDERBUFFER,
                    Some(&webgl_renderbuffer),
                );
                opts.resize(gl, width, height);
                gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);
                Ok(())
            })
        }
    }
}

impl Renderbuffer {
    fn alloc(ctx: &GlCtx, opts: &RenderbufferOpts) -> Result<Self> {
        let webgl_renderbuffer = ctx
            .with_gl(WebGl2RenderingContext::create_renderbuffer)
            .ok_or(GlError::Allocate)?;

        let hook = opts.dimensions.is_none().then(|| {
            let hook = opts.build_resize_hook(ctx.ctx_handle(), webgl_renderbuffer.clone());
            ctx.install_resize_hook(hook)
        });

        let renderbuffer = Self {
            ctx: ctx.ctx_handle(),
            webgl_renderbuffer,
            hook,
        };

        renderbuffer.bind(|| {
            ctx.with_gl(|gl| opts.cfg(gl, ctx.width(), ctx.height()));
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
        if let Some(hook) = self.hook {
            self.ctx.remove_resize_hook(hook);
        }

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
