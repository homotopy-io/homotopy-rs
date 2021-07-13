use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::slice;

use euclid::{Vector2D, Vector3D};

use js_sys;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use super::{GlCtx, GlError, Result};

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

pub(super) struct UntypedBuffer {
    ctx: WebGl2RenderingContext,

    kind: BufferKind,
    usage: BufferUsage,

    len: Cell<usize>,

    webgl_buffer: WebGlBuffer,
}

pub struct Buffer<T> {
    buffer: Rc<UntypedBuffer>,
    _phantom: PhantomData<T>,
}

pub unsafe trait UnsafeBufferable: Sized {
    unsafe fn buffer_to_unchecked<T>(buffer: &mut Buffer<T>, data: &[Self], len: usize);
}

pub trait Bufferable: Sized {
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]);
}

impl<T> Buffer<T> {
    pub fn new_with_kind_and_usage(
        ctx: &GlCtx,
        kind: BufferKind,
        usage: BufferUsage,
    ) -> Result<Self> {
        let webgl_buffer = ctx.webgl_ctx.create_buffer().ok_or(GlError::Allocate)?;

        let buffer = UntypedBuffer {
            ctx: ctx.webgl_ctx.clone(),
            kind,
            usage,
            len: Cell::new(0),
            webgl_buffer,
        };

        Ok(Self {
            buffer: Rc::new(buffer),
            _phantom: Default::default(),
        })
    }

    pub fn new(ctx: &GlCtx) -> Result<Self> {
        Buffer::new_with_kind_and_usage(ctx, Default::default(), Default::default())
    }

    pub fn buffer(&mut self, data: &[T])
    where
        T: Bufferable,
    {
        T::buffer_to(self, data);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len.get()
    }

    #[inline(always)]
    pub(super) fn bind<F, U>(&self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        self.buffer
            .ctx
            .bind_buffer(self.buffer.kind as u32, Some(&self.buffer.webgl_buffer));
        let result = f();
        self.buffer.ctx.bind_buffer(self.buffer.kind as u32, None);
        result
    }

    #[inline]
    pub(super) fn into_untyped(&self) -> Rc<UntypedBuffer> {
        Rc::clone(&self.buffer)
    }
}

impl Drop for UntypedBuffer {
    fn drop(&mut self) {
        self.ctx.delete_buffer(Some(&self.webgl_buffer));
    }
}

unsafe impl UnsafeBufferable for f32 {
    unsafe fn buffer_to_unchecked<T>(buffer: &mut Buffer<T>, data: &[f32], len: usize) {
        buffer.buffer.len.set(len);
        buffer.bind(|| {
            let view = js_sys::Float32Array::view(data);
            // NOTE no memory can be allocated here or `view` will be invalidated
            buffer.buffer.ctx.buffer_data_with_array_buffer_view(
                buffer.buffer.kind as u32,
                &view,
                buffer.buffer.usage as u32,
            );
        });
    }
}

impl Bufferable for f32 {
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]) {
        // SAFETY the unchecked `impl` does precisely what we would have done here
        unsafe {
            f32::buffer_to_unchecked(buffer, data, data.len());
        }
    }
}

impl<T, U> Bufferable for Vector2D<T, U>
where
    T: UnsafeBufferable,
{
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]) {
        // SAFETY we can always view a slice of `Vector2D`s as a slice of `T`s as `Vector2D` is
        // `#[repr(c)]`
        unsafe {
            let t_slice = slice::from_raw_parts(data.as_ptr() as *const T, data.len() * 2);
            T::buffer_to_unchecked(buffer, t_slice, data.len());
        }
    }
}

impl<T, U> Bufferable for Vector3D<T, U>
where
    T: UnsafeBufferable,
{
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]) {
        // SAFETY we can always view a slice of `Vector3D`s as a slice of `T`s as `Vector3D` is
        // `#[repr(c)]`
        unsafe {
            let t_slice = slice::from_raw_parts(data.as_ptr() as *const T, data.len() * 3);
            T::buffer_to_unchecked(buffer, t_slice, data.len());
        }
    }
}
