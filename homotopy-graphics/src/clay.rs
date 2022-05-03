//! C(ubical) Lay(out) Engine

use homotopy_core::{common::DimensionError, Diagram};

use crate::geom::{CubicalGeometry, SimplicialGeometry};

mod buffers;
mod subdivision;

pub fn clay(
    diagram: &Diagram,
    view_dimension: u8,
    smooth_time: bool,
    subdivision_depth: u8,
) -> Result<SimplicialGeometry, DimensionError> {
    let mut geom = match view_dimension {
        0 => CubicalGeometry::new::<0>(diagram)?,
        1 => CubicalGeometry::new::<1>(diagram)?,
        2 => CubicalGeometry::new::<2>(diagram)?,
        3 => CubicalGeometry::new::<3>(diagram)?,
        4 => CubicalGeometry::new::<4>(diagram)?,
        _ => return Err(DimensionError),
    };
    geom.subdivide(smooth_time, subdivision_depth);
    Ok(geom.into())
}
