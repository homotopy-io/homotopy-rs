use std::f32::consts::PI;

use homotopy_graphics::gl::GlCtx;
use ultraviolet::{
    projection::rh_yup::{orthographic_gl, perspective_gl},
    Mat4, Vec2, Vec3,
};

use crate::components::{
    delta::Delta,
    touch_interface::{TouchAction, TouchInterface},
    Finger, Point,
};

pub struct OrbitCamera {
    pub phi: f32,
    pub theta: f32,
    pub distance: f32,
    target: Vec3,
    fov: f32,
    ortho: bool,
    mouse: Option<Vec2>,
}

impl OrbitCamera {
    const DEFAULT_DISTANCE: f32 = 12.;
    const DEFAULT_FOV: f32 = 30.0;
    const DEFAULT_PHI: f32 = 0.5 * PI;
    const DEFAULT_THETA: f32 = 0.5 * PI;
    const EPSILON: f32 = 0.05;
    const FAR: f32 = 1000.;
    const NEAR: f32 = 0.01;

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
            let scale = self.distance / 10.;
            let aspect = ctx.aspect_ratio();
            orthographic_gl(
                -aspect * scale,
                aspect * scale,
                -scale,
                scale,
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

    pub fn apply_angle_delta(&mut self, delta: Vec2) {
        self.phi -= delta.x;
        self.theta = (self.theta + delta.y).clamp(Self::EPSILON, PI - Self::EPSILON);
    }

    pub fn apply_distance_delta(&mut self, delta: f32) {
        self.distance *= if delta > 0. { 1.1 } else { 1.0 / 1.1 };
    }

    pub fn set_ortho(&mut self, ortho: bool) {
        self.ortho = ortho;
    }
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: Default::default(),
            phi: Self::DEFAULT_PHI,
            theta: Self::DEFAULT_THETA,
            distance: Self::DEFAULT_DISTANCE,
            fov: Self::DEFAULT_FOV,
            ortho: false,
            mouse: None,
        }
    }
}

impl TouchInterface for OrbitCamera {
    fn mouse_down(&mut self, point: Point) {
        self.mouse = Some(Vec2::new(point.x as f32, point.y as f32));
    }

    fn mouse_up(&mut self) {
        self.mouse = None;
    }

    fn mouse_move(&mut self, next: Point) {
        let next = Vec2::new(next.x as f32, next.y as f32);
        if let Some(prev) = self.mouse {
            let delta = 4. * (next - prev) / 1000.;
            // TODO(@doctorn) divide by `self.gl_ctx.size()`
            self.apply_angle_delta(delta);
            self.mouse = Some(next);
        }
    }

    fn mouse_wheel(&mut self, _: Point, delta: f64) {
        self.apply_distance_delta(delta as f32);
    }

    fn touch_move(&mut self, _touches: &[(Finger, Point)]) {
        // TODO(@doctorn) touch contorls
    }

    fn touch_update(&mut self, _touches: &[(Finger, Point)]) {
        // TODO(@doctorn) touch contorls
    }

    fn reset(&mut self) {
        *self = Default::default();
    }
}

pub struct OrbitControl(Delta<OrbitCamera>);

impl OrbitControl {
    pub fn new() -> Self {
        Self(Delta::new())
    }

    pub fn zoom_in(&self) {
        self.0
            .emit(TouchAction::MouseWheel(Default::default(), -20.0));
    }

    pub fn zoom_out(&self) {
        self.0
            .emit(TouchAction::MouseWheel(Default::default(), 20.0));
    }

    pub fn reset(&self) {
        self.0.emit(TouchAction::Reset);
    }
}
