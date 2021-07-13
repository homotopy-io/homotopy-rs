#![allow(unused)]

use euclid::{Transform3D, Vector3D};

pub type Coord = f32;

pub struct ModelSpace;
pub struct WorldSpace;
pub struct ViewSpace;
pub struct ScreenSpace;

pub type Vertex = Vector3D<Coord, ModelSpace>;

pub type ModelMatrix = Transform3D<Coord, ModelSpace, WorldSpace>;
pub type ViewMatrix = Transform3D<Coord, WorldSpace, ViewSpace>;
pub type ProjectionMatrix = Transform3D<Coord, ViewSpace, ScreenSpace>;

pub type MVPMatrix = Transform3D<Coord, ModelSpace, ScreenSpace>;
