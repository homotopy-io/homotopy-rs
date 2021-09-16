//! Primal subdivision methods in this module follow a preset point order when subdividing elements.
//! For 2-cubes, 9 points are generated in the order (a). For 3-cubes 27 points are generated in order (b).
//! <img src="../../images/subdivision_illustration.png"/>

use std::{cmp, collections::HashMap, mem};

use homotopy_common::idx::{Idx, IdxVec};
use ultraviolet::{Mat4, Vec4};

use super::geom::{Boundary, CubeData, CubeMesh, SquareData, SquareMesh, Vertex, VertexExt};

struct SquareSubdivider<'a> {
    mesh: &'a mut SquareMesh,
    division_memory: HashMap<(Vertex, Vertex), Vertex>,
    valence: HashMap<Vertex, u32>,
}

struct CubeSubdivider<'a> {
    mesh: &'a mut CubeMesh,
    edge_division_memory: HashMap<(Vertex, Vertex), Vertex>,
    face_division_memory: HashMap<SquareData, Vertex>,
    valence: HashMap<Vertex, u32>,
}

impl<'a> SquareSubdivider<'a> {
    /// Defines how new squares should be from linearly divided points.
    ///
    /// Important property that is preserved - first point is a vertex point, and similar
    /// for edge points.
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
        // Perform division
        let v = {
            let v_1 = &self.mesh.vertices[v_1];
            let v_2 = &self.mesh.vertices[v_2];
            let v = 0.5 * (**v_1 + **v_2);
            let boundary = cmp::max(Boundary::One, cmp::max(v_1.boundary, v_2.boundary));
            let generator = if v_1.generator.dimension < v_2.generator.dimension {
                v_1.generator
            } else {
                v_2.generator
            };

            self.mesh
                .mk_vertex(v.with_boundary_and_generator(boundary, generator))
        };
        // Cache result
        self.division_memory.insert((v_1, v_2), v);
        v
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
            .unwrap_or_else(|| self.divide_uncached(v_1, v_2))
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
                *self.valence.entry(points[*v]).or_insert(0) += 1;
            }
        }
    }

    /// Subdivides the square mesh one time based on the primal scheme.
    ///
    /// This assumes that the mesh is filtered so that only the relevant elements
    /// are present and all elements have some area.
    fn subdivide_once(&mut self) {
        // (0. In debug, clone a copy of the original diagram for sanity checking)
        #[cfg(debug_assertions)]
        let unmodified = self.mesh.clone();

        // 1. Reset valence
        self.valence.clear();
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
            smoothed
                .push(Vec4::zero().with_boundary_and_generator(vertex.boundary, vertex.generator));
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
            let weights = Self::weight_matrix(bounds);
            // Shape vertices as a matrix
            let square_matrix = Mat4::new(
                *self.mesh.vertices[square[0]],
                *self.mesh.vertices[square[1]],
                *self.mesh.vertices[square[2]],
                *self.mesh.vertices[square[3]],
            );
            // Transform
            let transformed = square_matrix * weights;
            // Update positions
            for i in 0..4 {
                *smoothed[square[i]] += transformed[i];
            }
        }

        // c) Update vertex positions and divide by valence
        for (vertex, data) in smoothed {
            let valence = self.valence[&vertex];
            self.mesh.vertices[vertex].vertex = *data / (valence as f32);
        }

        // (5. In debug, sanity check the subdivided mesh)
        #[cfg(debug_assertions)]
        debug_assert!(self.check_bounds_preserved(&unmodified));
    }

    /// Generates a weight matrix for each surface element based on boundary information.
    ///
    /// Also does basic mesh validation - panics if the boundaries are inconsistent.
    fn weight_matrix(bounds: [Boundary; 4]) -> Mat4 {
        use Boundary::{One, Two, Zero};
        // Vertex point
        let row_0 = match bounds[0] {
            Zero => Vec4::unit_x(),
            One => match (bounds[1], bounds[2]) {
                (One, _) => Vec4::new(0.5, 0.5, 0.0, 0.0),
                (_, One) => Vec4::new(0.5, 0.0, 0.5, 0.0),
                // if it turns out such mesh is consistent, replace with the identity row
                _ => panic!("Inconsistent mesh!"),
            },
            _ => Vec4::broadcast(0.25),
        };
        let row_1 = match bounds[1] {
            One => Vec4::unit_y(),
            Two => Vec4::new(0.0, 0.5, 0.0, 0.5),
            _ => panic!("Inconsistent mesh!"),
        };
        let row_2 = match bounds[2] {
            One => Vec4::unit_z(),
            Two => Vec4::new(0.0, 0.0, 0.5, 0.5),
            _ => panic!("Inconsistent mesh!"),
        };
        let row_3 = Vec4::unit_w();

        Mat4::new(row_0, row_1, row_2, row_3)
    }

    #[cfg(debug_assertions)]
    fn check_bounds_preserved(&self, unmodified: &SquareMesh) -> bool {
        let (min, max) = unmodified.vertices.values().fold(
            (
                Vec4::broadcast(f32::INFINITY),
                Vec4::broadcast(f32::NEG_INFINITY),
            ),
            |a, v| (a.0.min_by_component(**v), a.1.max_by_component(**v)),
        );

        self.mesh
            .vertices
            .values()
            .all(|v| v.clamped(min, max) == **v)
    }
}

