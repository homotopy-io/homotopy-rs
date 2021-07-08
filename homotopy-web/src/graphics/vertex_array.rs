use homotopy_core::declare_idx;

use web_sys::WebGlVertexArrayObject;

use super::{Bindable, GraphicsCtx, GraphicsError, GraphicsObject, Result};

declare_idx! {
    pub struct VertexArray = usize;
}

pub(super) struct VertexArrayData {
    webgl_vao: WebGlVertexArrayObject,
}

impl GraphicsCtx {}

impl GraphicsObject for VertexArray {
    type Data = VertexArrayData;
    type Carrier = WebGlVertexArrayObject;

    #[inline]
    fn alloc_carrier(ctx: &mut GraphicsCtx) -> Result<Self::Carrier> {
        ctx.webgl_ctx
            .create_vertex_array()
            .ok_or(GraphicsError::Allocate)
    }

    #[inline]
    fn dealloc_carrier(self, ctx: &mut GraphicsCtx) {
        ctx.webgl_ctx
            .delete_vertex_array(Some(ctx.carrier_for(self)));
    }

    #[inline]
    fn get_data(self, ctx: &GraphicsCtx) -> &Self::Data {
        &ctx.vertex_arrays[self]
    }

    #[inline]
    fn get_carrier(self, ctx: &GraphicsCtx) -> &Self::Carrier {
        &ctx.vertex_arrays[self].webgl_vao
    }
}

impl Bindable for VertexArray {
    fn bind(self, ctx: &GraphicsCtx) {
        ctx.webgl_ctx.bind_vertex_array(Some(ctx.carrier_for(self)));
    }

    fn release(self, ctx: &GraphicsCtx) {
        ctx.webgl_ctx.bind_vertex_array(None);
    }
}
