use homotopy_graphics::{
    clay, draw,
    gl::{array::VertexArray, frame::Frame, GlCtx, Result},
    program, vertex_array,
};
use ultraviolet::{projection::rh_yup::perspective_gl, Mat4, Vec3};

use crate::components::gl::Renderer;

pub struct Diagram3D {
    mesh: VertexArray,
    t: f32,
}

impl Renderer for Diagram3D {
    fn init(ctx: &mut GlCtx) -> Result<Self> {
        let (elements, vertices) = clay::examples::example_3().into().buffer(ctx)?;
        let program = program!(
            ctx,
            "../../../glsl/vert.glsl",
            "../../../glsl/frag.glsl",
            { position },
            { mvp },
        )?;
        let mesh = vertex_array!(&program, &elements, { position: &vertices })?;

        Ok(Self { mesh, t: 0.0 })
    }

    fn update(&mut self, dt: f32) {
        self.t += 0.001 * dt;
        while self.t >= std::f32::consts::TAU {
            self.t -= std::f32::consts::TAU;
        }
    }

    fn render<'a>(&'a self, mut frame: Frame<'a>) {
        let mvp = perspective_gl(f32::to_radians(30.0), 1.0, 0.5, 10.0)
            * Mat4::from_translation(Vec3::new(0.0, 0.0, -7.0))
            * Mat4::from_rotation_y(self.t);

        frame.draw(draw! {
            &self.mesh,
            { mvp: mvp }
        });
    }
}