impl<'a> CubeSubdivider<'a> {
    /// Defines how new cubes should be from linearly divided points.
    ///
    /// Important property that is preserved - first point is a vertex point,
    /// edge and face points are also in precise positions.
    const CUBE_ASSEMBLY_ORDER: [[usize; 8]; 8] = [
        [0, 8, 9, 20, 10, 21, 22, 26],
        [1, 11, 8, 20, 12, 23, 21, 26],
        [2, 9, 13, 20, 14, 22, 24, 26],
        [3, 13, 11, 20, 15, 24, 23, 26],
        [4, 16, 10, 21, 17, 25, 22, 26],
        [5, 18, 12, 23, 16, 25, 21, 26],
        [6, 17, 14, 22, 19, 25, 24, 26],
        [7, 19, 15, 24, 18, 25, 23, 26],
    ];
    /// Defines all 12 edges of a cube based on vertex indices.
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
    /// Defines all 6 faces of a cube based on vertex indices.
    const CUBE_FACE_ORDER: [[usize; 4]; 6] = [
        [0, 1, 2, 3],
        [0, 1, 4, 5],
        [0, 2, 4, 6],
        [1, 3, 5, 7],
        [2, 3, 6, 7],
        [4, 5, 6, 7],
    ];

    fn new(mesh: &'a mut CubeMesh) -> Self {
        Self {
            mesh,
            edge_division_memory: Default::default(),
            face_division_memory: Default::default(),
            valence: Default::default(),
        }
    }

    fn divide_edge_uncached(&mut self, v_1: Vertex, v_2: Vertex) -> Vertex {
        // Perform division
        let v = {
            let v_1 = &self.mesh.vertices[v_1];
            let v_2 = &self.mesh.vertices[v_2];
            let v = 0.5 * (**v_1 + **v_2);
            let boundary = cmp::max(Boundary::One, cmp::max(v_1.boundary, v_2.boundary));
            let generator = if v_1.generator.dimension < v_2.generator.dimension {
                v_1.generator
            } else {
                v_2.generator
            };

            self.mesh
                .mk_vertex(v.with_boundary_and_generator(boundary, generator))
        };
        // Cache result
        self.edge_division_memory.insert((v_1, v_2), v);
        v
    }

    fn divide_edge(&mut self, mut v_1: Vertex, mut v_2: Vertex) -> Vertex {
        if v_1 == v_2 {
            return v_1;
        }

        if v_2 > v_1 {
            mem::swap(&mut v_1, &mut v_2);
        }

        self.edge_division_memory
            .get(&(v_1, v_2))
            .copied()
            .unwrap_or_else(|| self.divide_edge_uncached(v_1, v_2))
    }

