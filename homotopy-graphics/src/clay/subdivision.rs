//! Primal subdivision methods in this module follow a preset point order when subdividing elements.
//! For 2-cubes, 9 points are generated in the order (a). For 3-cubes 27 points are generated in order (b).
//! <img src="../../images/subdivision_illustration.png"/>

use std::collections::HashMap;
use std::{cmp, mem};

use ultraviolet::Vec4;

use homotopy_common::idx::IdxVec;

use super::geom::{SquareData, SquareMesh, Vertex, VertexExt};

// TODO(@doctorn) refactor
/// Generates a weight matrix for each surface element based on boundary information.
/// Also does basic mesh validation - panics if the boundaries are inconsistent.
fn create_square_matrix(bounds: [u8; 4]) -> [[f32; 4]; 4] {
    let mut matrix: [[f32; 4]; 4] = [[0.0; 4]; 4];
    // Vertex point
    matrix[0] = match bounds[0] {
        0 => [1.0, 0.0, 0.0, 0.0], // identity row
        1 => {
            match (bounds[1], bounds[2]) {
                (1, _) => [0.5, 0.5, 0.0, 0.0],
                (_, 1) => [0.5, 0.0, 0.5, 0.0],
                _ => panic!("Inconsistent mesh!"), // if it turns out such mesh is consistent, replace with the identity row
            }
        }
        2 => [0.25, 0.25, 0.25, 0.25],
        _ => panic!("Inconsistent mesh!"),
    };

    matrix[1] = match bounds[1] {
        1 => [0.0, 1.0, 0.0, 0.0],
        2 => [0.0, 0.5, 0.0, 0.5],
        _ => panic!("Inconsistent mesh!"),
    };

    matrix[2] = match bounds[2] {
        1 => [0.0, 0.0, 1.0, 0.0],
        2 => [0.0, 0.0, 0.5, 0.5],
        _ => panic!("Inconsistent mesh!"),
    };

    matrix[3] = [0.0, 0.0, 0.0, 1.0];

    matrix
}

// TODO(@doctorn) refactor and re-enable
/*
/// Debug check that signals that weights are reasonable
#[cfg(debug_assertions)]
fn check_bounds_preserved(
    original: &IdxVec<Vertex, VertexData>,
    subdivided: &IdxVec<Vertex, VertexData>,
) -> bool {
    let (max_x, min_x) = original
        .iter()
        .fold((f32::NEG_INFINITY, f32::INFINITY), |a, v| {
            (f32::max(a.0, v.1.x), f32::min(a.1, v.1.x))
        });
    let (max_y, min_y) = original
        .iter()
        .fold((f32::NEG_INFINITY, f32::INFINITY), |a, v| {
            (f32::max(a.0, v.1.y), f32::min(a.1, v.1.y))
        });
    let (max_z, min_z) = original
        .iter()
        .fold((f32::NEG_INFINITY, f32::INFINITY), |a, v| {
            (f32::max(a.0, v.1.z), f32::min(a.1, v.1.z))
        });
    let (max_t, min_t) = original
        .iter()
        .fold((f32::NEG_INFINITY, f32::INFINITY), |a, v| {
            (f32::max(a.0, v.1.w), f32::min(a.1, v.1.w))
        });

    subdivided.iter().fold(true, |a, v| {
        a && (v.1.x <= max_x && v.1.x >= min_x)
            && (v.1.y <= max_y && v.1.y >= min_y)
            && (v.1.z <= max_z && v.1.z >= min_z)
            && (v.1.w <= max_t && v.1.w >= min_t)
    })
}
*/

struct SquareSubdivider<'a> {
    mesh: &'a mut SquareMesh,
    division_memory: HashMap<(Vertex, Vertex), Vertex>,
    valence: HashMap<Vertex, i32>,
}

impl<'a> SquareSubdivider<'a> {
    /// Defines how new squares should be from linearly divided points.
    ///
    /// Important property that is preserved - first point is a vertex point, and similar for edge points.
    const SQUARE_ASSEMBLY_ORDER: [[usize; 4]; 4] =
        [[0, 4, 5, 8], [1, 6, 4, 8], [2, 5, 7, 8], [3, 7, 6, 8]];

