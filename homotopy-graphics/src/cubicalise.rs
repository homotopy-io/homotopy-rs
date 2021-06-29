use homotopy_core::DiagramN;

use crate::mesh::{Mesh, Vertex};

/// Turn a diagram into its surface cubical representation.
pub fn cubicalise(diagram: &DiagramN) -> Mesh {
    let mut mesh = Mesh::new();

    // The code for cubicalisation goes here, in the meantime generate dummy data
    let (coords, bounds) = match diagram.dimension() {
        3 => {
            // an octahedron that makes the last frame of the birth sphere homotopy
            const V_COORDS: [[i32; 4]; 6] = [
                [0, 0, 0, -1],
                [0, 0, -1, 0],
                [0, -1, 0, 0],
                [0, 0, 0, 1],
                [0, 0, 1, 0],
                [0, 1, 0, 0],
            ];
            const V_BOUNDS: [u8; 6] = [2, 2, 2, 2, 2, 2];

            (&V_COORDS[..], &V_BOUNDS[..])
        }
        4 => {
            const V_COORDS: [[i32; 4]; 13] = [
                [1, 0, 0, -1],
                [1, 0, -1, 0],
                [1, -1, 0, 0],
                [0, 0, 0, 0],
                [1, 0, 0, 1],
                [1, 0, 1, 0],
                [1, 1, 0, 0],
                [2, 0, 0, -1],
                [2, 0, -1, 0],
                [2, -1, 0, 0],
                [2, 0, 0, 1],
                [2, 0, 1, 0],
                [2, 1, 0, 0],
            ];
            const V_BOUNDS: [u8; 13] = [3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2];

            (&V_COORDS[..], &V_BOUNDS[..])
        }
        _ => panic!(),
    };

    let verts: Vec<_> = bounds
        .iter()
        .copied()
        .zip(coords.iter())
        .map(|(bound, coord)| {
            let v = Vertex::new(
                f64::from(coord[3]),
                f64::from(coord[2]),
                f64::from(coord[1]),
                f64::from(coord[0]),
                bound,
            );
            mesh.mk_vertex(v)
        })
        .collect();

    match diagram.dimension() {
        3 => {
            const SQUARES: [[usize; 4]; 8] = [
                [0, 1, 2, 2],
                [3, 1, 2, 2],
                [0, 4, 2, 2],
                [3, 4, 2, 2],
                [0, 1, 5, 5],
                [3, 1, 5, 5],
                [0, 4, 5, 5],
                [3, 4, 5, 5],
            ];

            for square in &SQUARES {
                let square: Vec<_> = square.iter().map(|index| verts[*index]).collect();
                mesh.mk_element(2, &square);
            }
        }
        4 => {
            const SQUARES: [[usize; 8]; 16] = [
                [0, 1, 2, 2, 3, 3, 3, 3],
                [4, 1, 2, 2, 3, 3, 3, 3],
                [0, 5, 2, 2, 3, 3, 3, 3],
                [4, 5, 2, 2, 3, 3, 3, 3],
                [0, 1, 6, 6, 3, 3, 3, 3],
                [4, 1, 6, 6, 3, 3, 3, 3],
                [0, 5, 6, 6, 3, 3, 3, 3],
                [4, 5, 6, 6, 3, 3, 3, 3],
                [0, 7, 1, 8, 2, 9, 2, 9],
                [4, 10, 1, 8, 2, 9, 2, 9],
                [0, 7, 5, 11, 2, 9, 2, 9],
                [4, 10, 5, 11, 2, 9, 2, 9],
                [0, 7, 1, 8, 6, 12, 6, 12],
                [4, 10, 1, 8, 6, 12, 6, 12],
                [0, 7, 5, 11, 6, 12, 6, 12],
                [4, 10, 5, 11, 6, 12, 6, 12],
            ];

            for square in &SQUARES {
                let square: Vec<_> = square.iter().map(|index| verts[*index]).collect();
                mesh.mk_element(3, &square);
            }
        }
        _ => panic!(),
    }
    mesh
}
