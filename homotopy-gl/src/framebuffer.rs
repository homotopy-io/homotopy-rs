use js_sys::Array;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlFramebuffer};

use super::{GlCtx, GlCtxHandle, GlError, Result};

#[derive(Copy, Clone)]
enum AttachmentPoint {
    Color(usize),
    Depth,
}

impl AttachmentPoint {
    const COLOR_ATTACHMENTS: [u32; 16] = [
        WebGl2RenderingContext::COLOR_ATTACHMENT0,
        WebGl2RenderingContext::COLOR_ATTACHMENT1,
        WebGl2RenderingContext::COLOR_ATTACHMENT2,
        WebGl2RenderingContext::COLOR_ATTACHMENT3,
        WebGl2RenderingContext::COLOR_ATTACHMENT4,
        WebGl2RenderingContext::COLOR_ATTACHMENT5,
        WebGl2RenderingContext::COLOR_ATTACHMENT6,
        WebGl2RenderingContext::COLOR_ATTACHMENT7,
        WebGl2RenderingContext::COLOR_ATTACHMENT8,
        WebGl2RenderingContext::COLOR_ATTACHMENT9,
        WebGl2RenderingContext::COLOR_ATTACHMENT10,
        WebGl2RenderingContext::COLOR_ATTACHMENT11,
        WebGl2RenderingContext::COLOR_ATTACHMENT12,
        WebGl2RenderingContext::COLOR_ATTACHMENT13,
        WebGl2RenderingContext::COLOR_ATTACHMENT14,
        WebGl2RenderingContext::COLOR_ATTACHMENT15,
    ];

    fn into_gl_const(self) -> u32 {
        match self {
            Self::Color(i) => Self::COLOR_ATTACHMENTS[i],
            Self::Depth => WebGl2RenderingContext::DEPTH_ATTACHMENT,
        }
    }
}

pub trait Attachable {
    /// # Safety
    ///
    /// Attached objects must not be dropped before the bound framebuffer is dropped
    unsafe fn attach(&self, gl: &WebGl2RenderingContext, target: u32);
}

pub struct Attachment {
    attachable: Box<dyn Attachable>,
    point: AttachmentPoint,
}

impl Attachment {
    fn new<T>(t: T, point: AttachmentPoint) -> Self
    where
        T: Attachable + 'static,
    {
        Self {
            attachable: Box::new(t) as Box<dyn Attachable>,
            point,
        }
    }

    #[inline]
    pub fn color<T>(t: T, id: usize) -> Self
    where
        T: Attachable + 'static,
    {
        Self::new(t, AttachmentPoint::Color(id))
    }

    #[inline]
    pub fn depth<T>(t: T) -> Self
    where
        T: Attachable + 'static,
    {
        Self::new(t, AttachmentPoint::Depth)
    }
}

pub struct Framebuffer {
    ctx: GlCtxHandle,
    attachments: Vec<Attachment>,
    webgl_framebuffer: WebGlFramebuffer,
}

impl Framebuffer {
    fn alloc(ctx: &GlCtx, attachments: Vec<Attachment>) -> Result<Self> {
        let webgl_framebuffer = ctx
            .with_gl(WebGl2RenderingContext::create_framebuffer)
            .ok_or(GlError::Allocate)?;

        let framebuffer = Self {
            ctx: ctx.ctx_handle(),
            attachments,
            webgl_framebuffer,
        };

        let completeness_check = framebuffer.bind(|| {
            ctx.with_gl(|gl| {
                for attachment in &framebuffer.attachments {
                    // SAFETY: we've already taken ownership of all of these objects,
                    // so there is no way that they can be deallocated before the
                    // framebuffer is dropped
                    unsafe {
                        attachment
                            .attachable
                            .attach(gl, attachment.point.into_gl_const());
                    }
                }

                let color_attachments = framebuffer
                    .attachments
                    .iter()
                    .filter_map(|attachment| {
                        if let AttachmentPoint::Color(_) = attachment.point {
                            Some(JsValue::from(attachment.point.into_gl_const()))
                        } else {
                            None
                        }
                    })
                    .collect::<Array>();

                gl.draw_buffers(&color_attachments.into());
                gl.check_framebuffer_status(WebGl2RenderingContext::FRAMEBUFFER)
            })
        });

        if completeness_check == WebGl2RenderingContext::FRAMEBUFFER_COMPLETE {
            Ok(framebuffer)
        } else {
            Err(GlError::Framebuffer)
        }
    }

    #[inline]
    pub(super) fn bind<F, U>(&self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        self.ctx.with_gl(|gl| {
            gl.bind_framebuffer(
                WebGl2RenderingContext::FRAMEBUFFER,
                Some(&self.webgl_framebuffer),
            );
            let result = f();
            gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
            result
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
    pub fn mk_framebuffer(&self, attachments: Vec<Attachment>) -> Result<Framebuffer> {
        Framebuffer::alloc(self, attachments)
    }
}
