use yew::prelude::*;
use yew_services::render::RenderTask;
use yew_services::RenderService;

use crate::graphics::buffer::Buffer;
use crate::graphics::geom::Vertex;
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
        let mut ctx = GraphicsCtx::attach(self.canvas.clone()).unwrap();

        let mut renderer = Renderer {
            ctx,
            triangle: None,
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
    }

    fn update(&mut self, dt: f64) {}

    fn render(&self) {
        // let mut frame = self.ctx.mk_frame();
        // frame.draw(Draw::new(self.program, self.array));
        // frame.render();
    }
}