    fn divide_face_uncached(&mut self, square: SquareData) -> Vertex {
        // Perform division
        let v = {
            let v_1 = &self.mesh.vertices[square[0]];
            let v_2 = &self.mesh.vertices[square[1]];
            let v_3 = &self.mesh.vertices[square[2]];
            let v_4 = &self.mesh.vertices[square[3]];
            let v = 0.25 * (**v_1 + **v_2 + **v_3 + **v_4);
            let boundary = cmp::max(
                Boundary::Two,
                cmp::max(
                    cmp::max(v_1.boundary, v_2.boundary),
                    cmp::max(v_3.boundary, v_4.boundary),
                ),
            );
            let generator = {
                let mut vertices = [v_1, v_2, v_3, v_4];
                vertices.sort_by_key(|v| v.generator.dimension);
                vertices[0].generator
            };

            self.mesh
                .mk_vertex(v.with_boundary_and_generator(boundary, generator))
        };
        // Cache result
        self.face_division_memory.insert(square, v);
        v
    }

    fn divide_face(&mut self, mut square: SquareData) -> Vertex {
        square.sort_unstable();

        // After sorting, we know that all the vertices are identical
        // if and only if the first vertex equals the last vertex. In
        // this case, we just return this unique vertex.
        if square[0] == square[3] {
            return square[0];
        }

        // In any of these three cases, we have a single pair of vertices.
        // As this corresponds to an edge, we just need to divide that edge
        // and move on.
        if (square[0] != square[1] && square[1] == square[3])
            || (square[0] == square[1] && square[2] == square[3])
            || (square[0] == square[2] && square[2] != square[3])
        {
            return self.divide_edge(square[0], square[3]);
        }

        self.face_division_memory
            .get(&square)
            .copied()
            .unwrap_or_else(|| self.divide_face_uncached(square))
    }

    fn subdivide_cube(&mut self, cube: CubeData) {
        let mut points = [Vertex::new(0); 27];

        points[0..8].copy_from_slice(&cube);

        for (i, edge) in Self::CUBE_EDGE_ORDER.iter().enumerate() {
            points[i + 8] = self.divide_edge(points[edge[0]], points[edge[1]]);
        }

        for (i, face) in Self::CUBE_FACE_ORDER.iter().enumerate() {
            points[i + 20] = self.divide_face([
                points[face[0]],
                points[face[1]],
                points[face[2]],
                points[face[3]],
            ]);
        }

        points[26] = self.divide_edge(points[20], points[25]);

        for order in &Self::CUBE_ASSEMBLY_ORDER {
            self.mesh.mk_cube([
                points[order[0]],
                points[order[1]],
                points[order[2]],
                points[order[3]],
                points[order[4]],
                points[order[5]],
                points[order[6]],
                points[order[7]],
            ]);

            for v in order.iter() {
                *self.valence.entry(points[*v]).or_insert(0) += 1;
            }
        }
    }

