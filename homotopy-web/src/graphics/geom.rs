use euclid::{Transform3D, Vector3D};

pub type Coord = f32;

pub struct ModelSpace;
pub struct WorldSpace;
pub struct ScreenSpace;

pub type Vertex = Vector3D<Coord, ModelSpace>;

pub type ViewMatrix = Transform3D<Coord, ModelSpace, WorldSpace>;
pub type ProjectionMatrix = Transform3D<Coord, WorldSpace, ScreenSpace>;
