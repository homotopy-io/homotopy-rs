use web_sys::WebGlRenderingContext;

use super::shader::Program;
use super::vertex_buffer::VertexBuffer;
use super::GraphicsCtx;

pub struct Frame<'ctx> {
    ctx: &'ctx GraphicsCtx,
    draws: Vec<Draw>,
}

pub struct DrawBuilder<'ctx, 'a> {
    frame: &'a mut Frame<'ctx>,
    draw: Draw,
}

pub struct Draw {
    program: Option<Program>,
    vertex_buffer: VertexBuffer,
}

impl Draw {
    #[inline]
    pub fn new(vertex_buffer: VertexBuffer) -> Self {
        Self {
            program: None,
            vertex_buffer,
        }
    }
}

impl<'ctx, 'a> DrawBuilder<'ctx, 'a> {
    #[inline]
    pub fn with_program(mut self, program: Program) -> Self {
        self.draw.program = Some(program);
        self
    }

    #[inline]
    pub fn commit(self) {
        self.frame.draws.push(self.draw);
    }
}

impl<'ctx> Frame<'ctx> {
    #[inline]
    pub fn build_draw(&mut self, vertex_buffer: VertexBuffer) -> DrawBuilder<'ctx, '_> {
        DrawBuilder {
            frame: self,
            draw: Draw::new(vertex_buffer),
        }
    }

    pub fn render(self) {
        // TODO(@doctorn) this should be customisable
        self.ctx.webgl_ctx.clear_color(0.0, 0.0, 1.0, 1.0);
        self.ctx.webgl_ctx.clear(
            WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT,
        );

        for draw in self.draws.iter() {
            if let Some(program) = draw.program {
                self.ctx
                    .webgl_ctx
                    .use_program(Some(self.ctx.programs[program].underlying_program()));
            }

            self.ctx
                .bind(&self.ctx.vertex_buffers[draw.vertex_buffer], |_| {
                    self.ctx.webgl_ctx.enable_vertex_attrib_array(0);
                    self.ctx.webgl_ctx.draw_arrays(
                        WebGlRenderingContext::TRIANGLES,
                        0,
                        self.ctx.vertex_buffers[draw.vertex_buffer].len() as i32,
                    );
                });

            self.ctx.webgl_ctx.use_program(None);
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
