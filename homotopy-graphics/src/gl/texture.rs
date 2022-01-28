use std::rc::Rc;

use web_sys::{WebGl2RenderingContext, WebGlTexture};

use super::{framebuffer::Attachable, GlCtx, GlCtxHandle, GlCtxHook, GlError, Result};

#[derive(Copy, Clone)]
pub enum Filter {
    Nearest = WebGl2RenderingContext::NEAREST as isize,
    Linear = WebGl2RenderingContext::LINEAR as isize,
}

impl Default for Filter {
    fn default() -> Self {
        Self::Nearest
    }
}

#[derive(Copy, Clone)]
pub enum InternalFormat {
    Rgba16F = WebGl2RenderingContext::RGBA16F as isize,
    Rgba = WebGl2RenderingContext::RGBA as isize,
}

#[derive(Copy, Clone)]
pub enum Type {
    UnsignedByte = WebGl2RenderingContext::UNSIGNED_BYTE as isize,
    Float = WebGl2RenderingContext::FLOAT as isize,
}

impl Default for Type {
    #[inline]
    fn default() -> Self {
        Self::UnsignedByte
    }
}

impl Default for InternalFormat {
    #[inline]
    fn default() -> Self {
        Self::Rgba
    }
}

#[derive(Default, Clone)]
pub struct TextureOpts {
    pub min_filter: Filter,
    pub mag_filter: Filter,
    pub internal_format: InternalFormat,
    pub dimensions: Option<(u32, u32)>,
    pub type_: Type,
}

impl TextureOpts {
    #[allow(clippy::map_err_ignore)]
    fn resize(&self, gl: &WebGl2RenderingContext, width: u32, height: u32) -> Result<()> {
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            self.internal_format as i32,
            width as i32,
            height as i32,
            0,
            WebGl2RenderingContext::RGBA,
            self.type_ as u32,
            None,
        )
        .map_err(|_| GlError::Texture)
    }

    fn cfg(&self, gl: &WebGl2RenderingContext, width: u32, height: u32) -> Result<()> {
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            self.min_filter as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            self.mag_filter as i32,
        );
        // NOTE this could be made configurable, but for now just zero
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );

        let (width, height) = self.dimensions.unwrap_or((width, height));
        self.resize(gl, width, height)
    }

    fn build_resize_hook(
        &self,
        ctx: GlCtxHandle,
        webgl_texture: WebGlTexture,
    ) -> impl Fn(u32, u32) -> Result<()> {
        let opts = self.clone();
        move |width, height| {
            ctx.with_gl(|gl| {
                gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&webgl_texture));
                opts.resize(gl, width, height)?;
                gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
                Ok(())
            })
        }
    }
}

struct TextureData {
    ctx: GlCtxHandle,
    webgl_texture: WebGlTexture,
    hook: Option<GlCtxHook>,
}

#[derive(Clone)]
pub struct Texture(Rc<TextureData>);

impl Texture {
    #[allow(clippy::map_err_ignore)]
    fn alloc(ctx: &GlCtx, opts: &TextureOpts) -> Result<Self> {
        let webgl_texture = ctx
            .with_gl(WebGl2RenderingContext::create_texture)
            .ok_or(GlError::Allocate)?;

        let hook = if opts.dimensions.is_none() {
            let hook = opts.build_resize_hook(ctx.ctx_handle(), webgl_texture.clone());
            Some(ctx.install_resize_hook(hook))
        } else {
            None
        };

        let texture = Self(Rc::new(TextureData {
            ctx: ctx.ctx_handle(),
            hook,
            webgl_texture,
        }));

        texture
            .bind(|| ctx.with_gl(|gl| opts.cfg(gl, ctx.width(), ctx.height())))
            .map_err(|_| GlError::Texture)?;

        Ok(texture)
    }

    #[inline]
    pub(super) fn bind<F, U>(&self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        self.0.ctx.with_gl(|gl| {
            gl.bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&self.0.webgl_texture),
            );
            let result = f();
            gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
            result
        })
    }

    #[inline]
    pub(super) fn activate(&self, i: usize) {
        const GL_TEXTURES: [u32; 3] = [
            WebGl2RenderingContext::TEXTURE0,
            WebGl2RenderingContext::TEXTURE1,
            WebGl2RenderingContext::TEXTURE2,
        ];

        // TODO(@doctorn) check if we should be unbinding
        self.0.ctx.with_gl(|gl| {
            gl.active_texture(GL_TEXTURES[i]);
            gl.bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&self.0.webgl_texture),
            );
        });
    }
}

impl Attachable for Texture {
    #[inline]
    unsafe fn attach(&self, gl: &WebGl2RenderingContext, target: u32) {
        gl.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            target,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&self.0.webgl_texture),
            0,
        );
    }
}

impl Drop for TextureData {
    #[inline]
    fn drop(&mut self) {
        if let Some(hook) = self.hook {
            self.ctx.remove_resize_hook(hook);
        }

        self.ctx
            .with_gl(|gl| gl.delete_texture(Some(&self.webgl_texture)));
    }
}

impl GlCtx {
    #[inline]
    pub fn mk_texture_with_opts(&self, opts: &TextureOpts) -> Result<Texture> {
        Texture::alloc(self, opts)
    }

    #[inline]
    pub fn mk_texture(&self) -> Result<Texture> {
        self.mk_texture_with_opts(&Default::default())
    }
}
