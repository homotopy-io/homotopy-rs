use homotopy_graphics::{
    clay::{Scene, ViewDimension},
    draw,
    gl::{
        frame::Frame,
        framebuffer::{Attachment, Framebuffer},
        texture::{InternalFormat, Texture, TextureOpts, Type},
        GlCtx, Result,
    },
};
use ultraviolet::{Vec2, Vec3, Vec4};

use super::{orbit_camera::OrbitCamera, GlDiagramProps};
use crate::{
    app::AppSettings,
    components::{settings::Store, Finger},
    model::proof::Signature,
};

pub struct Renderer {
    ctx: GlCtx,
    scene: Scene,
    gbuffer: GBuffer,
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
            gbuffer: GBuffer::new(&ctx)?,
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

    pub fn render(&mut self, dimension: u8, camera: &OrbitCamera, settings: &Store<AppSettings>) {
        let mut frame = Frame::new(&mut self.ctx).with_clear_color(Vec4::broadcast(1.));
        let vp = camera.transform(&*frame);

        if !*settings.get_mesh_hidden() {
            let normals = *settings.get_debug_normals();
            let lighting = *settings.get_disable_lighting();
            let camera = camera.position();
            let signature = &self.signature;

            if dimension <= 3 {
                self.scene.draw(&mut frame, |generator, array| {
                    let color = signature
                        .generator_info(generator)
                        .unwrap()
                        .color
                        .0
                        .into_format();
                    draw!(array, {
                        mvp: vp,
                        debug_normals: normals,
                        lighting_disable: lighting,
                        camera_pos: camera,
                        d: Vec3::new(color.red, color.green, color.blue),
                    })
                });
            } else {
                // TODO(@doctorn) something sensible for time control
                let t = f32::sin(0.00025 * self.t);

                self.scene.draw(&mut frame, |generator, array| {
                    let color = if generator.id == 0 {
                        Vec3::new(30. / 255., 144. / 255., 1.)
                    } else {
                        Vec3::zero()
                    };

                    draw!(array, {
                        mvp: vp,
                        debug_normals: normals,
                        lighting_disable: lighting,
                        camera_pos: camera,
                        t: t,
                        d: color,
                    })
                });
            }
        }

        if *settings.get_wireframe_3d() {
            self.scene.draw_wireframe(&mut frame, &vp);
        }

        if *settings.get_debug_axes() {
            self.scene.draw_axes(&mut frame, &vp);
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