    fn new(mesh: &'a mut SquareMesh) -> Self {
        Self {
            mesh,
            division_memory: Default::default(),
            valence: Default::default(),
        }
    }

    fn divide_uncached(&mut self, v_1: Vertex, v_2: Vertex) -> Vertex {
        let v_1 = &self.mesh.vertices[v_1];
        let v_2 = &self.mesh.vertices[v_2];
        let v = 0.5 * (**v_1 + **v_2);
        let boundary = cmp::min(2, cmp::max(v_1.boundary, v_2.boundary));

        self.mesh.mk_vertex(v.with_boundary(boundary))
    }

    fn divide(&mut self, mut v_1: Vertex, mut v_2: Vertex) -> Vertex {
        if v_1 == v_2 {
            return v_1;
        }

        if v_2 > v_1 {
            mem::swap(&mut v_1, &mut v_2);
        }

        self.division_memory
            .get(&(v_1, v_2))
            .copied()
            .unwrap_or_else(|| {
                let v = self.divide_uncached(v_1, v_2);
                self.division_memory.insert((v_1, v_2), v);
                v
            })
    }

    fn update_valence(&mut self, vertex: Vertex) {
        if let Some(valence) = self.valence.get_mut(&vertex) {
            *valence += 1;
            return;
        }

        self.valence.insert(vertex, 1);
    }

    fn subdivide_square(&mut self, square: SquareData) {
        let v_1 = self.divide(square[0], square[1]);
        let v_2 = self.divide(square[2], square[3]);
        let points = [
            square[0],
            square[1],
            square[2],
            square[3],
            // Edge midpoints
            v_1,
            self.divide(square[0], square[2]),
            self.divide(square[1], square[3]),
            v_2,
            // Centrepoint
            self.divide(v_1, v_2),
        ];

        for order in &Self::SQUARE_ASSEMBLY_ORDER {
            self.mesh.mk_square([
                points[order[0]],
                points[order[1]],
                points[order[2]],
                points[order[3]],
            ]);

            for v in order.iter() {
                self.update_valence(points[*v]);
            }
        }
    }

    /// Subdivides the square mesh one time based on the primal scheme.
    ///
    /// This assumes that the mesh is filtered so that only the relevant elements
    /// are present and all elments have some area.
    fn subdivide_once(&mut self) {
        // 1. Reset valence
        self.valence = HashMap::new();
        // 2. Remove all squares from mesh
        let mut squares = IdxVec::new();
        mem::swap(&mut self.mesh.elements, &mut squares);
        // 3. Subdivide each square linearly
        for square in squares.into_values() {
            self.subdivide_square(square);
        }
        // 4. Smooth
        let mut smoothed = IdxVec::with_capacity(self.mesh.vertices.len());
        // a) Populate with zero vertices
        for vertex in self.mesh.vertices.values() {
            smoothed.push(Vec4::zero().with_boundary(vertex.boundary));
        }
        // b) For each square
        for square in self.mesh.elements.values() {
            // gather the boundaries of its constituent vertices
            let bounds = [
                self.mesh.vertices[square[0]].boundary,
                self.mesh.vertices[square[1]].boundary,
                self.mesh.vertices[square[2]].boundary,
                self.mesh.vertices[square[3]].boundary,
            ];
            // and calculate a corresponding weight matrix
            let matrix = create_square_matrix(bounds);

            // TODO(@doctorn) refactor
            // Apply this matrix and update the smoothed position
            for (i, row) in matrix.iter().enumerate() {
                let v = &mut smoothed[square[i]];
                for (j, weight) in row.iter().enumerate() {
                    let w = &self.mesh.vertices[square[j]];
                    v.vertex += *weight * w.vertex;
                }
            }
        }

        // c) Update vertex positions and divide by valence
        for (vertex, data) in smoothed {
            let valence = self.valence[&vertex];
            self.mesh.vertices[vertex].vertex = *data / (valence as f32);
        }
    }
}

pub fn subdivide_3(mut mesh: SquareMesh, depth: u8) -> SquareMesh {
    let mut subdivider = SquareSubdivider::new(&mut mesh);

    for _ in 0..depth {
        subdivider.subdivide_once();
    }

    mesh
}

