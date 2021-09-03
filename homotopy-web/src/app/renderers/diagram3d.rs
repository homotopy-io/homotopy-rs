use ultraviolet::projection::lh_yup::perspective_gl;

use homotopy_graphics::gl::array::VertexArray;
use homotopy_graphics::gl::frame::Frame;
use homotopy_graphics::gl::{GlCtx, Result};
use homotopy_graphics::{draw, program, vertex_array};

use crate::components::gl::Renderer;

pub struct Diagram3D {
    mesh: VertexArray,
}

impl Renderer for Diagram3D {
    fn init(ctx: &mut GlCtx) -> Result<Self> {
        let (elements, vertices) = homotopy_graphics::clay::examples::example_3()
            .into()
            .buffer(ctx)?;
        let program = program!(
            ctx,
            "../../../glsl/vert.glsl",
            "../../../glsl/frag.glsl",
            { position },
            { mvp },
        )?;
        let mesh = vertex_array!(&program, &elements, { position: &vertices })?;

        Ok(Self { mesh })
    }

    fn update(&mut self, _dt: f32) {}

    fn render<'a>(&'a self, mut frame: Frame<'a>) {
        let mvp = perspective_gl(30.0, 1.0, 0.5, 10.0);

        frame.draw(draw! {
            &self.mesh,
            { mvp: mvp }
        });
    }
}