    /// Subdivides the cube mesh one time based on the primal scheme.
    ///
    /// This assumes that the mesh is filtered so that only the relevant elements
    /// are present and all elements have some volume.
    fn subdivide_once(&mut self) {
        // (0. In debug, clone a copy of the original diagram for sanity checking)
        #[cfg(debug_assertions)]
        let unmodified = self.mesh.clone();

        // 1. Reset valence
        self.valence.clear();
        // 2. Remove all cubes from mesh
        let mut cubes = IdxVec::new();
        mem::swap(&mut self.mesh.elements, &mut cubes);

        // 3. Subdivide each square linearly
        for cube in cubes.into_values() {
            self.subdivide_cube(cube);
        }

        // 4. Smooth
        let mut smoothed = IdxVec::with_capacity(self.mesh.vertices.len());

        // a) Populate with zero vertices
        for vertex in self.mesh.vertices.values() {
            smoothed
                .push(Vec4::zero().with_boundary_and_generator(vertex.boundary, vertex.generator));
        }

        // b) For each cube
        for cube in self.mesh.elements.values() {
            // gather the boundaries of its constituent vertices
            let bounds = [
                self.mesh.vertices[cube[0]].boundary,
                self.mesh.vertices[cube[1]].boundary,
                self.mesh.vertices[cube[2]].boundary,
                self.mesh.vertices[cube[3]].boundary,
                self.mesh.vertices[cube[4]].boundary,
                self.mesh.vertices[cube[5]].boundary,
                self.mesh.vertices[cube[6]].boundary,
                self.mesh.vertices[cube[7]].boundary,
            ];
            // and calculate a corresponding weight matrix
            let weights = Self::weight_matrix(bounds);
            // Shape vertices as matrix
            let upper = Mat4::new(
                *self.mesh.vertices[cube[0]],
                *self.mesh.vertices[cube[1]],
                *self.mesh.vertices[cube[2]],
                *self.mesh.vertices[cube[3]],
            );
            let lower = Mat4::new(
                *self.mesh.vertices[cube[4]],
                *self.mesh.vertices[cube[5]],
                *self.mesh.vertices[cube[6]],
                *self.mesh.vertices[cube[7]],
            );
            // Tranform
            let upper_transformed = upper * weights[0] + lower * weights[2];
            let lower_transformed = upper * weights[1] + lower * weights[3];
            // Update positions
            for i in 0..4 {
                *smoothed[cube[i]] += upper_transformed[i];
                *smoothed[cube[4 + i]] += lower_transformed[i];
            }
        }

        for (vertex, data) in smoothed {
            let valence = self.valence[&vertex];
            self.mesh.vertices[vertex].vertex = *data / (valence as f32);
        }

        // (5. In debug, sanity check the subdivided mesh)
        #[cfg(debug_assertions)]
        debug_assert!(self.check_bounds_preserved(&unmodified));
    }

