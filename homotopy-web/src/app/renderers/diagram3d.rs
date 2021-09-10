use std::f32;

use homotopy_graphics::{
    clay::{examples::example_3, subdivision},
    draw,
    gl::{array::VertexArray, frame::Frame, shader::Program, GlCtx, Result},
    program, vertex_array,
};
use ultraviolet::{Vec2, Vec3};

use super::common::DebugCtx;
use crate::{
    app::{renderers::common::OrbitCamera, AppSettings},
    components::{
        gl::{Renderer, RendererState},
        settings::Store,
    },
};

pub struct Diagram3D {
    debug_ctx: DebugCtx,
    program: Program,
    camera: OrbitCamera,

    solid_mesh: Option<VertexArray>,
    wireframe_mesh: Option<VertexArray>,

    subdivision_depth: u32,
    mouse: Option<Vec2>,
}

impl Renderer for Diagram3D {
    type Settings = AppSettings;

    fn init(ctx: &mut GlCtx, settings: &Store<Self::Settings>) -> Result<Self> {
        let program = program!(
            ctx,
            "../../../glsl/vert.glsl",
            "../../../glsl/frag.glsl",
            { position, normal },
            { mvp, debug_normals, light_pos },
        )?;

        let mut renderer = Self {
            debug_ctx: DebugCtx::new(ctx)?,
            camera: OrbitCamera::new(Vec3::zero(), 30.0),
            program,
            solid_mesh: None,
            wireframe_mesh: None,
            subdivision_depth: *settings.get_subdivision_depth(),
            mouse: None,
        };

        renderer.init_meshes(ctx)?;

        Ok(renderer)
    }

    fn update(this: &mut RendererState<Self>, _: f32) -> Result<()> {
        let depth = *this.settings().get_subdivision_depth();

        if this.subdivision_depth != depth {
            this.subdivision_depth = depth;
            this.with_ctx(|this, ctx| this.init_meshes(ctx))?;
        }

        let ortho = *this.settings().get_orthographic_3d();
        this.camera.set_ortho(ortho);

        Ok(())
    }

    fn render<'a>(&'a self, mut frame: Frame<'a>, settings: &Store<Self::Settings>) {
        let vp = self.camera.transform(&*frame);

        if !*settings.get_mesh_hidden() {
            frame.draw(draw! {
                self.solid_mesh.as_ref().unwrap(),
                {
                    mvp: vp,
                    debug_normals: *settings.get_debug_normals(),
                    light_pos: self.camera.position() + Vec3::new(0.5, 1., -1.),
                }
            });
        }

        if *settings.get_wireframe_3d() {
            frame.draw(draw! {
                self.wireframe_mesh.as_ref().unwrap(),
                { mvp: vp }
            });
        }

        if *settings.get_debug_axes() {
            self.debug_ctx.render_axes(&mut frame, vp);
        }
    }

    fn on_mouse_down(this: &mut RendererState<Self>, point: Vec2) {
        this.mouse = Some(point);
    }

    fn on_mouse_up(this: &mut RendererState<Self>) {
        this.mouse = None;
    }

    fn on_mouse_move(this: &mut RendererState<Self>, next: Vec2) {
        if let Some(prev) = this.mouse {
            let delta = 4. * (next - prev) / this.ctx().size();
            this.camera.apply_angle_delta(delta);
            this.mouse = Some(next);
        }
    }

    fn on_mouse_wheel(this: &mut RendererState<Self>, _: Vec2, delta: f32) {
        this.camera.apply_distance_delta(delta);
    }
}

impl Diagram3D {
    fn init_meshes(&mut self, ctx: &mut GlCtx) -> Result<()> {
        let subdivided = subdivision::subdivide_3(example_3().into(), self.subdivision_depth as u8);
        let buffers = subdivided.buffer(ctx)?;

        self.solid_mesh = Some(vertex_array!(
            &self.program,
            &buffers.element_buffer,
            {
                position: &buffers.vertex_buffer,
                normal: &buffers.normal_buffer,
            }
        )?);

        self.wireframe_mesh = Some(vertex_array!(
            self.debug_ctx.wireframe_program(),
            &buffers.wireframe_element_buffer,
            { position: &buffers.vertex_buffer }
        )?);

        Ok(())
    }
}
