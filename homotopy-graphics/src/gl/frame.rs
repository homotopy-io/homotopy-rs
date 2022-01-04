use std::{collections::HashMap, ops::Deref};

use web_sys::WebGl2RenderingContext;

use super::{array::VertexArray, buffer::ElementKind, shader::Uniformable, GlCtx};

#[macro_export]
macro_rules! draw {
    ($vao:expr, {$($uniform:ident : $value:expr),*$(,)*}) => {{
        $crate::gl::frame::Draw::new($vao)
            $(.uniform(stringify!($uniform), $value))*
    }};
}

pub struct Frame<'a> {
    ctx: &'a mut GlCtx,
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
    pub fn new(ctx: &'a mut GlCtx) -> Self {
        Self { ctx, draws: vec![] }
    }

    #[inline]
    pub fn draw(&mut self, draw: Draw<'a>) {
        self.draws.push(draw);
    }

    fn render(&mut self) {
        self.ctx.resize_to_fit();
        self.ctx.with_gl(|gl| {
            gl.clear_color(
                self.clear_color.x,
                self.clear_color.y,
                self.clear_color.z,
                1.0,
            );
            gl.clear(
                WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
            );

            for draw in &self.draws {
                // bind the program the draw expected
                // NOTE we could sort each draw queue by program to make this much more performant
                draw.vertex_array.program().bind(|| {
                    // bind the vertex array we're drawing
                    draw.vertex_array.bind(|| {
                        // set all of the uniforms
                        for (name, loc) in draw.vertex_array.program().uniforms() {
                            let data = if let Some(data) = draw.uniforms.get(name) {
                                data
                            } else {
                                // an unset uniform is a programmer error, so just panic
                                panic!("uniform '{}' is unset", name);
                            };

                            data.uniform(self.ctx, loc);
                        }

                        if let Some(elements) = draw.vertex_array.elements() {
                            // disable depth testing for lines
                            if elements.kind == ElementKind::Lines {
                                gl.disable(WebGl2RenderingContext::DEPTH_TEST);
                            }
                            // if we're given an element buffer, bind it and draw the appropriate
                            // number of elements

                            elements.buffer.bind(|| {
                                gl.draw_elements_with_i32(
                                    elements.kind as u32,
                                    elements.buffer.len() as i32,
                                    WebGl2RenderingContext::UNSIGNED_SHORT,
                                    0,
                                );
                            });
                            // re-enable depth testing
                            if elements.kind == ElementKind::Lines {
                                gl.enable(WebGl2RenderingContext::DEPTH_TEST);
                            }
                        } else {
                            // if no element buffer was provided, assume we're just drawing an array of
                            // triangles
                            gl.draw_arrays(
                                WebGl2RenderingContext::TRIANGLES,
                                0,
                                draw.vertex_array.len() as i32,
                            );
                        }
                    });
                });
            }

            gl.flush();
        });
    }
}

impl<'a> Deref for Frame<'a> {
    type Target = GlCtx;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl<'a> Drop for Frame<'a> {
    fn drop(&mut self) {
        self.render();
    }
}
