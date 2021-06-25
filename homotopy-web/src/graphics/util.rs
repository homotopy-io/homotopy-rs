use std::ops::Deref;

use js_sys;
use web_sys::{WebGlBuffer, WebGlRenderingContext};

use super::{Coord, GraphicsCtx};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BufferKind {
    Array = WebGlRenderingContext::ARRAY_BUFFER as isize,
}

pub trait BufferObject {
    const BUFFER_KIND: BufferKind;

    fn underlying_buffer(&self) -> &WebGlBuffer;
}

pub struct BoundBuffer<'a, T>
where
    T: BufferObject,
{
    ctx: &'a GraphicsCtx,
    buffer: &'a T,
}

impl<'a, T> BoundBuffer<'a, T>
where
    T: BufferObject,
{
    #[inline]
    fn bind(ctx: &'a GraphicsCtx, buffer: &'a T) -> Self {
        ctx.webgl_ctx
            .bind_buffer(T::BUFFER_KIND as u32, Some(buffer.underlying_buffer()));
        Self { ctx, buffer }
    }

    pub(super) fn buffer_data(&self, data: &[Coord]) {
        let vert_array = unsafe { js_sys::Float32Array::view(data) };

        self.ctx.webgl_ctx.buffer_data_with_array_buffer_view(
            T::BUFFER_KIND as u32,
            &vert_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }
}

impl<'a, T> Deref for BoundBuffer<'a, T>
where
    T: BufferObject,
{
    type Target = WebGlBuffer;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.buffer.underlying_buffer()
    }
}

impl<'a, T> Drop for BoundBuffer<'a, T>
where
    T: BufferObject,
{
    #[inline]
    fn drop(&mut self) {
        self.ctx.webgl_ctx.bind_buffer(T::BUFFER_KIND as u32, None);
    }
}

impl GraphicsCtx {
    #[inline]
    pub fn bind<'a, T>(&'a self, buffer: &'a T) -> BoundBuffer<'a, T>
    where
        T: BufferObject,
    {
        BoundBuffer::bind(self, buffer)
    }
}
