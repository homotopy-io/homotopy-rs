use web_sys::WebGl2RenderingContext;

use super::array::VertexArray;
use super::shader::Program;
use super::GraphicsCtx;

pub struct Frame<'a> {
    ctx: WebGl2RenderingContext,
    draws: Vec<Draw<'a>>,
}

pub struct Draw<'a> {
    program: &'a Program,
    vertex_array: &'a VertexArray,
    // TODO(@doctorn) all of the uniforms
}

impl<'a> Draw<'a> {
    #[inline]
    pub fn new(program: &'a Program, vertex_array: &'a VertexArray) -> Self {
        Self {
            program,
            vertex_array,
        }
    }
}

impl<'a> Frame<'a> {
    pub fn new(ctx: &GraphicsCtx) -> Self {
        Self {
            ctx: ctx.webgl_ctx.clone(),
            draws: vec![],
        }
    }

    #[inline]
    pub fn draw(&mut self, draw: Draw<'a>) {
        self.draws.push(draw);
    }

    pub fn render(self) {
        // TODO(@doctorn) this should be customisable
        self.ctx.clear_color(0.0, 0.0, 1.0, 1.0);
        self.ctx.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        for draw in self.draws.iter() {
            draw.program.bind(|| {
                draw.vertex_array.bind(|| {
                    self.ctx.draw_arrays(
                        WebGl2RenderingContext::TRIANGLES,
                        0,
                        3, // FIXME(@doctorn) this should be the number of vertices
                    );
                })
            });
        }

        self.ctx.flush();
    }
}
