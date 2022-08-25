use homotopy_gl::{
    gl::{array::VertexArray, buffer::ElementKind, GlCtx, Result},
    vertex_array,
};
use ultraviolet::{Vec2, Vec3};

pub struct Quad {
    pub array: VertexArray,
}

impl Quad {
    pub fn new(ctx: &GlCtx) -> Result<Self> {
        let quad_elements = ctx.mk_element_buffer(&[0, 1, 2, 0, 2, 3], ElementKind::Triangles)?;
        let quad_verts = ctx.mk_buffer(&[
            Vec3::new(-1., 1., 0.),
            Vec3::new(-1., -1., 0.),
            Vec3::new(1., -1., 0.),
            Vec3::new(1., 1., 0.),
        ])?;
        let quad_uvs = ctx.mk_buffer(&[
            Vec2::new(0., 1.),
            Vec2::new(0., 0.),
            Vec2::new(1., 0.),
            Vec2::new(1., 1.),
        ])?;

        Ok(Self {
            array: vertex_array!(ctx, &quad_elements, [&quad_verts, &quad_uvs])?,
        })
    }
}
