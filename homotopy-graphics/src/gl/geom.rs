use euclid::{Transform3D, Vector3D};

use super::GlCtx;

pub type Coord = f32;

pub struct RgbSpace;
pub struct ModelSpace;
pub struct WorldSpace;
pub struct ViewSpace;
pub struct ScreenSpace;

pub type Color = Vector3D<Coord, RgbSpace>;
pub type Vertex = Vector3D<Coord, ModelSpace>;

pub type ModelMatrix = Transform3D<Coord, ModelSpace, WorldSpace>;
pub type ViewMatrix = Transform3D<Coord, WorldSpace, ViewSpace>;
pub type ProjectionMatrix = Transform3D<Coord, ViewSpace, ScreenSpace>;

pub type MVPMatrix = Transform3D<Coord, ModelSpace, ScreenSpace>;

#[inline]
pub fn perspective(fov: f32, width: u32, height: u32, near: f32, far: f32) -> ProjectionMatrix {
    let aspect_ratio = width as f32 / height as f32;
    let tan_half_fov = f32::tan(f32::to_radians(fov / 2.0));
    let range = near - far;

    ProjectionMatrix::new(
        //
        1.0 / (tan_half_fov * aspect_ratio),
        0.0,
        0.0,
        0.0,
        //
        0.0,
        1.0 / tan_half_fov,
        0.0,
        0.0,
        //
        0.0,
        0.0,
        (far + near) / range,
        (2.0 * far * near) / range,
        //
        0.0,
        0.0,
        -1.0,
        0.0,
    )
}

impl GlCtx {
    #[inline]
    pub fn perspective(&self, fov: f32, near: f32, far: f32) -> ProjectionMatrix {
        perspective(fov, self.width, self.height, near, far)
    }
}
