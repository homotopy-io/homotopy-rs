use homotopy_gl::gl::{
    framebuffer::{Attachment, Framebuffer},
    texture::{InternalFormat, Texture, TextureOpts, Type},
    GlCtx, Result,
};

pub struct GBuffer {
    pub framebuffer: Framebuffer,
    pub positions: Texture,
    pub normals: Texture,
    pub albedo: Texture,
}

impl GBuffer {
    pub fn new(ctx: &GlCtx) -> Result<Self> {
        let positions = ctx.mk_texture_with_opts(&TextureOpts {
            internal_format: InternalFormat::Rgba16F,
            type_: Type::Float,
            ..Default::default()
        })?;
        let normals = ctx.mk_texture_with_opts(&TextureOpts {
            internal_format: InternalFormat::Rgba16F,
            type_: Type::Float,
            ..Default::default()
        })?;
        let albedo = ctx.mk_texture()?;
        let renderbuffer = ctx.mk_renderbuffer()?;

        let framebuffer = ctx.mk_framebuffer(vec![
            Attachment::color(positions.clone(), 0),
            Attachment::color(normals.clone(), 1),
            Attachment::color(albedo.clone(), 2),
            Attachment::depth(renderbuffer),
        ])?;

        Ok(Self {
            framebuffer,
            positions,
            normals,
            albedo,
        })
    }
}