/*
// Defines all 12 edges of a cube based on vertex indices.
const CUBE_EDGE_ORDER: [[usize; 2]; 12] = [
    [0, 1],
    [0, 2],
    [0, 4],
    [1, 3],
    [1, 5],
    [2, 3],
    [2, 6],
    [3, 7],
    [4, 5],
    [4, 6],
    [5, 7],
    [6, 7],
];

// Defines all 6 faces of a cube based on vertex indices.
const CUBE_FACE_ORDER: [[usize; 4]; 6] = [
    [0, 1, 2, 3],
    [0, 1, 4, 5],
    [0, 2, 4, 6],
    [1, 3, 5, 7],
    [2, 3, 6, 7],
    [4, 5, 6, 7],
];

/// Defines how new cubes should be from linearly divided points.
/// See Picture 2.
/// Important property that is preserved - first point is a vertex point, edge and face points are also in precise positions.
const CUBE_ASSEMBLY_ORDER: [[usize; 8]; 8] = [
    [0, 8, 9, 20, 10, 21, 22, 26],
    [1, 11, 8, 20, 12, 23, 21, 26],
    [2, 9, 13, 20, 14, 22, 24, 26],
    [3, 13, 11, 20, 15, 24, 23, 26],
    [4, 16, 17, 25, 10, 21, 22, 26],
    [5, 18, 16, 25, 12, 23, 21, 26],
    [6, 17, 19, 25, 14, 22, 24, 26],
    [7, 19, 18, 25, 15, 24, 23, 26],
];

/// Generates a weight matrix for each volume element based on boundary information.
/// Also does basic mesh validation - panics if the boundaries are inconsistent.
fn create_cube_matrix(bounds: [u8; 8]) -> [[f64; 8]; 8] {
    let mut matrix: [[f64; 8]; 8] = [[0.0; 8]; 8];
    // Vertex point
    matrix[0] = match bounds[0] {
        0 => [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // identity row
        1 => {
            match (bounds[1], bounds[2], bounds[4]) {
                (1, _, _) => [0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                (_, 1, _) => [0.5, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
                (_, _, 1) => [0.5, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0],
                _ => panic!("Inconsistent mesh!"), // if it turns out such mesh is consistent, replace with the identity row
            }
        }
        2 => {
            match (bounds[3], bounds[5], bounds[6]) {
                (2, _, _) => [0.25, 0.25, 0.25, 0.25, 0.0, 0.0, 0.0, 0.0],
                (_, 2, _) => [0.25, 0.25, 0.0, 0.0, 0.25, 0.25, 0.0, 0.0],
                (_, _, 2) => [0.25, 0.0, 0.25, 0.0, 0.25, 0.0, 0.25, 0.0],
                _ => panic!("Inconsistent mesh!"), // if it turns out such mesh is consistent, replace with the identity row
            }
        }
        3 => [0.125, 0.125, 0.125, 0.125, 0.125, 0.125, 0.125, 0.125],
        _ => panic!("Inconsistent mesh!"),
    };
    // Edge points
    matrix[1] = match bounds[1] {
        1 => [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // identity row
        2 => {
            match (bounds[3], bounds[5]) {
                (2, _) => [0.0, 0.5, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0],
                (_, 2) => [0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0],
                _ => panic!("Inconsistent mesh!"), // if it turns out such mesh is consistent, replace with the identity row
            }
        }
        3 => [0.0, 0.25, 0.0, 0.25, 0.0, 0.25, 0.0, 0.25],
        _ => panic!("Inconsistent mesh!"),
    };
    matrix[2] = match bounds[2] {
        1 => [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0], // identity row
        2 => {
            match (bounds[3], bounds[6]) {
                (2, _) => [0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0],
                (_, 2) => [0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 0.0],
                _ => panic!("Inconsistent mesh!"), // if it turns out such mesh is consistent, replace with the identity row
            }
        }
        3 => [0.0, 0.0, 0.25, 0.25, 0.0, 0.0, 0.25, 0.25],
        _ => panic!("Inconsistent mesh!"),
    };
    matrix[4] = match bounds[4] {
        1 => [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0], // identity row
        2 => {
            match (bounds[5], bounds[6]) {
                (2, _) => [0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0],
                (_, 2) => [0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.5, 0.0],
                _ => panic!("Inconsistent mesh!"), // if it turns out such mesh is consistent, replace with the identity row
            }
        }
        3 => [0.0, 0.0, 0.0, 0.0, 0.25, 0.25, 0.25, 0.25],
        _ => panic!("Inconsistent mesh!"),
    };
    // Face points
    matrix[3] = match bounds[3] {
        2 => [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
        3 => [0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5],
        _ => panic!("Inconsistent mesh!"),
    };
    matrix[5] = match bounds[5] {
        2 => [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        3 => [0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.5],
        _ => panic!("Inconsistent mesh!"),
    };
    matrix[6] = match bounds[6] {
        2 => [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        3 => [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5],
        _ => panic!("Inconsistent mesh!"),
    };
    // Centroid
    matrix[7] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0];
    matrix
}

/// Subdivides cube mesh one time based on the primal scheme.
/// Assumption: the mesh is filtered so only the relevant elements are there and all elments have some volume.
pub fn subdivide4(control_mesh: &CubeMesh) -> CubeMesh {
    let mut new_mesh = CubeMesh::new();

    // Copy vertices as primal subdivision preserves them.
    let mut vert_map = HashMap::new();
    for v in control_mesh.vertices.iter() {
        vert_map.insert(v.0, new_mesh.mk_vertex(v.1.clone()));
    }

    let mut vert_valence = HashMap::new();
    // Linearly divide each cube
    for e in control_mesh.cubes.iter() {
        // Generate new points
        let mut points = [e.1[0]; 27]; //0-7 vertex, 8-19 edge, 20-25 face, 26 centre
        for (i, p) in points.iter_mut().enumerate().take(8) {
            *p = vert_map[&e.1[i]];
        }
        for (i, e) in CUBE_EDGE_ORDER.iter().enumerate() {
            let mut edge = Vec::new();
            for id in e {
                edge.push(points[*id]);
            }
            points[i + 8] = new_mesh.linearly_divide(edge);
        }
        for (i, f) in CUBE_FACE_ORDER.iter().enumerate() {
            let mut face = Vec::new();
            for id in f {
                face.push(points[*id]);
            }
            points[i + 20] = new_mesh.linearly_divide(face);
        }
        points[26] = new_mesh.linearly_divide(vec![points[20], points[25]]);

        // Assemble subcubes
        for order in &CUBE_ASSEMBLY_ORDER {
            let mut cube = [points[0]; 8];
            for (i, j) in order.iter().enumerate() {
                cube[i] = points[*j];
            }
            new_mesh.mk_cube(cube);
            for v_id in &cube {
                match vert_valence.get_mut(v_id) {
                    Some(v) => {
                        *v += 1;
                    }
                    None => {
                        vert_valence.insert(*v_id, 1);
                    }
                }
            }
        }
    }
    // Smoothing pass
    let mut smooth_vertices = HashMap::new();
    for v in new_mesh.vertices.iter() {
        smooth_vertices.insert(v.0, Vertex::new(0.0, 0.0, 0.0, 0.0, v.1.boundary));
    }
    for s in new_mesh.cubes.iter() {
        // Construct smoothing matrix based on boundaries and valence
        let mut bounds = [0; 8];
        for (i, b) in bounds.iter_mut().enumerate() {
            *b = new_mesh.vertices[s.1[i]].boundary;
        }
        let matrix = create_cube_matrix(bounds);
        // Apply the matrix
        for (i, row) in matrix.iter().enumerate() {
            let v = smooth_vertices.get_mut(&s.1[i]).unwrap();
            for (j, weight) in row.iter().enumerate() {
                let w = new_mesh.vertices[s.1[j]].clone();
                v.add_scaled(&w, *weight);
            }
        }
    }
    // Copy the final vertex positions and divide by valence
    for (v_id, v) in &smooth_vertices {
        let valence = vert_valence[v_id];
        let vtx = new_mesh.vertices.get_mut(*v_id).unwrap();
        vtx.copy_from(v);
        vtx.scale(1.0 / f64::from(valence));
    }
    debug_assert!(check_bounds_preserved(
        &control_mesh.vertices,
        &new_mesh.vertices
    ));
    new_mesh
}
*/
