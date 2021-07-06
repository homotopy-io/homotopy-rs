use homotopy_core::declare_idx;

use web_sys::{WebGlBuffer, WebGlRenderingContext};

use super::util::{BufferKind, BufferObject};
use super::{geom, GraphicsCtx, GraphicsError, Result};

declare_idx! {
    pub struct VertexBuffer = usize;
}

pub struct VertexBufferData {
    webgl_buffer: WebGlBuffer,
    len: usize,
}

impl VertexBufferData {
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

impl BufferObject for VertexBufferData {
    const BUFFER_KIND: BufferKind = BufferKind::Array;

    #[inline]
    fn underlying_buffer(&self) -> &WebGlBuffer {
        &self.webgl_buffer
    }
}

impl GraphicsCtx {
    pub fn mk_vertex_buffer(&mut self, data: &[geom::Vertex]) -> Result<VertexBuffer> {
        let webgl_buffer = self
            .webgl_ctx
            .create_buffer()
            .ok_or(GraphicsError::BufferAllocate)?;
        let vertex_buffer = VertexBufferData {
            webgl_buffer,
            len: data.len(),
        };

        self.bind(&vertex_buffer, |bound| {
            bound.buffer_data(data);

            // TODO(@doctorn) this shouldn't be done here
            self.webgl_ctx.vertex_attrib_pointer_with_i32(
                0,
                3,
                WebGlRenderingContext::FLOAT,
                false,
                0,
                0,
            );
        });

        Ok(self.vertex_buffers.push(vertex_buffer))
    }
}
