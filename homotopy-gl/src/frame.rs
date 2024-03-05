use std::ops::Deref;

use homotopy_common::hash::FastHashMap;
use ultraviolet::Vec4;
use web_sys::WebGl2RenderingContext;

use super::{
    array::VertexArray,
    framebuffer::Framebuffer,
    shader::{Program, Uniformable},
    texture::Texture,
    GlCtx,
};

#[macro_export]
macro_rules! draw {
    (
        $program:expr,
        $vao:expr,
        $textures:expr,
        {$($uniform:ident : $value:expr),*$(,)*}
    ) => {{
        $crate::frame::Draw::new($program, $vao, $textures)
            $(.uniform(stringify!($uniform), $value))*
    }};
    (
        $program:expr,
        $vao:expr,
        $textures:expr,
        $depth:expr,
        {$($uniform:ident : $value:expr),*$(,)*}
    ) => {{
        $crate::frame::Draw::new_with_depth($program, $vao, $textures, $depth)
            $(.uniform(stringify!($uniform), $value))*
    }};
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DepthTest {
    Enable,
    Disable,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Clear {
    None,
    Depth,
    Color,
    ColorDepth,
}

struct FrameOpts<'a> {
    clear: Clear,
    clear_color: Vec4,
    framebuffer: Option<&'a Framebuffer>,
}

impl<'a> Default for FrameOpts<'a> {
    #[inline]
    fn default() -> Self {
        Self {
            clear: Clear::ColorDepth,
            clear_color: Vec4::new(0., 0., 0., 1.),
            framebuffer: None,
        }
    }
}

pub struct Frame<'a> {
    ctx: &'a mut GlCtx,
    opts: FrameOpts<'a>,
    draws: Vec<Draw<'a>>,
}

pub struct Draw<'a> {
    program: &'a Program,
    vertex_array: &'a VertexArray,
    depth_test: DepthTest,
    uniforms: FastHashMap<&'static str, Box<dyn Uniformable>>,
    textures: Vec<&'a Texture>,
}

impl<'a> Draw<'a> {
    #[inline]
    #[must_use]
    pub fn new_with_depth(
        program: &'a Program,
        vertex_array: &'a VertexArray,
        textures: &[&'a Texture],
        depth_test: DepthTest,
    ) -> Self {
        Self {
            program,
            vertex_array,
            depth_test,
            uniforms: Default::default(),
            textures: textures.to_owned(),
        }
    }

    #[inline]
    #[must_use]
    pub fn new(
        program: &'a Program,
        vertex_array: &'a VertexArray,
        textures: &[&'a Texture],
    ) -> Self {
        Self::new_with_depth(program, vertex_array, textures, DepthTest::Enable)
    }

    #[inline]
    #[must_use]
    pub fn uniform<T>(mut self, name: &'static str, t: T) -> Self
    where
        T: Uniformable,
    {
        assert!(self.program.has_uniform(name));
        self.uniforms
            .insert(name, Box::new(t) as Box<dyn Uniformable>);
        self
    }
}

impl<'a> Frame<'a> {
    #[inline]
    pub fn new(ctx: &'a mut GlCtx) -> Self {
        Self {
            ctx,
            opts: Default::default(),
            draws: vec![],
        }
    }

    #[inline]
    #[must_use]
    pub fn with_clear_color(mut self, color: Vec4) -> Self {
        self.opts.clear_color = color;
        self
    }

    #[inline]
    #[must_use]
    pub fn with_clear_opts(mut self, clear: Clear) -> Self {
        self.opts.clear = clear;
        self
    }

    #[inline]
    #[must_use]
    pub fn with_frame_buffer(mut self, framebuffer: &'a Framebuffer) -> Self {
        self.opts.framebuffer = Some(framebuffer);
        self
    }

    #[inline]
    pub fn draw(&mut self, draw: Draw<'a>) {
        self.draws.push(draw);
    }

    fn render_with_framebuffer(&mut self) {
        if let Some(framebuffer) = self.opts.framebuffer {
            framebuffer.bind(|| {
                self.render();
            });
        } else {
            self.render();
        }
    }

    fn render(&mut self) {
        self.ctx.resize_to_fit().unwrap();
        self.ctx.with_gl(|gl| {
            let clear_opts = match self.opts.clear {
                Clear::None => 0,
                Clear::Color => WebGl2RenderingContext::COLOR_BUFFER_BIT,
                Clear::Depth => WebGl2RenderingContext::DEPTH_BUFFER_BIT,
                Clear::ColorDepth => {
                    WebGl2RenderingContext::COLOR_BUFFER_BIT
                        | WebGl2RenderingContext::DEPTH_BUFFER_BIT
                }
            };

            gl.clear_color(
                self.opts.clear_color.x,
                self.opts.clear_color.y,
                self.opts.clear_color.z,
                self.opts.clear_color.w,
            );
            gl.clear(clear_opts);

            for draw in &self.draws {
                match draw.depth_test {
                    DepthTest::Enable => gl.enable(WebGl2RenderingContext::DEPTH_TEST),
                    DepthTest::Disable => gl.disable(WebGl2RenderingContext::DEPTH_TEST),
                }

                for (i, &texture) in draw.textures.iter().enumerate() {
                    texture.activate(i);
                }

                // bind the program the draw expected
                // NOTE we could sort each draw queue by program to make this more performant
                draw.program.bind(|| {
                    // bind the vertex array we're drawing
                    draw.vertex_array.bind(|| {
                        // set all of the uniforms
                        for (name, loc) in draw.program.uniforms() {
                            let Some(data) = draw.uniforms.get(name) else {
                                // an unset uniform is a programmer error, so just panic
                                panic!("uniform '{name}' is unset");
                            };

                            data.uniform(self.ctx, loc);
                        }

                        if let Some(elements) = draw.vertex_array.elements() {
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
        self.render_with_framebuffer();
    }
}
