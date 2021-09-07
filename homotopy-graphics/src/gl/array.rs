use std::{collections::HashMap, rc::Rc};

use ultraviolet::{Vec2, Vec3};
use web_sys::{WebGl2RenderingContext, WebGlVertexArrayObject};

use super::{
    buffer::{Buffer, ElementBuffer, UntypedBuffer},
    shader::Program,
    GlError, Result,
};

#[macro_export]
macro_rules! vertex_array {
    ($program:expr, {$($attribute:ident : $value:expr),*$(,)*}) => {{
        $crate::gl::array::VertexArray::new($program)
            $(.map(|x| x.attribute(stringify!($attribute), $value)))*
    }};

    ($program:expr, $elements:expr, {$($attribute:ident : $value:expr),*$(,)*}) => {{
        $crate::gl::array::VertexArray::new_with_elements(
            $program,
            Some($elements),
        )$(.map(|x| x.attribute(stringify!($attribute), $value)))*
    }}
}

pub struct VertexArray {
    program: Program,

    attributes: HashMap<&'static str, Rc<UntypedBuffer>>,
    elements: Option<ElementBuffer>,
    len: usize,

    webgl_vao: WebGlVertexArrayObject,
}

impl VertexArray {
    pub fn new_with_elements(program: &Program, elements: Option<&ElementBuffer>) -> Result<Self> {
        let webgl_vao = program
            .ctx()
            .create_vertex_array()
            .ok_or(GlError::Allocate)?;

        Ok(Self {
            program: program.clone(),
            attributes: HashMap::new(),
            elements: elements.cloned(),
            len: 0,
            webgl_vao,
        })
    }

    #[inline]
    pub fn new(program: &Program) -> Result<Self> {
        Self::new_with_elements(program, None)
    }

    #[inline]
    pub(super) fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub(super) fn program(&self) -> &Program {
        &self.program
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
        self.program.ctx().bind_vertex_array(Some(&self.webgl_vao));
        let result = f();
        self.program.ctx().bind_vertex_array(None);
        result
    }
}

impl VertexArray {
    pub fn attribute<T>(mut self, attribute: &'static str, src: &Buffer<T>) -> Self
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

        // get the location of the target attribute
        let loc = self.program.attribute_loc(attribute);
        // bind the VAO
        self.bind(|| {
            // bind the source buffer
            src.bind(|| {
                // enable the specified attribute array
                self.program.ctx().enable_vertex_attrib_array(loc);
                // pass on the dimension and type information of the buffer
                // TODO(@doctorn) stride and offset? (probably not...)
                self.program.ctx().vertex_attrib_pointer_with_i32(
                    loc,
                    T::DIMENSION,
                    T::TYPE,
                    false,
                    0,
                    0,
                );
            });
        });

        // set the length of the buffer to match the length of the source data (this will be a
        // no-op unless the array is uninitialised)
        self.len = src.len();
        // hold a reference to the source data to stop it being dropped
        self.attributes.insert(attribute, src.as_untyped());

        self
    }
}

impl Drop for VertexArray {
    #[inline]
    fn drop(&mut self) {
        self.program
            .ctx()
            .delete_vertex_array(Some(&self.webgl_vao));
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

unsafe impl Attributable for Vec2 {
    const DIMENSION: i32 = 2;
    const TYPE: u32 = WebGl2RenderingContext::FLOAT;
}

unsafe impl Attributable for Vec3 {
    const DIMENSION: i32 = 3;
    const TYPE: u32 = WebGl2RenderingContext::FLOAT;
}
