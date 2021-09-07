use homotopy_graphics::{
    clay, draw,
    gl::{array::VertexArray, frame::Frame, shader::Program, GlCtx, Result},
    program, vertex_array,
};
use ultraviolet::{projection::rh_yup::perspective_gl, Mat4, Vec3};

use crate::components::gl::Renderer;

pub struct Diagram3D {
    meshes: [VertexArray; 4],
    t: f32,
}

impl Renderer for Diagram3D {
    fn init(ctx: &mut GlCtx) -> Result<Self> {
        let program = program!(
            ctx,
            "../../../glsl/vert.glsl",
            "../../../glsl/frag.glsl",
            { position, normal },
            { m, m_inv, mvp },
        )?;

        Ok(Self {
            meshes: [
                Self::example(ctx, &program, 1)?,
                Self::example(ctx, &program, 2)?,
                Self::example(ctx, &program, 3)?,
                Self::example(ctx, &program, 4)?,
            ],
            t: 0.0,
        })
    }

    fn update(&mut self, dt: f32) {
        self.t += 0.001 * dt;
        while self.t >= std::f32::consts::TAU {
            self.t -= std::f32::consts::TAU;
        }
    }

    fn render<'a>(&'a self, mut frame: Frame<'a>) {
        let p = perspective_gl(f32::to_radians(30.0), frame.aspect_ratio(), 0.5, 10.0);
        for i in 0..4 {
            let m = Mat4::from_translation(Vec3::new(-2.25 + 1.5 * i as f32, 0.0, -7.0))
                * Mat4::from_rotation_y(self.t)
                * Mat4::from_scale(0.8);
            let m_inv = m.inversed();
            let mvp = p * m;

            frame.draw(draw! {
                &self.meshes[i],
                {
                    m: m,
                    m_inv: m_inv,
                    mvp: mvp,
                }
            });
        }
    }
}

impl Diagram3D {
    fn example(ctx: &mut GlCtx, program: &Program, depth: u8) -> Result<VertexArray> {
        let buffers = clay::subdivision::subdivide_3(clay::examples::example_3().into(), depth)
            .buffer(ctx)?;

        vertex_array!(
            program,
            &buffers.element_buffer,
            {
                position: &buffers.vertex_buffer,
                normal: &buffers.normal_buffer,
            }
        )
    }
}
