use std::marker::PhantomData;
use std::slice;

use euclid::{Vector2D, Vector3D};

use js_sys;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use super::{GraphicsCtx, GraphicsError, Result};

#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BufferKind {
    Array = WebGl2RenderingContext::ARRAY_BUFFER as isize,
    ElementArray = WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER as isize,
}

#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BufferUsage {
    // Static
    StaticDraw = WebGl2RenderingContext::STATIC_DRAW as isize,
    StaticRead = WebGl2RenderingContext::STATIC_READ as isize,
    StaticCopy = WebGl2RenderingContext::STATIC_COPY as isize,
    // Stream
    StreamDraw = WebGl2RenderingContext::STREAM_DRAW as isize,
    StreamRead = WebGl2RenderingContext::STREAM_READ as isize,
    StreamCopy = WebGl2RenderingContext::STREAM_COPY as isize,
    // Dynamic
    DynamicDraw = WebGl2RenderingContext::DYNAMIC_DRAW as isize,
    DynamicRead = WebGl2RenderingContext::DYNAMIC_READ as isize,
    DynamicCopy = WebGl2RenderingContext::DYNAMIC_COPY as isize,
}

trait UnsafeBufferable: Sized {
    unsafe fn buffer_to_unchecked<T>(buffer: &mut Buffer<T>, data: &[Self], len: usize);
}

pub trait Bufferable: Sized {
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]);
}

impl Default for BufferKind {
    fn default() -> Self {
        BufferKind::Array
    }
}

impl Default for BufferUsage {
    fn default() -> Self {
        BufferUsage::StaticDraw
    }
}

pub struct Buffer<T> {
    ctx: WebGl2RenderingContext,

    kind: BufferKind,
    usage: BufferUsage,
    len: usize,

    webgl_buffer: WebGlBuffer,

    _phantom: PhantomData<T>,
}

impl<T> Buffer<T> {
    pub fn new_with_kind_and_usage(
        ctx: &GraphicsCtx,
        kind: BufferKind,
        usage: BufferUsage,
    ) -> Result<Self> {
        let webgl_buffer = ctx
            .webgl_ctx
            .create_buffer()
            .ok_or(GraphicsError::Allocate)?;

        Ok(Self {
            ctx: ctx.webgl_ctx.clone(),
            kind,
            usage,
            len: 0,
            webgl_buffer,
            _phantom: Default::default(),
        })
    }

    pub fn new(ctx: &GraphicsCtx) -> Result<Self> {
        Buffer::new_with_kind_and_usage(ctx, Default::default(), Default::default())
    }

    pub fn buffer(&mut self, data: &[T])
    where
        T: Bufferable,
    {
        T::buffer_to(self, data);
    }

    pub(super) fn bind<F, U>(&self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        self.ctx
            .bind_buffer(self.kind as u32, Some(&self.webgl_buffer));
        let result = f();
        self.ctx.bind_buffer(self.kind as u32, None);
        result
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        self.ctx.delete_buffer(Some(&self.webgl_buffer));
    }
}

impl UnsafeBufferable for f32 {
    unsafe fn buffer_to_unchecked<T>(buffer: &mut Buffer<T>, data: &[f32], len: usize) {
        buffer.len = len;
        buffer.bind(|| {
            let view = js_sys::Float32Array::view(data);
            buffer.ctx.buffer_data_with_array_buffer_view(
                buffer.kind as u32,
                &view,
                buffer.usage as u32,
            );
        });
    }
}

impl Bufferable for f32 {
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]) {
        // TODO(@doctorn) safety note
        unsafe {
            f32::buffer_to_unchecked(buffer, data, data.len());
        }
    }
}

impl<T> Bufferable for Vector2D<f32, T> {
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]) {
        // TODO(@doctorn) safety note
        unsafe {
            let f32_slice = slice::from_raw_parts(data.as_ptr() as *const f32, data.len() * 2);
            f32::buffer_to_unchecked(buffer, f32_slice, data.len());
        }
    }
}

impl<T> Bufferable for Vector3D<f32, T> {
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]) {
        // TODO(@doctorn) safety note
        unsafe {
            let f32_slice = slice::from_raw_parts(data.as_ptr() as *const f32, data.len() * 3);
            f32::buffer_to_unchecked(buffer, f32_slice, data.len());
        }
    }
}
