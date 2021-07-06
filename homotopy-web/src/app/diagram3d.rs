use yew::prelude::*;
use yew_services::render::RenderTask;
use yew_services::RenderService;

use crate::graphics::shader::ShaderKind;
use crate::graphics::{buffer, geom, shader, GraphicsCtx};

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
    _render_loop: Option<RenderTask>,
}

impl Component for Diagram3D {
    type Properties = Props3D;
    type Message = Message3D;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            canvas: Default::default(),
            renderer: None,
            _render_loop: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        let Message3D::Render(dt) = msg;

        if let Some(ref mut renderer) = self.renderer {
            renderer.update(dt);
            renderer.render();
        }

        false
    }

    fn view(&self) -> Html {
        html! {
            <canvas ref={self.canvas.clone()}></canvas>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        let mut ctx = GraphicsCtx::attach(self.canvas.clone()).unwrap();

        let vert_shader = ctx
            .compile_shader(
                ShaderKind::Vert,
                include_str!("../graphics/shaders/vert.glsl"),
            )
            .unwrap();
        let frag_shader = ctx
            .compile_shader(
                ShaderKind::Frag,
                include_str!("../graphics/shaders/frag.glsl"),
            )
            .unwrap();

        let mut renderer = Renderer {
            program: ctx.link_program(vert_shader, frag_shader).unwrap(),
            triangle: ctx
                .mk_vertex_buffer(&[
                    geom::Vertex::new(-0.7, -0.7, 0.0),
                    geom::Vertex::new(0.7, -0.7, 0.0),
                    geom::Vertex::new(0.0, 0.7, 0.0),
                ])
                .unwrap(),
            ctx,
        };
        renderer.init();

        self.renderer = Some(renderer);

        if first_render {
            let render_frame = self.link.callback(Message3D::Render);
            let handle = RenderService::request_animation_frame(render_frame);
            self._render_loop = Some(handle);
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }
}

pub struct Renderer {
    ctx: GraphicsCtx,
    program: shader::Program,
    triangle: buffer::VertexBuffer,
}

impl Renderer {
    fn init(&mut self) {}

    fn update(&mut self, dt: f64) {}

    fn render(&self) {
        let mut frame = self.ctx.mk_frame();
        frame
            .build_draw(self.triangle)
            .with_program(self.program)
            .commit();
        frame.render();
    }
}
