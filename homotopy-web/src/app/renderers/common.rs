use std::f32::consts::PI;

use homotopy_core::DiagramN;
use homotopy_graphics::{
    draw,
    gl::{array::VertexArray, buffer::ElementKind, frame::Frame, shader::Program, GlCtx, Result},
    program, vertex_array,
};
use ultraviolet::{
    projection::rh_yup::{orthographic_gl, perspective_gl},
    Mat4, Vec2, Vec3,
};
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct DiagramProps {
    pub diagram: DiagramN,
}

pub struct DebugCtx {
    program: Program,
    axes: VertexArray,
}

impl DebugCtx {
    pub fn new(ctx: &mut GlCtx) -> Result<Self> {
        let program = program!(
            ctx,
            "../../../glsl/wireframe_vert.glsl",
            "../../../glsl/wireframe_frag.glsl",
            { position, color },
            { mvp },
        )?;

        let axes_elements = ctx.mk_element_buffer(&[0, 1, 2, 3, 4, 5], ElementKind::Lines)?;
        let axes_verts = ctx.mk_buffer(&[
            Vec3::zero(),
            Vec3::unit_x(),
            Vec3::zero(),
            Vec3::unit_y(),
            Vec3::zero(),
            Vec3::unit_z(),
        ])?;
        let axes_colors = ctx.mk_buffer(&[
            Vec3::unit_x(),
            Vec3::unit_x(),
            Vec3::unit_y(),
            Vec3::unit_y(),
            Vec3::unit_z(),
            Vec3::unit_z(),
        ])?;

        Ok(Self {
            axes: vertex_array!(
                &program,
                &axes_elements,
                {
                    position: &axes_verts,
                    color: &axes_colors,
                }
            )?,
            program,
        })
    }

    pub fn render_axes<'a>(&'a self, frame: &mut Frame<'a>, transform: Mat4) {
        frame.draw(draw! {
            &self.axes,
            { mvp: transform }
        });
    }

    pub fn wireframe_program(&self) -> &Program {
        &self.program
    }
}

pub struct OrbitCamera {
    target: Vec3,
    phi: f32,
    theta: f32,
    distance: f32,
    fov: f32,
    ortho: bool,
}

impl OrbitCamera {
    const DEFAULT_DISTANCE: f32 = 12.;
    const DEFAULT_PHI: f32 = 0.5 * PI;
    const DEFAULT_THETA: f32 = 0.5 * PI;
    const EPSILON: f32 = 0.05;
    const FAR: f32 = 100.;
    const NEAR: f32 = 0.01;

    pub fn new(target: Vec3, fov: f32) -> Self {
        Self {
            target,
            phi: Self::DEFAULT_PHI,
            theta: Self::DEFAULT_THETA,
            distance: Self::DEFAULT_DISTANCE,
            fov,
            ortho: false,
        }
    }

    pub fn position(&self) -> Vec3 {
        let sin_phi = f32::sin(self.phi);
        let cos_phi = f32::cos(self.phi);
        let sin_theta = f32::sin(self.theta);
        let cos_theta = f32::cos(self.theta);

        self.distance * Vec3::new(cos_phi * sin_theta, -cos_theta, -sin_phi * sin_theta)
            + self.target
    }

    pub fn transform(&self, ctx: &GlCtx) -> Mat4 {
        let perspective = if self.ortho {
            orthographic_gl(
                -ctx.aspect_ratio(),
                ctx.aspect_ratio(),
                -1.,
                1.,
                Self::NEAR,
                Self::FAR,
            )
        } else {
            perspective_gl(
                f32::to_radians(self.fov),
                ctx.aspect_ratio(),
                Self::NEAR,
                Self::FAR,
            )
        };
        let view = Mat4::look_at(self.position(), self.target, Vec3::unit_y());

        perspective * view
    }

    pub fn set_ortho(&mut self, ortho: bool) {
        self.ortho = ortho;
    }

    pub fn apply_angle_delta(&mut self, delta: Vec2) {
        self.phi -= delta.x;
        self.theta = (self.theta + delta.y).clamp(Self::EPSILON, PI - Self::EPSILON);
    }

    pub fn apply_distance_delta(&mut self, delta: f32) {
        self.distance *= if delta > 0. { 1.1 } else { 1.0 / 1.1 };
    }
}
