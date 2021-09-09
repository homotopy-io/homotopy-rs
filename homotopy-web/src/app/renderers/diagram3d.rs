use std::f32;

use homotopy_graphics::{
    clay::{examples::snake_3, subdivision},
    draw,
    gl::{array::VertexArray, buffer::ElementKind, frame::Frame, GlCtx, Result},
    program, vertex_array,
};
use ultraviolet::{
    projection::rh_yup::{orthographic_gl, perspective_gl},
    Mat4, Vec2, Vec3,
};

use crate::{
    app::AppSettings,
    components::{
        gl::{Renderer, RendererState},
        settings::Store,
    },
};

pub struct Diagram3D {
    solid_mesh: VertexArray,
    wireframe_mesh: VertexArray,
    camera: Vec3,
    mouse: Option<Vec2>,
}

impl Renderer for Diagram3D {
    type Settings = AppSettings;

    fn init(ctx: &mut GlCtx) -> Result<Self> {
        let subdivided = subdivision::subdivide_3(snake_3().into(), 4);
        let solid_buffers = subdivided.buffer(ctx, ElementKind::Triangles)?;
        let wireframe_buffers = subdivided.buffer(ctx, ElementKind::Lines)?;
        let program = program!(
            ctx,
            "../../../glsl/vert.glsl",
            "../../../glsl/frag.glsl",
            { position, normal },
            { mvp, lighting, light_pos },
        )?;
        let solid_mesh = vertex_array!(
            &program,
            &solid_buffers.element_buffer,
            {
                position: &solid_buffers.vertex_buffer,
                normal: &solid_buffers.normal_buffer,
            }
        )?;
        let wireframe_mesh = vertex_array!(
            &program,
            &wireframe_buffers.element_buffer,
            {
                position: &wireframe_buffers.vertex_buffer,
                normal: &wireframe_buffers.normal_buffer,
            }
        )?;

        Ok(Self {
            solid_mesh,
            wireframe_mesh,
            camera: Vec3::new(0., 0.5 * f32::consts::PI, 12.),
            mouse: None,
        })
    }

    fn update(_this: &mut RendererState<Self>, _: f32) {}

    fn render<'a>(&'a self, mut frame: Frame<'a>, settings: &Store<Self::Settings>) {
        let vp = self.vp_matrix(frame.aspect_ratio(), *settings.get_orthographic_3d());

        frame.draw(draw! {
            if *settings.get_wireframe_3d() {
                &self.wireframe_mesh
            } else {
                &self.solid_mesh
            },
            {
                mvp: vp,
                lighting: !*settings.get_lighting_disable(),
                light_pos: self.camera_position() + Vec3::broadcast(1.),
            }
        });
    }

    fn on_mouse_down(this: &mut RendererState<Self>, point: Vec2) {
        this.mouse = Some(point);
    }

    fn on_mouse_up(this: &mut RendererState<Self>) {
        this.mouse = None;
    }

    fn on_mouse_move(this: &mut RendererState<Self>, next: Vec2) {
        if let Some(prev) = this.mouse {
            let delta = (next - prev) / this.ctx().size();
            this.camera.x -= delta.x;
            this.camera.y = (this.camera.y + delta.y).clamp(0.05, f32::consts::PI - 0.05);
            this.mouse = Some(next);
        }
    }

    fn on_mouse_wheel(this: &mut RendererState<Self>, _: Vec2, delta: f32) {
        this.camera.z *= if delta > 0.0 { 1.1 } else { 1.0 / 1.1 };
    }
}

impl Diagram3D {
    fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.camera_position(), Vec3::zero(), Vec3::unit_y())
    }

    fn projection_matrix(aspect_ratio: f32, ortho: bool) -> Mat4 {
        if ortho {
            orthographic_gl(-aspect_ratio, aspect_ratio, -1., 1., 0.01, 20.0)
        } else {
            perspective_gl(f32::to_radians(30.0), aspect_ratio, 0.01, 20.0)
        }
    }

    fn vp_matrix(&self, aspect_ratio: f32, ortho: bool) -> Mat4 {
        Self::projection_matrix(aspect_ratio, ortho) * self.view_matrix()
    }

    fn camera_position(&self) -> Vec3 {
        let phi = self.camera.x;
        let theta = self.camera.y;
        let distance = self.camera.z;

        let sin_phi = f32::sin(phi);
        let cos_phi = f32::cos(phi);
        let sin_theta = f32::sin(theta);
        let cos_theta = f32::cos(theta);

        distance * Vec3::new(cos_phi * sin_theta, -cos_theta, -sin_phi * sin_theta)
    }
}
