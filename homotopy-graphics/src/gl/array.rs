use std::rc::Rc;

use euclid::{Vector2D, Vector3D};

use web_sys::{WebGl2RenderingContext, WebGlVertexArrayObject};

use super::buffer::{Buffer, UntypedBuffer};
use super::{GlCtx, GlError, Result};

pub struct VertexArray {
    ctx: WebGl2RenderingContext,

    attributes: Vec<Rc<UntypedBuffer>>,
    len: usize,

    webgl_vao: WebGlVertexArrayObject,
}

impl VertexArray {
    pub fn new(ctx: &GlCtx) -> Result<Self> {
        let webgl_vao = ctx
            .webgl_ctx
            .create_vertex_array()
            .ok_or(GlError::Allocate)?;

        Ok(Self {
            ctx: ctx.webgl_ctx.clone(),
            attributes: vec![],
            len: 0,
            webgl_vao,
        })
    }

    #[inline(always)]
    pub(super) fn bind<F, U>(&self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        self.ctx.bind_vertex_array(Some(&self.webgl_vao));
        let result = f();
        self.ctx.bind_vertex_array(None);
        result
    }
}

impl VertexArray {
    // TODO(@doctorn) this definitely shouldn't be public
    pub fn attribute<T>(&mut self, loc: u32, src: &Buffer<T>)
    where
        T: Attributable,
    {
        // TODO(@doctorn) should be able to use locations other than 0,
        // but this is program dependent (need to work out how to support
        // this...)

        if !self.attributes.is_empty() {
            assert_eq!(
                self.len,
                src.len(),
                "buffer does not match length of vertex array"
            );
        }

        self.len = src.len();

        // bind the VAO
        self.bind(|| {
            // bind the source buffer
            src.bind(|| {
                // enable the specified attribute array
                self.ctx.enable_vertex_attrib_array(loc);
                // pass on the dimension and type information of the buffer
                // TODO(@doctorn) stride and offset? (probably not...)
                self.ctx
                    .vertex_attrib_pointer_with_i32(loc, T::DIMENSION, T::TYPE, false, 0, 0);
            });
        });

        self.attributes.push(src.into_untyped());
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        self.ctx.delete_vertex_array(Some(&self.webgl_vao));
    }
}

pub unsafe trait Attributable {
    const DIMENSION: i32;
    const TYPE: u32;
}

unsafe impl Attributable for f32 {
    const DIMENSION: i32 = 1;
    const TYPE: u32 = WebGl2RenderingContext::FLOAT;
}

unsafe impl<T, U> Attributable for Vector2D<T, U>
where
    T: Attributable,
{
    const DIMENSION: i32 = 2;
    const TYPE: u32 = T::TYPE;
}

unsafe impl<T, U> Attributable for Vector3D<T, U>
where
    T: Attributable,
{
    const DIMENSION: i32 = 3;
    const TYPE: u32 = T::TYPE;
}
