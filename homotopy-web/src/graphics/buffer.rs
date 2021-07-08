use std::slice;

use homotopy_core::declare_idx;

use js_sys;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use super::{geom, Bindable, GraphicsCtx, GraphicsError, GraphicsObject, Result};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BufferKind {
    Array = WebGl2RenderingContext::ARRAY_BUFFER as isize,
}

declare_idx! {
    pub struct VertexBuffer = usize;
}

pub struct VertexBufferData {
    webgl_buffer: WebGlBuffer,
    kind: BufferKind,
    len: usize,
}

impl GraphicsObject for VertexBuffer {
    type Carrier = WebGlBuffer;
    type Data = VertexBufferData;

    #[inline]
    fn alloc_carrier(ctx: &mut GraphicsCtx) -> Result<Self::Carrier> {
        ctx.webgl_ctx.create_buffer().ok_or(GraphicsError::Allocate)
    }

    #[inline]
    fn dealloc_carrier(self, ctx: &mut GraphicsCtx) {
        ctx.webgl_ctx.delete_buffer(Some(ctx.carrier_for(self)));
    }

    #[inline]
    fn get_data(self, ctx: &GraphicsCtx) -> &Self::Data {
        &ctx.vertex_buffers[self]
    }

    #[inline]
    fn get_carrier(self, ctx: &GraphicsCtx) -> &Self::Carrier {
        &ctx.vertex_buffers[self].webgl_buffer
    }
}

impl Bindable for VertexBuffer {
    #[inline]
    fn bind(self, ctx: &GraphicsCtx) {
        let vertex_buffer = ctx.get(self);
        ctx.webgl_ctx
            .bind_buffer(vertex_buffer.kind as u32, Some(&vertex_buffer.webgl_buffer));
    }

    #[inline]
    fn release(self, ctx: &GraphicsCtx) {
        ctx.webgl_ctx.bind_buffer(ctx.get(self).kind as u32, None);
    }
}

impl GraphicsCtx {
    pub fn mk_vertex_buffer(&mut self, data: &[geom::Vertex]) -> Result<VertexBuffer> {
        let webgl_buffer = self.alloc::<VertexBuffer>()?;
        let vertex_buffer = self.vertex_buffers.push(VertexBufferData {
            webgl_buffer,
            kind: BufferKind::Array,
            len: data.len(),
        });

        self.bind(vertex_buffer, || {
            // TODO(@doctorn) write safety note
            //
            // (just have to be careful we don't allocate memory between `Float32Array::view` and
            // `buffer_data_with_array_buffer_view`)
            unsafe {
                let f32_slice = slice::from_raw_parts(data.as_ptr() as *const f32, data.len() * 3);
                let vert_array = js_sys::Float32Array::view(f32_slice);

                self.webgl_ctx.buffer_data_with_array_buffer_view(
                    self.get(vertex_buffer).kind as u32,
                    &vert_array,
                    // TODO(@doctorn) investigate other options
                    WebGl2RenderingContext::STATIC_DRAW,
                );
            }

            // TODO(@doctorn) this shouldn't be done here
            self.webgl_ctx.vertex_attrib_pointer_with_i32(
                0,
                3,
                WebGl2RenderingContext::FLOAT,
                false,
                0,
                0,
            );
        });

        Ok(vertex_buffer)
    }
}
