use std::rc::Rc;

use ultraviolet::{Vec2, Vec3, Vec4};
use web_sys::{WebGl2RenderingContext, WebGlVertexArrayObject};

use super::{
    buffer::{Buffer, ElementBuffer, UntypedBuffer},
    GlCtx, GlCtxHandle, GlError, Result,
};

pub const VAO_LIMIT: usize = 0x0001_0000;

#[macro_export]
macro_rules! vertex_array {
    ($ctx:expr, [$($value:expr),*$(,)*]) => {{
        $crate::gl::array::VertexArray::new($ctx)
            $(.map(|x| x.attribute($value)))*
    }};

    ($ctx:expr, $elements:expr, [$($value:expr),*$(,)*]) => {{
        $crate::gl::array::VertexArray::new_with_elements(
            $ctx,
            Some($elements),
        )$(.map(|x| x.attribute($value)))*
    }}
}

pub struct VertexArray {
    ctx: GlCtxHandle,

    attributes: Vec<Rc<UntypedBuffer>>,
    elements: Option<ElementBuffer>,
    len: usize,

    webgl_vao: WebGlVertexArrayObject,
}

impl VertexArray {
    pub fn new_with_elements(ctx: &GlCtx, elements: Option<&ElementBuffer>) -> Result<Self> {
        let webgl_vao = ctx
            .with_gl(WebGl2RenderingContext::create_vertex_array)
            .ok_or(GlError::Allocate)?;

        Ok(Self {
            ctx: ctx.ctx_handle(),
            attributes: Default::default(),
            elements: elements.cloned(),
            len: 0,
            webgl_vao,
        })
    }

    #[inline]
    pub fn new(ctx: &GlCtx) -> Result<Self> {
        Self::new_with_elements(ctx, None)
    }

    #[inline]
    pub(super) fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub(super) fn elements(&self) -> Option<&ElementBuffer> {
        self.elements.as_ref()
    }

    #[inline]
    pub(super) fn bind<F, U>(&self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        self.ctx.with_gl(|gl| {
            gl.bind_vertex_array(Some(&self.webgl_vao));
            let result = f();
            gl.bind_vertex_array(None);
            result
        })
    }

    #[must_use]
    pub fn attribute<T>(mut self, src: &Buffer<T>) -> Self
    where
        T: Attributable,
    {
        if !self.attributes.is_empty() {
            assert_eq!(
                self.len,
                src.len(),
                "buffer does not match length of vertex array"
            );
        }

        assert!(src.len() <= VAO_LIMIT, "buffer exceeds maximum VAO size");

        // get the next attribute location
        let loc = self.attributes.len() as u32;
        // bind the VAO
        self.bind(|| {
            // bind the source buffer
            src.bind(|| {
                self.ctx.with_gl(|gl| {
                    // enable the specified attribute array
                    gl.enable_vertex_attrib_array(loc);
                    // pass on the dimension and type information of the buffer
                    gl.vertex_attrib_pointer_with_i32(loc, T::DIMENSION, T::TYPE, false, 0, 0);
                });
            });
        });

        // set the length of the buffer to match the length of the source data (this will be a
        // no-op unless the array is uninitialised)
        self.len = src.len();
        // hold a reference to the source data to stop it being dropped
        self.attributes.push(src.as_untyped());

        self
    }
}

impl<'ctx> Drop for VertexArray {
    #[inline]
    fn drop(&mut self) {
        self.ctx.with_gl(|gl| {
            gl.delete_vertex_array(Some(&self.webgl_vao));
        });
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe trait Attributable {
    const DIMENSION: i32;
    const TYPE: u32;
}

unsafe impl Attributable for f32 {
    const DIMENSION: i32 = 1;
    const TYPE: u32 = WebGl2RenderingContext::FLOAT;
}

unsafe impl Attributable for Vec2 {
    const DIMENSION: i32 = 2;
    const TYPE: u32 = WebGl2RenderingContext::FLOAT;
}

unsafe impl Attributable for Vec3 {
    const DIMENSION: i32 = 3;
    const TYPE: u32 = WebGl2RenderingContext::FLOAT;
}

unsafe impl Attributable for Vec4 {
    const DIMENSION: i32 = 4;
    const TYPE: u32 = WebGl2RenderingContext::FLOAT;
}
