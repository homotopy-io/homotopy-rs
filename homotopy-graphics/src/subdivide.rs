use std::collections::HashMap;

use homotopy_common::idx::IdxVec;

use crate::mesh::*;

/// Defines how new squares should be from linearly divided points.
/// See Picture 1.
/// Important property that is preserved - first point is a vertex point, and similar for edge points.
const SQUARE_ASSEMBLY_ORDER: [[usize; 4]; 4] =
    [[0, 4, 5, 8], [1, 6, 4, 8], [2, 5, 7, 8], [3, 7, 6, 8]];

/// Generates a weight matrix for each surface element based on boundary information.
/// Also does basic mesh validation - panics if the boundaries are inconsistent.
fn create_square_matrix(bounds: [u8; 4]) -> [[f64; 4]; 4] {
    let mut matrix: [[f64; 4]; 4] = [[0.0; 4]; 4];
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

/// Debug check that signals that weights are reasonable
fn check_bounds_preserved(
    original: &IdxVec<VertexId, Vertex>,
    subdivided: &IdxVec<VertexId, Vertex>,
) -> bool {
    let (max_x, min_x) = original
        .iter()
        .fold((f64::NEG_INFINITY, f64::INFINITY), |a, v| {
            (f64::max(a.0, v.1.x), f64::min(a.1, v.1.x))
        });
    let (max_y, min_y) = original
        .iter()
        .fold((f64::NEG_INFINITY, f64::INFINITY), |a, v| {
            (f64::max(a.0, v.1.y), f64::min(a.1, v.1.y))
        });
    let (max_z, min_z) = original
        .iter()
        .fold((f64::NEG_INFINITY, f64::INFINITY), |a, v| {
            (f64::max(a.0, v.1.z), f64::min(a.1, v.1.z))
        });
    let (max_t, min_t) = original
        .iter()
        .fold((f64::NEG_INFINITY, f64::INFINITY), |a, v| {
            (f64::max(a.0, v.1.t), f64::min(a.1, v.1.t))
        });

    subdivided.iter().fold(true, |a, v| {
        a && (v.1.x <= max_x && v.1.x >= min_x)
            && (v.1.y <= max_y && v.1.y >= min_y)
            && (v.1.z <= max_z && v.1.z >= min_z)
            && (v.1.t <= max_t && v.1.t >= min_t)
    })
}

/// Subdivides square mesh one time based on the primal scheme.
/// Assumption: the mesh is filtered so only the relevant elements are there and all elments have some area.
pub fn subdivide3(control_mesh: &SquareMesh) -> SquareMesh {
    let mut new_mesh = SquareMesh::new();
    // Copy vertices as primal subdivision preserves them.
    let mut vert_map = HashMap::new();
    for v in control_mesh.vertices.iter() {
        vert_map.insert(v.0, new_mesh.mk_vertex(v.1.clone()));
    }
    // Vertex point valence is calculated differently because smoothing weights depend on it.
    let mut valence = HashMap::new();
    // Linearly divide each square
    for e in control_mesh.squares.iter() {
        // Translate to new mesh indices
        let mut points = [e.1[0]; 9];
        for (i, p) in points.iter_mut().enumerate().take(4) {
            *p = vert_map[&e.1[i]];
        }
        // Calculate edge midpoints
        points[4] = new_mesh.linearly_divide(points[0], points[1]);
        points[5] = new_mesh.linearly_divide(points[0], points[2]);
        points[6] = new_mesh.linearly_divide(points[1], points[3]);
        points[7] = new_mesh.linearly_divide(points[2], points[3]);
        // centre point..
        points[8] = new_mesh.linearly_divide(points[4], points[7]);
        // Assemble subsquares
        for order in &SQUARE_ASSEMBLY_ORDER {
            let mut sq = [points[0]; 4];
            for j in 0..4 {
                sq[j] = points[order[j]];
            }
            new_mesh.mk_square(sq);
            for v_id in &sq {
                match valence.get_mut(v_id) {
                    Some(v) => {
                        *v += 1;
                    }
                    None => {
                        valence.insert(*v_id, 1);
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
    for s in new_mesh.squares.iter() {
        // Construct smoothing matrix based on boundaries and valence
        let mut bounds = [0; 4];
        for (i, b) in bounds.iter_mut().enumerate() {
            *b = new_mesh.vertices[s.1[i]].boundary;
        }
        let matrix = create_square_matrix(bounds);
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
        let valence = valence[&v_id];
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
        let valence = vert_valence[&v_id];
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
