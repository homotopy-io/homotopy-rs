use euclid::Angle;

use yew::prelude::*;
use yew::services::render::RenderTask;
use yew::services::RenderService;

use homotopy_graphics::gl::array::VertexArray;
use homotopy_graphics::gl::buffer::{BufferKind, ElementBuffer};
use homotopy_graphics::gl::frame::Frame;
use homotopy_graphics::gl::geom::{Color, MVPMatrix, Vertex};
use homotopy_graphics::gl::GlCtx;
use homotopy_graphics::{draw, program, vertex_array};

#[derive(Clone, PartialEq, Properties)]
pub struct Props3D {}

pub enum Message3D {
    Render(f64),
}

pub struct Diagram3D {
    link: ComponentLink<Self>,
    canvas: NodeRef,
    renderer: Option<Renderer>,

    // If the render task is dropped, we won't get notified about `requestAnimationFrame()` calls,
    // so store a reference to the task here
    render_loop: Option<RenderTask>,
}

impl Component for Diagram3D {
    type Properties = Props3D;
    type Message = Message3D;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            canvas: Default::default(),
            renderer: None,
            render_loop: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        let Message3D::Render(dt) = msg;

        if let Some(ref mut renderer) = self.renderer {
            renderer.update(dt);
            renderer.render();

            let render_frame = self.link.callback(Message3D::Render);
            let handle = RenderService::request_animation_frame(render_frame);
            self.render_loop = Some(handle);
        }

        false
    }

    fn view(&self) -> Html {
        html! {
            <canvas width=1000 height=1000 ref={self.canvas.clone()}></canvas>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        let ctx = GlCtx::attach(self.canvas.clone()).unwrap();

        let mut renderer = Renderer {
            ctx,
            vertex_array: None,
            vertex_array2: None,
            elements: None,
            t: 0f32,
        };

        renderer.init();

        self.renderer = Some(renderer);

        if first_render {
            let render_frame = self.link.callback(Message3D::Render);
            let handle = RenderService::request_animation_frame(render_frame);
            self.render_loop = Some(handle);
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }
}

pub struct Renderer {
    ctx: GlCtx,
    vertex_array: Option<VertexArray>,
    vertex_array2: Option<VertexArray>,
    elements: Option<ElementBuffer>,
    t: f32,
}

impl Renderer {
    fn init(&mut self) {
        let triangle = self
            .ctx
            .mk_buffer(&[
                Vertex::new(-0.7, -0.7, 0.0),
                Vertex::new(0.7, -0.7, 0.0),
                Vertex::new(0.0, 0.7, 0.0),
            ])
            .unwrap();

        let square = self
            .ctx
            .mk_buffer(&[
                Vertex::new(-1.0, -1.0, 0.0),
                Vertex::new(-1.0, 1.0, 0.0),
                Vertex::new(1.0, 1.0, 0.0),
                Vertex::new(1.0, -1.0, 0.0),
            ])
            .unwrap();

        let colors = self
            .ctx
            .mk_buffer(&[
                Color::new(1.0, 0.0, 0.0),
                Color::new(0.0, 1.0, 0.0),
                Color::new(0.0, 0.0, 1.0),
            ])
            .unwrap();

        let colors2 = self
            .ctx
            .mk_buffer(&[
                Color::new(1.0, 0.0, 0.0),
                Color::new(0.0, 1.0, 0.0),
                Color::new(0.0, 0.0, 1.0),
                Color::new(1.0, 1.0, 1.0),
            ])
            .unwrap();

        let program = program!(
            self.ctx,
            "../../glsl/vert.glsl",
            "../../glsl/frag.glsl",
            { position, in_color },
            { transform }
        )
        .unwrap();

        let vertex_array = vertex_array!(&program, {
            position: &triangle,
            in_color: &colors,
        })
        .unwrap();
        self.vertex_array = Some(vertex_array);

        let vertex_array2 = vertex_array!(
            &program,
            &self.ctx.mk_element_buffer(&[0, 1, 2, 0, 2, 3]).unwrap(),
            {
                position: &square,
                in_color: &colors2,
            }
        ).unwrap();
        self.vertex_array2 = Some(vertex_array2);
    }

    fn update(&mut self, dt: f64) {
        self.t = dt as f32;
    }

    fn render(&self) {
        let mut frame = Frame::new(&self.ctx);

        frame.draw(draw!(self.vertex_array.as_ref().unwrap(), {
            transform: MVPMatrix::identity(),
        }));
        frame.draw(draw!(
            self.vertex_array2.as_ref().unwrap(),
            {
                transform: MVPMatrix::identity().then_rotate(
                    0.0,
                    0.0,
                    1.0,
                    Angle::radians(self.t / 200.0)
                ).then_scale(f32::sin(self.t / 170.0), f32::sin(self.t / 170.0), 1.0),
            }
        ));

        frame.render();
    }
}