    /// Generates a weight matrix for each surface element based on boundary information.
    ///
    /// Also does basic mesh validation - panics if the boundaries are inconsistent.
    fn weight_matrix(bounds: [Boundary; 8]) -> [Mat4; 4] {
        use Boundary::{One, Three, Two, Zero};

        // Vertex point
        let (row_00, row_01) = match bounds[0] {
            Zero => (Vec4::unit_x(), Vec4::zero()),
            One => match (bounds[1], bounds[2], bounds[4]) {
                (One, _, _) => (Vec4::new(0.5, 0.5, 0.0, 0.), Vec4::zero()),
                (_, One, _) => (Vec4::new(0.5, 0., 0.5, 0.), Vec4::zero()),
                (_, _, One) => (Vec4::new(0.5, 0., 0., 0.), Vec4::new(0.5, 0., 0., 0.)),
                // if it turns out such mesh is consistent, replace with the identity row
                _ => panic!("Inconsistent mesh!"),
            },
            Two => match (bounds[3], bounds[5], bounds[6]) {
                (Two, _, _) => (Vec4::broadcast(0.25), Vec4::zero()),
                (_, Two, _) => (Vec4::new(0.25, 0.25, 0., 0.), Vec4::new(0.25, 0.25, 0., 0.)),
                (_, _, Two) => (Vec4::new(0.25, 0., 0.25, 0.), Vec4::new(0.25, 0., 0.25, 0.)),
                // if it turns out such mesh is consistent, replace with the identity row
                _ => panic!("Inconsistent mesh!"),
            },
            Three => (Vec4::broadcast(0.125), Vec4::broadcast(0.125)),
        };

        // Edge points
        let (row_10, row_11) = match bounds[1] {
            Zero => panic!("Inconsistent mesh!"),
            One => (Vec4::unit_y(), Vec4::zero()),
            Two => match (bounds[3], bounds[5]) {
                (Two, _) => (Vec4::new(0.0, 0.5, 0.0, 0.5), Vec4::zero()),
                (_, Two) => (Vec4::new(0., 0.5, 0., 0.), Vec4::new(0., 0.5, 0., 0.)),
                // if it turns out such mesh is consistent, replace with the identity row
                _ => panic!("Inconsistent mesh!"),
            },
            Three => (Vec4::new(0.25, 0., 0.25, 0.), Vec4::new(0., 0.25, 0., 0.25)),
        };
        let (row_20, row_21) = match bounds[2] {
            Zero => panic!("Inconsistent mesh!"),
            One => (Vec4::unit_z(), Vec4::zero()),
            Two => match (bounds[3], bounds[6]) {
                (Two, _) => (Vec4::new(0., 0., 0.5, 0.5), Vec4::zero()),
                (_, Two) => (Vec4::new(0., 0., 0.5, 0.), Vec4::new(0., 0., 0.5, 0.)),
                // if it turns out such mesh is consistent, replace with the identity row
                _ => panic!("Inconsistent mesh!"),
            },
            Three => (Vec4::new(0., 0., 0.25, 0.25), Vec4::new(0., 0., 0.25, 0.25)),
        };
        let (row_40, row_41) = match bounds[4] {
            Zero => panic!("Inconsistent mesh!"),
            One => (Vec4::zero(), Vec4::unit_x()),
            Two => match (bounds[5], bounds[6]) {
                (Two, _) => (Vec4::zero(), Vec4::new(0.5, 0.5, 0., 0.)),
                (_, Two) => (Vec4::zero(), Vec4::new(0.5, 0., 0.5, 0.)),
                // if it turns out such mesh is consistent, replace with the identity row
                _ => panic!("Inconsistent mesh!"),
            },
            Three => (Vec4::zero(), Vec4::broadcast(0.25)),
        };

        // Face points
        let (row_30, row_31) = match bounds[3] {
            Two => (Vec4::unit_w(), Vec4::zero()),
            Three => (Vec4::new(0., 0., 0., 0.5), Vec4::new(0., 0., 0., 0.5)),
            _ => panic!("Inconsistent mesh!"),
        };
        let (row_50, row_51) = match bounds[5] {
            Two => (Vec4::zero(), Vec4::unit_y()),
            Three => (Vec4::zero(), Vec4::new(0., 0.5, 0., 0.5)),
            _ => panic!("Inconsistent mesh!"),
        };
        let (row_60, row_61) = match bounds[6] {
            Two => (Vec4::zero(), Vec4::unit_z()),
            Three => (Vec4::zero(), Vec4::new(0., 0., 0.5, 0.5)),
            _ => panic!("Inconsistent mesh!"),
        };

        // Centroid
        let (row_70, row_71) = (Vec4::zero(), Vec4::unit_w());

        [
            Mat4::new(row_00, row_10, row_20, row_30),
            Mat4::new(row_40, row_50, row_60, row_70),
            Mat4::new(row_01, row_11, row_21, row_31),
            Mat4::new(row_41, row_51, row_61, row_71),
        ]
    }

    #[cfg(debug_assertions)]
    fn check_bounds_preserved(&self, unmodified: &CubeMesh) -> bool {
        let (min, max) = unmodified.vertices.values().fold(
            (
                Vec4::broadcast(f32::INFINITY),
                Vec4::broadcast(f32::NEG_INFINITY),
            ),
            |a, v| (a.0.min_by_component(**v), a.1.max_by_component(**v)),
        );

        self.mesh
            .vertices
            .values()
            .all(|v| v.clamped(min, max) == **v)
    }
}

pub fn subdivide_3(mut mesh: SquareMesh, depth: u8) -> SquareMesh {
    let mut subdivider = SquareSubdivider::new(&mut mesh);

    for _ in 0..depth {
        subdivider.subdivide_once();
    }

    mesh
}

pub fn subdivide_4(mut mesh: CubeMesh, depth: u8) -> CubeMesh {
    let mut subdivider = CubeSubdivider::new(&mut mesh);

    for _ in 0..depth {
        subdivider.subdivide_once();
    }

    mesh
}
