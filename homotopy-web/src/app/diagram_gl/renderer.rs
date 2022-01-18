use homotopy_graphics::{
    draw,
    gl::{
        frame::{Clear, Frame},
        framebuffer::{Attachment, Framebuffer},
        texture::{InternalFormat, Texture, TextureOpts, Type},
        GlCtx, Result,
    },
};
use ultraviolet::{Vec3, Vec4};

use super::{
    orbit_camera::OrbitCamera,
    scene::{Scene, ViewDimension},
    GlDiagramProps,
};
use crate::{app::AppSettings, components::settings::Store, model::proof::Signature};

pub struct Renderer {
    ctx: GlCtx,
    scene: Scene,
    gbuffer: GBuffer,
    kernel_gbuffer: GBuffer,
    signature: Signature,
    subdivision_depth: u8,
    geometry_samples: u8,
    t: f32,
}

struct GBuffer {
    framebuffer: Framebuffer,
    positions: Texture,
    normals: Texture,
    albedo: Texture,
}

impl Renderer {
    pub fn new(ctx: GlCtx, settings: &Store<AppSettings>, props: &GlDiagramProps) -> Result<Self> {
        let depth = *settings.get_subdivision_depth() as u8;
        let samples = *settings.get_geometry_samples() as u8;
        let kernel_gbuffer = GBuffer::new(&ctx)?;
        let gbuffer = GBuffer::new(&ctx)?;

        Ok(Self {
            scene: Scene::build(
                &ctx,
                &props.diagram,
                if props.view.dimension() <= 3 {
                    ViewDimension::Three
                } else {
                    ViewDimension::Four
                },
                depth,
                samples,
            )?,
            kernel_gbuffer,
            gbuffer,
            ctx,
            signature: props.signature.clone(),
            subdivision_depth: depth,
            geometry_samples: samples,
            t: 0.0,
        })
    }

    pub fn update(&mut self, settings: &Store<AppSettings>, dt: f32) -> Result<()> {
        let depth = *settings.get_subdivision_depth() as u8;
        let samples = *settings.get_geometry_samples() as u8;

        if self.subdivision_depth != depth || self.geometry_samples != samples {
            self.subdivision_depth = depth;
            self.geometry_samples = samples;
            self.scene.reload_meshes(&self.ctx, depth, samples)?;
        }

        self.t += dt;

        Ok(())
    }

    pub fn render(&mut self, camera: &OrbitCamera, settings: &Store<AppSettings>) {
        let vp = camera.transform(&self.ctx);

        // Render wireframe to kernel GBuffer
        {
            let mut frame = Frame::new(&mut self.ctx)
                .with_frame_buffer(&self.kernel_gbuffer.framebuffer)
                .with_clear_color(Vec4::new(0., 0., 0., 1.));

            if !*settings.get_mesh_hidden() {
                self.scene.draw_kernels(&mut frame, |_, array| {
                    draw!(array, &[], {
                        mvp: vp,
                        albedo: Vec3::new(1., 0., 0.),
                        t: f32::sin(0.00025 * self.t),
                    })
                });
            }
        }

        // Surfaces
        {
            let mut frame = Frame::new(&mut self.ctx)
                .with_frame_buffer(&self.gbuffer.framebuffer)
                .with_clear_color(Vec4::new(0., 0., 0., 1.));

            if !*settings.get_mesh_hidden() {
                let signature = &self.signature;

                self.scene.draw(&mut frame, |generator, array| {
                    let color = signature.generator_info(generator).map_or(
                        palette::rgb::Rgb {
                            red: 0.,
                            green: 0.,
                            blue: 1.,
                            ..Default::default()
                        },
                        |info| info.color.0.into_format(),
                    );
                    draw!(array, &[], {
                        mvp: vp,
                        albedo: Vec3::new(color.red, color.green, color.blue),
                        t: f32::sin(0.00025 * self.t),
                    })
                });

                self.scene.kernel_pass(
                    &mut frame,
                    &[&self.kernel_gbuffer.positions, &self.kernel_gbuffer.albedo],
                );
            }

            if *settings.get_wireframe_3d() {
                self.scene.draw_wireframe(&mut frame, &vp);
            }

            if *settings.get_debug_axes() {
                self.scene.draw_axes(&mut frame, &vp);
            }
        }

        // Lighting pass
        {
            let mut frame = Frame::new(&mut self.ctx).with_clear_color(Vec4::new(1., 1., 1., 1.));
            self.scene.lighting_pass(
                &mut frame,
                &[
                    &self.gbuffer.positions,
                    &self.gbuffer.normals,
                    &self.gbuffer.albedo,
                ],
                camera.position(),
            );
        }
    }
}

impl GBuffer {
    fn new(ctx: &GlCtx) -> Result<Self> {
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
