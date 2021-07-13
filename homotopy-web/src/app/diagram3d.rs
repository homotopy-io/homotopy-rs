use yew::prelude::*;
use yew_services::render::RenderTask;
use yew_services::RenderService;

use crate::graphics::array::VertexArray;
use crate::graphics::buffer::Buffer;
use crate::graphics::frame::{Draw, Frame};
use crate::graphics::geom::Vertex;
use crate::graphics::shader::{FragmentShader, Program, VertexShader};
use crate::graphics::GraphicsCtx;

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
            <canvas ref={self.canvas.clone()}></canvas>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        let ctx = GraphicsCtx::attach(self.canvas.clone()).unwrap();

        let mut renderer = Renderer {
            ctx,
            triangle: None,
            colors: None,
            vertex_array: None,
            program: None,
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
    ctx: GraphicsCtx,
    triangle: Option<Buffer<Vertex>>,
    colors: Option<Buffer<Vertex>>,
    vertex_array: Option<VertexArray>,
    program: Option<Program>,
}

impl Renderer {
    fn init(&mut self) {
        let mut triangle = Buffer::new(&self.ctx).unwrap();
        triangle.buffer(&[
            Vertex::new(-0.7, -0.7, 0.0),
            Vertex::new(0.7, -0.7, 0.0),
            Vertex::new(0.0, 0.7, 0.0),
        ]);
        self.triangle = Some(triangle);

        let mut colors = Buffer::new(&self.ctx).unwrap();
        colors.buffer(&[
            Vertex::new(1.0, 0.0, 0.0),
            Vertex::new(0.0, 1.0, 0.0),
            Vertex::new(0.0, 0.0, 1.0),
        ]);
        self.colors = Some(colors);

        let mut vertex_array = VertexArray::new(&self.ctx).unwrap();
        vertex_array.attribute(0, self.triangle.as_ref().unwrap());
        vertex_array.attribute(1, self.colors.as_ref().unwrap());
        self.vertex_array = Some(vertex_array);

        let program = Program::link(
            &self.ctx,
            VertexShader::compile(&self.ctx, include_str!("../graphics/shader/vert.glsl")).unwrap(),
            FragmentShader::compile(&self.ctx, include_str!("../graphics/shader/frag.glsl"))
                .unwrap(),
        )
        .unwrap();
        self.program = Some(program);
    }

    fn update(&mut self, _dt: f64) {}

    fn render(&self) {
        let mut frame = Frame::new(&self.ctx);
        frame.draw(Draw::new(
            self.program.as_ref().unwrap(),
            self.vertex_array.as_ref().unwrap(),
        ));
        frame.render();
    }
}
