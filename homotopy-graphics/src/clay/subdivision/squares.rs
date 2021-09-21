use std::{cmp, collections::HashMap, mem};

use ultraviolet::{Mat4, Vec4};

use super::engine::{InterpolationCtx, SmoothingCtx, Subdivider};
use crate::clay::geom::{Boundary, Square, SquareData, Vertex, VertexExt};

pub struct SquareSubdivider {
    division_memory: HashMap<(Vertex, Vertex), Vertex>,
}

impl SquareSubdivider {
    /// Defines how new squares should be from linearly divided points.
    ///
    /// Important property that is preserved - first point is a vertex point, and similar
    /// for edge points.
    const SQUARE_ASSEMBLY_ORDER: [[usize; 4]; 4] =
        [[0, 4, 5, 8], [1, 6, 4, 8], [2, 5, 7, 8], [3, 7, 6, 8]];

    fn divide_uncached(ctx: &mut InterpolationCtx<Self>, v_1: Vertex, v_2: Vertex) -> Vertex {
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
        ctx.division_memory.insert((v_1, v_2), v);
        v
    }

    fn divide(ctx: &mut InterpolationCtx<Self>, mut v_1: Vertex, mut v_2: Vertex) -> Vertex {
        if v_1 == v_2 {
            return v_1;
        }

        if v_2 > v_1 {
            mem::swap(&mut v_1, &mut v_2);
        }

        ctx.division_memory
            .get(&(v_1, v_2))
            .copied()
            .unwrap_or_else(|| Self::divide_uncached(ctx, v_1, v_2))
    }

    /// Generates a weight matrix for each surface element based on boundary information.
    fn weight_matrix(bounds: [Boundary; 4]) -> Mat4 {
        use Boundary::{One, Zero};
        // Vertex point
        let row_0 = match bounds[0] {
            Zero => Vec4::unit_x(),
            One => match (bounds[1], bounds[2]) {
                (One, _) => Vec4::new(0.5, 0.5, 0., 0.),
                (_, One) => Vec4::new(0.5, 0., 0.5, 0.),
                _ => Vec4::unit_x(),
            },
            _ => Vec4::broadcast(0.25),
        };
        let row_1 = match bounds[1] {
            One => Vec4::unit_y(),
            _ => Vec4::new(0., 0.5, 0., 0.5),
        };
        let row_2 = match bounds[2] {
            One => Vec4::unit_z(),
            _ => Vec4::new(0., 0., 0.5, 0.5),
        };
        let row_3 = Vec4::unit_w();

        Mat4::new(row_0, row_1, row_2, row_3)
    }
}

impl Subdivider for SquareSubdivider {
    type Primitive = SquareData;

    #[inline]
    fn new() -> Self {
        Self {
            division_memory: HashMap::new(),
        }
    }

    #[inline]
    fn interpolate(ctx: &mut InterpolationCtx<Self>, square: SquareData) {
        let v_1 = Self::divide(ctx, square[0], square[1]);
        let v_2 = Self::divide(ctx, square[2], square[3]);
        let points = [
            square[0],
            square[1],
            square[2],
            square[3],
            // Edge midpoints
            v_1,
            Self::divide(ctx, square[0], square[2]),
            Self::divide(ctx, square[1], square[3]),
            v_2,
            // Centrepoint
            Self::divide(ctx, v_1, v_2),
        ];

        for order in &Self::SQUARE_ASSEMBLY_ORDER {
            ctx.mesh.mk_square([
                points[order[0]],
                points[order[1]],
                points[order[2]],
                points[order[3]],
            ]);

            for v in order.iter() {
                ctx.update_valence(points[*v]);
            }
        }
    }

    #[inline]
    fn smooth(ctx: &mut SmoothingCtx<Self>, square: Square) {
        let square = ctx.mesh.elements[square];
        // Gather the boundaries of its constituent vertices
        let bounds = [
            ctx.mesh.vertices[square[0]].boundary,
            ctx.mesh.vertices[square[1]].boundary,
            ctx.mesh.vertices[square[2]].boundary,
            ctx.mesh.vertices[square[3]].boundary,
        ];
        // and calculate a corresponding weight matrix
        let weights = Self::weight_matrix(bounds);
        // Shape vertices as a matrix
        let square_matrix = Mat4::new(
            *ctx.mesh.vertices[square[0]],
            *ctx.mesh.vertices[square[1]],
            *ctx.mesh.vertices[square[2]],
            *ctx.mesh.vertices[square[3]],
        );
        // Transform
        let transformed = square_matrix * weights;
        // Update positions
        for i in 0..4 {
            ctx.update_smoothed(square[i], transformed[i]);
        }
    }
}
