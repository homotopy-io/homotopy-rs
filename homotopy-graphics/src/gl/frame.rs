use std::collections::HashMap;

use web_sys::WebGl2RenderingContext;

use super::array::VertexArray;
use super::shader::Uniformable;
use super::GlCtx;

#[macro_export]
macro_rules! draw {
    ($vao:expr, {$($uniform:ident : $value:expr),*$(,)*}) => {{
        $crate::gl::frame::Draw::new($vao)
            $(.uniform(stringify!($uniform), $value))*
    }};
}

pub struct Frame<'a> {
    ctx: WebGl2RenderingContext,
    draws: Vec<Draw<'a>>,
}

pub struct Draw<'a> {
    vertex_array: &'a VertexArray,
    uniforms: HashMap<&'static str, Box<dyn Uniformable>>,
}

impl<'a> Draw<'a> {
    #[inline]
    pub fn new(vertex_array: &'a VertexArray) -> Self {
        Self {
            vertex_array,
            uniforms: HashMap::new(),
        }
    }

    #[inline]
    pub fn uniform<T>(mut self, name: &'static str, t: T) -> Self
    where
        T: Uniformable,
    {
        assert!(self.vertex_array.program().has_uniform(name));
        self.uniforms
            .insert(name, Box::new(t) as Box<dyn Uniformable>);
        self
    }
}

impl<'a> Frame<'a> {
    pub fn new(ctx: &GlCtx) -> Self {
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

        // TODO(@doctorn) multiple draw queues (alpha channel etc.)
        for draw in &self.draws {
            // bind the program the draw expected
            // NOTE we could sort each draw queue by program to make this much more performant
            draw.vertex_array.program().bind(|| {
                // bind the vertex array we're drawing
                draw.vertex_array.bind(|| {
                    // set all of the uniforms
                    for (name, loc) in draw.vertex_array.program().uniforms() {
                        let data = draw
                            .uniforms
                            .get(name)
                            .expect(&format!("uniform '{}' is unset", name));
                        data.uniform(&self.ctx, loc);
                    }

                    if let Some(elements) = draw.vertex_array.elements() {
                        // if we're given an element buffer, bind it and draw the appropriate
                        // number of elements
                        elements.bind(|| {
                            self.ctx.draw_elements_with_i32(
                                WebGl2RenderingContext::TRIANGLES,
                                elements.len() as i32,
                                WebGl2RenderingContext::UNSIGNED_SHORT,
                                0, // TODO(@doctorn) offset? (probably not...)
                            );
                        })
                    } else {
                        // if no element buffer was provided, assume we're just drawing an array of
                        // triangles
                        self.ctx.draw_arrays(
                            WebGl2RenderingContext::TRIANGLES,
                            0,
                            draw.vertex_array.len() as i32,
                        );
                    }
                })
            });
        }

        self.ctx.flush();
    }
}
