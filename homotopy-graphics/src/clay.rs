//! C(ubical) Lay(out) Engine

use homotopy_core::{common::DimensionError, DiagramN};

mod buffers;
mod geom;
mod layout;
mod subdivision;

pub fn clay(
    diagram: &DiagramN,
    view_dimension: usize,
    subdivision_depth: u8,
) -> Result<geom::simplicial::SimplicialMesh, DimensionError> {
    let mut mesh = layout::extract_mesh(diagram, view_dimension)?;
    mesh.subdivide(subdivision_depth);
    Ok(mesh.into())
}
