use homotopy_graphics::{
    draw,
    gl::{
        frame::{DepthTest, Frame},
        GlCtx, Result,
    },
};
use ultraviolet::{Vec3, Vec4};

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
        let vp = camera.transform(&self.ctx);

        let program = if self.scene.view_dimension == ViewDimension::Three {
            &self.shaders.geometry_3d
        } else {
            &self.shaders.geometry_4d
        };

        // Render animated wireframes to cylinder buffer
        if self.scene.view_dimension == ViewDimension::Four {
            let mut frame = Frame::new(&mut self.ctx)
                .with_frame_buffer(&self.cylinder_buffer.framebuffer)
                .with_clear_color(Vec4::new(0., 0., 0., 0.));

            if !*settings.get_mesh_hidden() {
                let signature = &self.signature;

                for (generator, array) in &self.scene.cylinder_components {
                    let color = signature.generator_info(*generator).map_or(
                        palette::rgb::Rgb {
                            red: 0.,
                            green: 0.,
                            blue: 1.,
                            ..Default::default()
                        },
                        |info| info.color.0.into_format(),
                    );
                    frame.draw(draw!(program, array, &[], {
                        mvp: vp,
                        albedo: Vec3::new(color.red, color.green, color.blue),
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
                let signature = &self.signature;

                for (generator, array) in &self.scene.components {
                    let color = signature.generator_info(*generator).map_or(
                        palette::rgb::Rgb {
                            red: 0.,
                            green: 0.,
                            blue: 1.,
                            ..Default::default()
                        },
                        |info| info.color.0.into_format(),
                    );
                    frame.draw(draw!(program, array, &[], {
                        mvp: vp,
                        albedo: Vec3::new(color.red, color.green, color.blue),
                        t: t,
                    }));
                }

                if self.scene.view_dimension == ViewDimension::Four {
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
                    camera_pos: camera.position(),
                    disable_lighting: *settings.get_disable_lighting(),
                    debug_normals: *settings.get_debug_normals(),
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
                        { mvp: vp }
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
                    { mvp: vp }
                });
            }
        }
    }
}
