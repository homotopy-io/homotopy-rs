use std::rc::Rc;

use web_sys::{WebGl2RenderingContext, WebGlTexture};

use super::{GlCtx, GlCtxHandle, GlError, Result};

struct TextureData {
    ctx: GlCtxHandle,
    webgl_texture: WebGlTexture,
}

pub enum Filter {
    Nearest = WebGl2RenderingContext::NEAREST as isize,
    Linear = WebGl2RenderingContext::LINEAR as isize,
}

pub enum Kind {
    Float = WebGl2RenderingContext::FLOAT as isize,
    UnsignedByte = WebGl2RenderingContext::UNSIGNED_SHORT as isize,
}

pub enum Format {
    Rgba16f = WebGl2RenderingContext::RGBA16F as isize,
    Rgba = WebGl2RenderingContext::RGBA as isize,
}

impl Default for Filter {
    fn default() -> Self {
        Self::Nearest
    }
}

impl Default for Kind {
    fn default() -> Self {
        Self::Float
    }
}

impl Default for Format {
    fn default() -> Self {
        Self::Rgba
    }
}

#[derive(Default)]
pub struct TextureOpts {
    pub format: Format,
    pub internal_format: Format,
    pub min_filter: Filter,
    pub mag_filter: Filter,
    pub kind: Kind,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Clone)]
pub struct Texture(Rc<TextureData>);

impl Texture {
    #[allow(clippy::map_err_ignore)]
    fn alloc(ctx: &GlCtx, opts: TextureOpts) -> Result<Self> {
        let webgl_texture = ctx
            .with_gl(WebGl2RenderingContext::create_texture)
            .ok_or(GlError::Allocate)?;

        let texture = Self(Rc::new(TextureData {
            ctx: ctx.ctx_handle(),
            webgl_texture,
        }));

        texture
            .bind(|| {
                ctx.with_gl(|gl| {
                    gl.tex_parameteri(
                        WebGl2RenderingContext::TEXTURE_2D,
                        WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                        opts.min_filter as i32,
                    );
                    gl.tex_parameteri(
                        WebGl2RenderingContext::TEXTURE_2D,
                        WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                        opts.mag_filter as i32,
                    );
                    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                        WebGl2RenderingContext::TEXTURE_2D,
                        0,
                        opts.internal_format as i32,
                        // FIXME(@doctorn) this needs to be adaptive
                        opts.width.unwrap_or_else(|| ctx.width()) as i32,
                        opts.height.unwrap_or_else(|| ctx.height()) as i32,
                        0,
                        opts.format as u32,
                        opts.kind as u32,
                        None,
                    )
                })
            })
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
}

impl Drop for TextureData {
    #[inline]
    fn drop(&mut self) {
        self.ctx
            .with_gl(|gl| gl.delete_texture(Some(&self.webgl_texture)));
    }
}

impl GlCtx {
    #[inline]
    pub fn mk_texture(&self, opts: TextureOpts) -> Result<Texture> {
        Texture::alloc(self, opts)
    }
}
