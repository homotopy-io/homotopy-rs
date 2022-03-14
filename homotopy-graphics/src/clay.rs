//! C(ubical) Lay(out) Engine

use homotopy_core::{common::DimensionError, DiagramN};

use crate::geom::{CubicalGeometry, SimplicialGeometry};

mod buffers;
mod subdivision;

pub fn clay(
    diagram: &DiagramN,
    view_dimension: usize,
    smooth_time: bool,
    subdivision_depth: u8,
) -> Result<SimplicialGeometry, DimensionError> {
    let mut geom = match view_dimension {
        3 => CubicalGeometry::new::<3>(diagram)?,
        4 => CubicalGeometry::new::<4>(diagram)?,
        _ => return Err(DimensionError),
    };
    geom.subdivide(smooth_time, subdivision_depth);
    Ok(geom.into())
}
