use std::{marker::PhantomData, rc::Rc, slice};

use js_sys;
use ultraviolet::{Vec2, Vec3, Vec4};
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
        Self::Array
    }
}

impl Default for BufferUsage {
    fn default() -> Self {
        Self::StaticDraw
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ElementKind {
    Lines = WebGl2RenderingContext::LINES as isize,
    Triangles = WebGl2RenderingContext::TRIANGLES as isize,
}

#[derive(Clone)]
pub struct ElementBuffer {
    pub(super) buffer: Buffer<u16>,
    pub(super) kind: ElementKind,
}

impl UntypedBuffer {
    #[inline]
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

    /// # Safety
    ///
    /// Assumes that when buffered `data` will have length `len`.
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

    /// # Safety
    ///
    /// Assumes that when buffered `data` will have length `len`.
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
    pub fn mk_element_buffer(&self, data: &[u16], kind: ElementKind) -> Result<ElementBuffer> {
        let buffer = self.mk_buffer_with_kind_and_usage(
            BufferKind::ElementArray,
            BufferUsage::StaticDraw,
            data,
        )?;

        Ok(ElementBuffer { buffer, kind })
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
    /// # Safety
    ///
    /// Assumes that when buffered `data` will have length `self.len()`.
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

        let buffer = Self {
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
    pub fn is_empty(&self) -> bool {
        self.buffer.len == 0
    }

    #[inline]
    pub(super) fn as_untyped(&self) -> Rc<UntypedBuffer> {
        Rc::clone(&self.buffer)
    }

    #[inline]
    pub(super) fn bind<F, U>(&self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        self.buffer.bind(f)
    }
}

pub unsafe trait UnsafeBufferable: Sized {
    /// # Safety
    ///
    /// Assumes that when buffered `data` will have length `buffer.len()`.
    unsafe fn buffer_to_unchecked<T>(buffer: &mut Buffer<T>, data: &[Self]);
}

pub trait Bufferable: Sized {
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]);
}

unsafe impl UnsafeBufferable for f32 {
    unsafe fn buffer_to_unchecked<T>(buffer: &mut Buffer<T>, data: &[Self]) {
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
    unsafe fn buffer_to_unchecked<T>(buffer: &mut Buffer<T>, data: &[Self]) {
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
            Self::buffer_to_unchecked(buffer, data);
        }
    }
}

impl Bufferable for u16 {
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]) {
        assert_eq!(buffer.len(), data.len());
        // SAFETY the unchecked `impl` does precisely what we would have done here
        unsafe {
            Self::buffer_to_unchecked(buffer, data);
        }
    }
}

impl Bufferable for Vec2 {
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]) {
        assert_eq!(buffer.len(), data.len());
        // SAFETY we can always view a slice of `Vector2D`s as a slice of `T`s as `Vec2` is
        // `#[repr(c)]`
        unsafe {
            let t_slice = slice::from_raw_parts(data.as_ptr().cast::<f32>(), data.len() * 2);
            f32::buffer_to_unchecked(buffer, t_slice);
        }
    }
}

impl Bufferable for Vec3 {
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]) {
        assert_eq!(buffer.len(), data.len());
        // SAFETY we can always view a slice of `Vector3D`s as a slice of `T`s as `Vec3` is
        // `#[repr(c)]`
        unsafe {
            let t_slice = slice::from_raw_parts(data.as_ptr().cast::<f32>(), data.len() * 3);
            f32::buffer_to_unchecked(buffer, t_slice);
        }
    }
}

impl Bufferable for Vec4 {
    fn buffer_to(buffer: &mut Buffer<Self>, data: &[Self]) {
        assert_eq!(buffer.len(), data.len());
        // SAFETY we can always view a slice of `Vector3D`s as a slice of `T`s as `Vec4` is
        // `#[repr(c)]`
        unsafe {
            let t_slice = slice::from_raw_parts(data.as_ptr().cast::<f32>(), data.len() * 4);
            f32::buffer_to_unchecked(buffer, t_slice);
        }
    }
}
