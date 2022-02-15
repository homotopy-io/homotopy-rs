use homotopy_core::Generator;
use homotopy_graphics::{
    draw,
    gl::{
        frame::{DepthTest, Frame},
        GlCtx, Result,
    },
};
use ultraviolet::{Mat4, Vec3, Vec4};

use self::{axes::Axes, gbuffer::GBuffer, quad::Quad, scene::Scene, shaders::Shaders};
use super::{orbit_camera::OrbitCamera, GlDiagramProps};
use crate::{app::AppSettings, components::settings::Store, model::proof::Signature};

mod axes;
mod gbuffer;
mod quad;
mod scene;
mod shaders;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ViewDimension {
    Three = 3,
    Four = 4,
}

pub struct Renderer {
    // outside world
    ctx: GlCtx,
    signature: Signature,
    // state
    subdivision_depth: u8,
    geometry_samples: u8,
    // resources
    shaders: Shaders,
    scene: Scene,
    axes: Axes,
    quad: Quad,
    // pipeline state
    gbuffer: GBuffer,
    cylinder_buffer: GBuffer,
}

impl Renderer {
    pub fn new(ctx: GlCtx, settings: &Store<AppSettings>, props: &GlDiagramProps) -> Result<Self> {
        let depth = *settings.get_subdivision_depth() as u8;
        let samples = *settings.get_geometry_samples() as u8;

        Ok(Self {
            scene: Scene::new(
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
            shaders: Shaders::new(&ctx)?,
            axes: Axes::new(&ctx)?,
            quad: Quad::new(&ctx)?,
            gbuffer: GBuffer::new(&ctx)?,
            cylinder_buffer: GBuffer::new(&ctx)?,
            ctx,
            signature: props.signature.clone(),
            subdivision_depth: depth,
            geometry_samples: samples,
        })
    }

    pub fn update(&mut self, settings: &Store<AppSettings>) -> Result<()> {
        let depth = *settings.get_subdivision_depth() as u8;
        let samples = *settings.get_geometry_samples() as u8;
        let pixel_ratio = if *settings.get_dpr_scale() {
            web_sys::window().unwrap().device_pixel_ratio()
        } else {
            1.
        };

        self.ctx.set_pixel_ratio(pixel_ratio)?;

        if self.subdivision_depth != depth || self.geometry_samples != samples {
            self.subdivision_depth = depth;
            self.geometry_samples = samples;
            self.scene.reload_meshes(&self.ctx, depth, samples)?;
        }

        Ok(())
    }

    pub fn render(&mut self, camera: &OrbitCamera, settings: &Store<AppSettings>, t: f32) {
        let duration = self.scene.diagram.size() as f32;
        let geometry_scale = *settings.get_geometry_scale() as f32 / 10.;

        let v = camera.view_transform(&self.ctx);
        let p = camera.perspective_transform(&self.ctx);

        let program = if self.scene.view_dimension == ViewDimension::Three {
            &self.shaders.geometry_3d
        } else {
            &self.shaders.geometry_4d
        };

        let signature = &self.signature;
        let color_of = |generator: &Generator| {
            let color = signature.generator_info(*generator).map_or(
                palette::rgb::Rgb {
                    red: 0.,
                    green: 0.,
                    blue: 1.,
                    ..Default::default()
                },
                |info| info.color.0.into_format(),
            );
            Vec3::new(color.red, color.green, color.blue)
        };

        // Render animated wireframes to cylinder buffer
        if self.scene.view_dimension == ViewDimension::Four {
            let mut frame = Frame::new(&mut self.ctx)
                .with_frame_buffer(&self.cylinder_buffer.framebuffer)
                .with_clear_color(Vec4::new(0., 0., 0., 0.));

            if !*settings.get_mesh_hidden() {
                for (generator, array) in &self.scene.cylinder_components {
                    frame.draw(draw!(program, array, &[], {
                        mv: v,
                        p: p,
                        albedo: color_of(generator),
                        t: t,
                    }));
                }
            }
        }

        // Render surfaces to GBuffer and cylindrify anything in the cylinder buffer
        {
            let mut frame = Frame::new(&mut self.ctx)
                .with_frame_buffer(&self.gbuffer.framebuffer)
                .with_clear_color(Vec4::new(0., 0., 0., 0.));

            if !*settings.get_mesh_hidden() {
                for (generator, array) in &self.scene.components {
                    frame.draw(draw!(program, array, &[], {
                        mv: v,
                        p: p,
                        albedo: color_of(generator),
                        t: t,
                    }));
                }

                if self.scene.view_dimension == ViewDimension::Four {
                    for animation_curve in &self.scene.animation_curves {
                        if let (Some(position), Some(sphere)) =
                            (animation_curve.at(t), self.scene.sphere.as_ref())
                        {
                            frame.draw(draw!(&self.shaders.geometry_3d, sphere, &[], {
                                mv: v * Mat4::from_translation(position.xyz()) * Mat4::from_scale(geometry_scale),
                                p: p,
                                albedo: color_of(&animation_curve.generator),
                                t: t,
                            }));
                        }
                    }

                    if *settings.get_animate_singularities() {
                        let radius = *settings.get_singularity_duration() as f32 / 10.;

                        for (generator, point) in &self.scene.animation_singularities {
                            let dt = duration * (point.w - t).abs();
                            if dt > radius {
                                continue;
                            }

                            if let Some(sphere) = self.scene.sphere.as_ref() {
                                let scale = geometry_scale * 1.4 * f32::sqrt(1. - dt / radius);
                                frame.draw(draw!(&self.shaders.geometry_3d, sphere, &[], {
                                mv: v * Mat4::from_translation(point.xyz()) * Mat4::from_scale(scale),
                                p: p,
                                albedo: color_of(generator),
                                t: t,
                            }));
                            }
                        }
                    }

                    frame.draw(draw! {
                        &self.shaders.cylinder_pass,
                        &self.quad.array,
                        &[
                            &self.cylinder_buffer.positions,
                            &self.cylinder_buffer.albedo,
                        ],
                        {
                            in_position: 0,
                            in_albedo: 1,
                            p: p,
                        }
                    });
                }
            }
        }

        // Final pass
        {
            // Apply lighting to scene
            let mut frame = Frame::new(&mut self.ctx).with_clear_color(Vec4::broadcast(1.));
            frame.draw(draw! {
                &self.shaders.lighting_pass,
                &self.quad.array,
                &[
                    &self.gbuffer.positions,
                    &self.gbuffer.normals,
                    &self.gbuffer.albedo,
                ],
                {
                    g_position: 0,
                    g_normal: 1,
                    g_albedo: 2,
                    disable_lighting: *settings.get_disable_lighting(),
                    debug_normals: *settings.get_debug_normals(),
                    spec: 1e-2 * *settings.get_specularity() as f32,
                    alpha: *settings.get_shininess() as f32,
                    gamma: 0.1 * *settings.get_gamma() as f32,
                    camera_pos: camera.position(),
                }
            });

            // Add in relevant wireframes
            if *settings.get_wireframe_3d() {
                for array in &self.scene.wireframe_components {
                    frame.draw(draw! {
                        &self.shaders.wireframe,
                        array,
                        &[],
                        DepthTest::Disable,
                        {
                            mv: v,
                            p: p,
                        }
                    });
                }
            }

            // Render axes
            if *settings.get_debug_axes() {
                frame.draw(draw! {
                    &self.shaders.wireframe,
                    &self.axes.array,
                    &[],
                    DepthTest::Disable,
                    {
                        mv: v,
                        p: p,
                    }
                });
            }
        }
    }
}
