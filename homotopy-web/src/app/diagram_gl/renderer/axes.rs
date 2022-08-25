use homotopy_gl::{array::VertexArray, buffer::ElementKind, vertex_array, GlCtx, Result};
use ultraviolet::Vec3;

pub struct Axes {
    pub array: VertexArray,
}

impl Axes {
    pub fn new(ctx: &GlCtx) -> Result<Self> {
        let axes_elements = ctx.mk_element_buffer(&[0, 1, 2, 3, 4, 5], ElementKind::Lines)?;
        let axes_verts = ctx.mk_buffer(&[
            Vec3::zero(),
            Vec3::unit_x(),
            Vec3::zero(),
            Vec3::unit_y(),
            Vec3::zero(),
            Vec3::unit_z(),
        ])?;
        let axes_albedos = ctx.mk_buffer(&[
            Vec3::unit_x(),
            Vec3::unit_x(),
            Vec3::unit_y(),
            Vec3::unit_y(),
            Vec3::unit_z(),
            Vec3::unit_z(),
        ])?;

        Ok(Self {
            array: vertex_array!(ctx, &axes_elements, [&axes_verts, &axes_albedos])?,
        })
    }
}
