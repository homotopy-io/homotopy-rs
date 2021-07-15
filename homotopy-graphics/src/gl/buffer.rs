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

    len: usize,

    webgl_buffer: WebGlBuffer,
}

#[derive(Clone)]
pub struct Buffer<T> {
    buffer: Rc<UntypedBuffer>,
    _phantom: PhantomData<T>,
}

pub type ElementBuffer = Buffer<u16>;

impl UntypedBuffer {
    #[inline(always)]
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

impl Drop for UntypedBuffer {
    #[inline]
    fn drop(&mut self) {
        self.ctx.delete_buffer(Some(&self.webgl_buffer));
    }
}

impl GlCtx {
    pub fn mk_buffer_with_kind_and_usage<T>(
        &self,
        kind: BufferKind,
        usage: BufferUsage,
        data: &[T],
    ) -> Result<Buffer<T>>
    where
        T: Bufferable,
    {
        let mut buffer = Buffer::alloc(self, kind, usage, data.len())?;
        buffer.buffer(data);
        Ok(buffer)
    }

    pub unsafe fn mk_buffer_with_kind_and_usage_unchecked<T, U>(
        &self,
        kind: BufferKind,
        usage: BufferUsage,
        data: &[U],
        len: usize,
    ) -> Result<Buffer<T>>
    where
        U: UnsafeBufferable,
    {
        let mut buffer = Buffer::alloc(self, kind, usage, len)?;
        buffer.buffer_unchecked(data);
        Ok(buffer)
    }

    #[inline]
    pub fn mk_buffer<T>(&self, data: &[T]) -> Result<Buffer<T>>
    where
        T: Bufferable,
    {
        self.mk_buffer_with_kind_and_usage(Default::default(), Default::default(), data)
    }

    #[inline]
    pub unsafe fn mk_buffer_unchecked<T, U>(&self, data: &[U], len: usize) -> Result<Buffer<T>>
    where
        U: UnsafeBufferable,
    {
        self.mk_buffer_with_kind_and_usage_unchecked(
            Default::default(),
            Default::default(),
            data,
            len,
        )
    }

    #[inline]
    pub fn mk_element_buffer(&self, data: &[u16]) -> Result<ElementBuffer> {
        self.mk_buffer_with_kind_and_usage(BufferKind::ElementArray, BufferUsage::StaticDraw, data)
    }
}

impl<T> Buffer<T>
where
    T: Bufferable,
{
    #[inline]
    pub fn buffer(&mut self, data: &[T]) {
        assert_eq!(self.buffer.len, data.len());
        T::buffer_to(self, data);
    }
}

impl<T> Buffer<T> {
    #[inline]
    pub unsafe fn buffer_unchecked<U>(&mut self, data: &[U])
    where
        U: UnsafeBufferable,
    {
        U::buffer_to_unchecked(self, data);
    }
}

impl<T> Buffer<T> {
    fn alloc(ctx: &GlCtx, kind: BufferKind, usage: BufferUsage, len: usize) -> Result<Self> {
        let webgl_buffer = ctx.webgl_ctx.create_buffer().ok_or(GlError::Allocate)?;
        let untyped_buffer = UntypedBuffer {
            ctx: ctx.webgl_ctx.clone(),
            kind,
            usage,
            len,
            webgl_buffer,
        };

        let buffer = Buffer {
            buffer: Rc::new(untyped_buffer),
            _phantom: Default::default(),
        };

        Ok(buffer)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len
    }

    #[inline]
    pub(super) fn into_untyped(&self) -> Rc<UntypedBuffer> {
        Rc::clone(&self.buffer)
    }

    #[inline(always)]
    pub(super) fn bind<F, U>(&self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        self.buffer.bind(f)
    }
}

pub unsafe trait UnsafeBufferable: Sized {
    unsafe fn buffer_to_unchecked<T>(buffer: &mut Buffer<T>, data: &[Self]);
}

pub trait Bufferable: Sized {
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]);
}

unsafe impl UnsafeBufferable for f32 {
    unsafe fn buffer_to_unchecked<T>(buffer: &mut Buffer<T>, data: &[f32]) {
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

unsafe impl UnsafeBufferable for u16 {
    unsafe fn buffer_to_unchecked<T>(buffer: &mut Buffer<T>, data: &[u16]) {
        buffer.bind(|| {
            let view = js_sys::Uint16Array::view(data);
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
        assert_eq!(buffer.len(), data.len());
        // SAFETY the unchecked `impl` does precisely what we would have done here
        unsafe {
            f32::buffer_to_unchecked(buffer, data);
        }
    }
}

impl Bufferable for u16 {
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]) {
        assert_eq!(buffer.len(), data.len());
        // SAFETY the unchecked `impl` does precisely what we would have done here
        unsafe {
            u16::buffer_to_unchecked(buffer, data);
        }
    }
}

impl<T, U> Bufferable for Vector2D<T, U>
where
    T: UnsafeBufferable,
{
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]) {
        assert_eq!(buffer.len(), data.len());
        // SAFETY we can always view a slice of `Vector2D`s as a slice of `T`s as `Vector2D` is
        // `#[repr(c)]`
        unsafe {
            let t_slice = slice::from_raw_parts(data.as_ptr() as *const T, data.len() * 2);
            T::buffer_to_unchecked(buffer, t_slice);
        }
    }
}

impl<T, U> Bufferable for Vector3D<T, U>
where
    T: UnsafeBufferable,
{
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]) {
        assert_eq!(buffer.len(), data.len());
        // SAFETY we can always view a slice of `Vector3D`s as a slice of `T`s as `Vector3D` is
        // `#[repr(c)]`
        unsafe {
            let t_slice = slice::from_raw_parts(data.as_ptr() as *const T, data.len() * 3);
            T::buffer_to_unchecked(buffer, t_slice);
        }
    }
}
