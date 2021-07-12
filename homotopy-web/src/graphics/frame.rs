use web_sys::WebGl2RenderingContext;

use super::shader::Program;
use super::vertex_array::VertexArray;
use super::GraphicsCtx;

pub struct Frame<'ctx> {
    ctx: &'ctx GraphicsCtx,
    draws: Vec<Draw>,
}

pub struct Draw {
    program: Program,
    vertex_array: VertexArray,
}

impl Draw {
    #[inline]
    pub fn new(program: Program, vertex_array: VertexArray) -> Self {
        Self {
            program,
            vertex_array,
        }
    }
}

impl<'ctx> Frame<'ctx> {
    #[inline]
    pub fn draw(&mut self, draw: Draw) {
        self.draws.push(draw);
    }

    pub fn render(self) {
        // TODO(@doctorn) this should be customisable
        self.ctx.webgl_ctx.clear_color(0.0, 0.0, 1.0, 1.0);
        self.ctx.webgl_ctx.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        for draw in self.draws.iter() {
            self.ctx.bind(draw.program, || {
                self.ctx.bind(draw.vertex_array, || {
                    self.ctx.webgl_ctx.draw_arrays(
                        WebGl2RenderingContext::TRIANGLES,
                        0,
                        3, // FIXME(@doctorn) this should be the number of vertices
                    );
                });
            });

            self.ctx.webgl_ctx.flush();
        }
    }
}

impl GraphicsCtx {
    #[inline]
    pub fn mk_frame<'ctx>(&'ctx self) -> Frame<'ctx> {
        Frame {
            ctx: self,
            draws: vec![],
        }
    }
}
