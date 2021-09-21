use super::geom::Mesh;

mod cubes;
mod engine;
mod lines;
mod squares;

pub use cubes::CubeSubdivider;
pub use lines::LineSubdivider;
pub use squares::SquareSubdivider;

pub fn subdivide<T>(mesh: Mesh, depth: u8) -> Mesh<T::Primitive>
where
    T: engine::Subdivider,
{
    let mut mesh = mesh.into();
    let mut engine = engine::SubdivisionEngine::<T>::new(&mut mesh);

    for _ in 0..depth {
        engine.subdivide_once();
    }

    mesh
}
