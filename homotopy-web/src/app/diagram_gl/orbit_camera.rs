use std::f32::consts::PI;

use homotopy_graphics::gl::GlCtx;
use ultraviolet::{
    projection::rh_yup::{orthographic_gl, perspective_gl},
    Mat4, Vec2, Vec3,
};

use crate::components::{touch_interface::TouchInterface, Finger, Point};

pub struct OrbitCamera {
    pub phi: f32,
    pub theta: f32,
    pub distance: f32,
    pub target: Vec3,
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
        self.distance * self.offset() + self.target
    }

    pub fn view_transform(&self, _ctx: &GlCtx) -> Mat4 {
        Mat4::look_at(self.position(), self.target, Vec3::unit_y())
    }

    pub fn perspective_transform(&self, ctx: &GlCtx) -> Mat4 {
        if self.ortho {
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
        }
    }

    pub fn apply_angle_delta(&mut self, delta: Vec2) {
        self.phi -= delta.x;
        self.theta = (self.theta + delta.y).clamp(Self::EPSILON, PI - Self::EPSILON);
    }

    pub fn apply_distance_delta(&mut self, delta: f32) {
        self.distance *= if delta > 0. { 1.1 } else { 1.0 / 1.1 };
    }

    pub fn apply_target_delta(&mut self, delta: Vec2) {
        let z = -self.offset();
        let up = Vec3::unit_y();
        let x = up.cross(z).normalized();
        let y = z.cross(x).normalized();

        self.target += delta.x * x + delta.y * y;
    }

    pub fn set_ortho(&mut self, ortho: bool) {
        self.ortho = ortho;
    }

    fn offset(&self) -> Vec3 {
        let sin_phi = f32::sin(self.phi);
        let cos_phi = f32::cos(self.phi);
        let sin_theta = f32::sin(self.theta);
        let cos_theta = f32::cos(self.theta);

        Vec3::new(cos_phi * sin_theta, -cos_theta, -sin_phi * sin_theta)
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
    fn mouse_down(&mut self, _alt_key: bool, point: Point) {
        self.mouse = Some(Vec2::new(point.x as f32, point.y as f32));
    }

    fn mouse_up(&mut self) {
        self.mouse = None;
    }

    fn mouse_move(&mut self, alt_key: bool, next: Point) {
        let next = Vec2::new(next.x as f32, next.y as f32);
        if let Some(prev) = self.mouse {
            let delta = next - prev;
            if alt_key {
                self.apply_target_delta(1.5e-2 * delta);
            } else {
                // TODO(@doctorn) divide by `self.gl_ctx.size()`
                self.apply_angle_delta(4e-3 * delta);
            }
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
