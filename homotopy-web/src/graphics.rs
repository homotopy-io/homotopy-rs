use yew::prelude::*;

use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use thiserror::Error;

use homotopy_core::idx::{Idx, IdxVec};

pub mod buffer;
pub mod frame;
pub mod geom;
pub mod shader;
pub mod vertex_array;

use self::buffer::{VertexBuffer, VertexBufferData};
use self::shader::{FragmentShader, Program, ProgramData, ShaderData, VertexShader};
use self::vertex_array::{VertexArray, VertexArrayData};

#[derive(Error, Debug)]
pub enum GraphicsError {
    #[error("failed to attach to WebGL context")]
    Attachment(&'static str),
    #[error("failed to allocate WebGL object")]
    Allocate,
    #[error("failed to compile shader: {0}")]
    ShaderCompile(String),
    #[error("failed to link shader program: {0}")]
    ProgramLink(String),
}

pub type Result<T> = std::result::Result<T, GraphicsError>;

trait GraphicsObject: Idx {
    type Data;
    type Carrier;

    fn alloc_carrier(ctx: &mut GraphicsCtx) -> Result<Self::Carrier>;

    fn dealloc_carrier(self, ctx: &mut GraphicsCtx);

    fn get_data(self, ctx: &GraphicsCtx) -> &Self::Data;

    fn get_carrier(self, ctx: &GraphicsCtx) -> &Self::Carrier;
}

trait Bindable: GraphicsObject {
    fn bind(self, ctx: &GraphicsCtx);

    fn release(self, ctx: &GraphicsCtx);
}

pub struct GraphicsCtx {
    webgl_ctx: WebGl2RenderingContext,
    vert_shaders: IdxVec<VertexShader, ShaderData>,
    frag_shaders: IdxVec<FragmentShader, ShaderData>,
    programs: IdxVec<Program, ProgramData>,
    vertex_buffers: IdxVec<VertexBuffer, VertexBufferData>,
    vertex_arrays: IdxVec<VertexArray, VertexArrayData>,
}

impl GraphicsCtx {
    pub fn attach(node_ref: NodeRef) -> Result<Self> {
        let canvas = node_ref.cast::<HtmlCanvasElement>().ok_or_else(|| {
            GraphicsError::Attachment("supplied node ref does not point to a canvas element")
        })?;

        let webgl_ctx = if let Ok(Some(obj)) = canvas.get_context("webgl2") {
            obj.dyn_into::<WebGl2RenderingContext>().map_err(|_| {
                GraphicsError::Attachment("failed to cast WebGL context to a rendering context")
            })?
        } else {
            return Err(GraphicsError::Attachment(
                "failed to get WebGL context for canvas",
            ));
        };

        Ok(Self {
            webgl_ctx,
            vert_shaders: IdxVec::new(),
            frag_shaders: IdxVec::new(),
            programs: IdxVec::new(),
            vertex_buffers: IdxVec::new(),
            vertex_arrays: IdxVec::new(),
        })
    }

    #[inline]
    fn alloc<T>(&mut self) -> Result<T::Carrier>
    where
        T: GraphicsObject,
    {
        T::alloc_carrier(self)
    }

    #[inline]
    fn drop<T>(&mut self, t: T)
    where
        T: GraphicsObject,
    {
        t.dealloc_carrier(self);
    }

    #[inline]
    fn carrier_for<T>(&self, t: T) -> &T::Carrier
    where
        T: GraphicsObject,
    {
        t.get_carrier(self)
    }

    #[inline]
    fn get<T>(&self, t: T) -> &T::Data
    where
        T: GraphicsObject,
    {
        t.get_data(self)
    }

    #[inline]
    fn bind<T, F, U>(&self, t: T, f: F) -> U
    where
        T: Bindable,
        F: FnOnce() -> U,
    {
        // Bind object
        t.bind(self);
        // Compute
        let result = f();
        // Release object
        t.release(self);
        // Forward result
        result
    }
}

impl Drop for GraphicsCtx {
    fn drop(&mut self) {}
}
