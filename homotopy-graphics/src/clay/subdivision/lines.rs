use std::cmp;

use ultraviolet::{Mat4, Vec4};

use super::engine::{InterpolationCtx, SmoothingCtx, Subdivider};
use crate::clay::geom::{Boundary, Line, LineData, Vertex, VertexExt};

pub struct LineSubdivider;

impl LineSubdivider {
    fn divide(ctx: &mut InterpolationCtx<Self>, v_1: Vertex, v_2: Vertex) -> Vertex {
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
    }

    fn weight_matrix(bounds: [Boundary; 2]) -> Mat4 {
        let row_0 = match bounds[0] {
            Boundary::Zero => Vec4::unit_x(),
            _ => Vec4::new(0.5, 0.5, 0., 0.),
        };
        let row_1 = Vec4::unit_y();

        Mat4::new(row_0, row_1, Vec4::zero(), Vec4::zero())
    }
}

impl Subdivider for LineSubdivider {
    type Primitive = LineData;

    #[inline]
    fn new() -> Self {
        Self
    }

    #[inline]
    fn interpolate(ctx: &mut InterpolationCtx<Self>, line: LineData) {
        let midpoint = Self::divide(ctx, line[0], line[1]);

        ctx.mesh.mk_line([line[0], midpoint]);
        ctx.mesh.mk_line([line[1], midpoint]);

        ctx.update_valence(line[0]);
        ctx.update_valence(midpoint);

        ctx.update_valence(line[1]);
        ctx.update_valence(midpoint);
    }

    #[inline]
    fn smooth(ctx: &mut SmoothingCtx<Self>, line: Line) {
        let line = ctx.mesh.elements[line];
        let bounds = [
            ctx.mesh.vertices[line[0]].boundary,
            ctx.mesh.vertices[line[1]].boundary,
        ];
        let weights = Self::weight_matrix(bounds);
        let line_matrix = Mat4::new(
            *ctx.mesh.vertices[line[0]],
            *ctx.mesh.vertices[line[1]],
            Vec4::zero(),
            Vec4::zero(),
        );
        let transformed = line_matrix * weights;
        for i in 0..2 {
            ctx.update_smoothed(line[i], transformed[i]);
        }
    }
}
