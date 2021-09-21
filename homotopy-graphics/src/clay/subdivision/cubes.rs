use std::{cmp, collections::HashMap, mem};

use ultraviolet::{Mat4, Vec4};

use super::engine::{InterpolationCtx, SmoothingCtx, Subdivider};
use crate::clay::geom::{Boundary, Cube, CubeData, SquareData, Vertex, VertexExt};

pub struct CubeSubdivider {
    edge_division_memory: HashMap<(Vertex, Vertex), Vertex>,
    face_division_memory: HashMap<SquareData, Vertex>,
}

impl CubeSubdivider {
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

    fn divide_edge_uncached(ctx: &mut InterpolationCtx<Self>, v_1: Vertex, v_2: Vertex) -> Vertex {
        // Perform division
        let v = {
            let v_1 = &ctx.mesh.vertices[v_1];
            let v_2 = &ctx.mesh.vertices[v_2];
            let v = 0.5 * (**v_1 + **v_2);
            let boundary = cmp::max(Boundary::One, cmp::max(v_1.boundary, v_2.boundary));
            let generator = if v_1.generator.dimension < v_2.generator.dimension {
                v_1.generator
            } else {
                v_2.generator
            };

            ctx.mesh
                .mk_vertex(v.with_boundary_and_generator(boundary, generator))
        };
        // Cache result
        ctx.edge_division_memory.insert((v_1, v_2), v);
        v
    }

    fn divide_edge(ctx: &mut InterpolationCtx<Self>, mut v_1: Vertex, mut v_2: Vertex) -> Vertex {
        if v_1 == v_2 {
            return v_1;
        }

        if v_2 > v_1 {
            mem::swap(&mut v_1, &mut v_2);
        }

        ctx.edge_division_memory
            .get(&(v_1, v_2))
            .copied()
            .unwrap_or_else(|| Self::divide_edge_uncached(ctx, v_1, v_2))
    }

    fn divide_face_uncached(ctx: &mut InterpolationCtx<Self>, square: SquareData) -> Vertex {
        // Perform division
        let v = {
            let v_1 = &ctx.mesh.vertices[square[0]];
            let v_2 = &ctx.mesh.vertices[square[1]];
            let v_3 = &ctx.mesh.vertices[square[2]];
            let v_4 = &ctx.mesh.vertices[square[3]];
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

            ctx.mesh
                .mk_vertex(v.with_boundary_and_generator(boundary, generator))
        };
        // Cache result
        ctx.face_division_memory.insert(square, v);
        v
    }

    fn divide_face(ctx: &mut InterpolationCtx<Self>, mut square: SquareData) -> Vertex {
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
            return Self::divide_edge(ctx, square[0], square[3]);
        }

        ctx.face_division_memory
            .get(&square)
            .copied()
            .unwrap_or_else(|| Self::divide_face_uncached(ctx, square))
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
}

impl Subdivider for CubeSubdivider {
    type Primitive = CubeData;

    #[inline]
    fn new() -> Self {
        Self {
            edge_division_memory: HashMap::new(),
            face_division_memory: HashMap::new(),
        }
    }

    #[inline]
    fn interpolate(ctx: &mut InterpolationCtx<Self>, cube: CubeData) {
        let mut points = {
            use homotopy_common::idx::Idx;
            [Vertex::new(0); 27]
        };

        points[0..8].copy_from_slice(&cube);

        for (i, edge) in Self::CUBE_EDGE_ORDER.iter().enumerate() {
            points[i + 8] = Self::divide_edge(ctx, points[edge[0]], points[edge[1]]);
        }

        for (i, face) in Self::CUBE_FACE_ORDER.iter().enumerate() {
            points[i + 20] = Self::divide_face(
                ctx,
                [
                    points[face[0]],
                    points[face[1]],
                    points[face[2]],
                    points[face[3]],
                ],
            );
        }

        points[26] = Self::divide_edge(ctx, points[20], points[25]);

        for order in &Self::CUBE_ASSEMBLY_ORDER {
            ctx.mesh.mk_cube([
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
                ctx.update_valence(points[*v]);
            }
        }
    }

    #[inline]
    fn smooth(ctx: &mut SmoothingCtx<Self>, cube: Cube) {
        let cube = ctx.mesh.elements[cube];
        // Gather the boundaries of its constituent vertices
        let bounds = [
            ctx.mesh.vertices[cube[0]].boundary,
            ctx.mesh.vertices[cube[1]].boundary,
            ctx.mesh.vertices[cube[2]].boundary,
            ctx.mesh.vertices[cube[3]].boundary,
            ctx.mesh.vertices[cube[4]].boundary,
            ctx.mesh.vertices[cube[5]].boundary,
            ctx.mesh.vertices[cube[6]].boundary,
            ctx.mesh.vertices[cube[7]].boundary,
        ];
        // and calculate a corresponding weight matrix
        let weights = Self::weight_matrix(bounds);
        // Shape vertices as matrix
        let upper = Mat4::new(
            *ctx.mesh.vertices[cube[0]],
            *ctx.mesh.vertices[cube[1]],
            *ctx.mesh.vertices[cube[2]],
            *ctx.mesh.vertices[cube[3]],
        );
        let lower = Mat4::new(
            *ctx.mesh.vertices[cube[4]],
            *ctx.mesh.vertices[cube[5]],
            *ctx.mesh.vertices[cube[6]],
            *ctx.mesh.vertices[cube[7]],
        );
        // Tranform
        let upper_transformed = upper * weights[0] + lower * weights[2];
        let lower_transformed = upper * weights[1] + lower * weights[3];
        // Update positions
        for i in 0..4 {
            ctx.update_smoothed(cube[i], upper_transformed[i]);
            ctx.update_smoothed(cube[4 + i], lower_transformed[i]);
        }
    }
}
