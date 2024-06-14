use std::fmt::Write;

use homotopy_core::{common::DimensionError, Diagram};
use serde::Serialize;

use crate::{
    geom::{CubicalGeometry, SimplicialGeometry},
    style::SignatureStyleData,
};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize)]
pub struct StlOptions {
    pub geometry_samples: u8,
    pub subdivision_depth: u8,
}

pub fn render(
    diagram: &Diagram,
    signature_styles: &impl SignatureStyleData,
    options: StlOptions,
) -> Result<String, DimensionError> {
    let mut output = String::new();

    let mut cubical = CubicalGeometry::new::<3>(diagram, false)?;
    cubical.subdivide(false, options.subdivision_depth);

    let mut simplicial = SimplicialGeometry::from(cubical);
    simplicial.inflate_3d(options.geometry_samples, signature_styles);

    writeln!(output, "solid assoc").unwrap();

    for ([i, j, k], parity) in simplicial.areas.values().copied() {
        let v_1 = simplicial.verts[i].position.xyz();
        let v_2 = simplicial.verts[j].position.xyz();
        let v_3 = simplicial.verts[k].position.xyz();

        let sign = if parity.is_even() { 1. } else { -1. };
        let n = sign * (v_2 - v_1).cross(v_3 - v_2);

        writeln!(output, "facet normal {} {} {}", n.x, n.y, n.z).unwrap();
        writeln!(output, "outer loop").unwrap();

        for v in [v_1, v_2, v_3] {
            writeln!(output, "vertex {} {} {}", v.x, v.y, v.z).unwrap();
        }

        writeln!(output, "endloop").unwrap();
        writeln!(output, "endfacet").unwrap();
    }

    writeln!(output, "endsolid assoc").unwrap();

    Ok(output)
}
