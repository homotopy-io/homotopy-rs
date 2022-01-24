//! C(ubical) Lay(out) Engine

use homotopy_core::{common::DimensionError, DiagramN};

use crate::geom::{CubicalGeometry, SimplicialGeometry};

mod buffers;
mod subdivision;

pub fn clay(
    diagram: &DiagramN,
    view_dimension: usize,
    subdivision_depth: u8,
) -> Result<SimplicialGeometry, DimensionError> {
    let mut geom = CubicalGeometry::new(diagram, view_dimension)?;
    geom.subdivide(subdivision_depth);
    Ok(geom.into())
}
