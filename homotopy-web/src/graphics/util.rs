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
        // TODO(@doctorn) write safety note
        //
        // (just have to be careful we don't allocate memory between `Float32Array::view` and
        // `buffer_data_with_array_buffer_view`)
        unsafe {
            let vert_array = js_sys::Float32Array::view(data);

            self.ctx.webgl_ctx.buffer_data_with_array_buffer_view(
                T::BUFFER_KIND as u32,
                &vert_array,
                // TODO(@doctorn) investigate other options
                WebGlRenderingContext::STATIC_DRAW,
            );
        }
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
    pub fn bind<'a, T, U, F>(&'a self, buffer: &'a T, f: F) -> U
    where
        T: BufferObject,
        F: FnOnce(&BoundBuffer<'a, T>) -> U,
    {
        let bound = BoundBuffer::bind(self, buffer);
        f(&bound)
    }
}
