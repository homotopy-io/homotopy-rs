use euclid::{Angle, Vector3D};

use yew::prelude::*;
use yew::services::render::RenderTask;
use yew::services::RenderService;

use homotopy_graphics::gl::array::VertexArray;
use homotopy_graphics::gl::buffer::Buffer;
use homotopy_graphics::gl::frame::Frame;
use homotopy_graphics::gl::geom::{Color, ModelMatrix, Vertex, ViewMatrix};
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
            <canvas style="width: 100%; height: 100%; display: block" ref={self.canvas.clone()}></canvas>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        let mut ctx = GlCtx::attach(self.canvas.clone()).unwrap();
        ctx.set_clear_color(Color::new(0.1, 0.1, 0.1));
        self.renderer = Some(Renderer::init(ctx));

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
    rabbit: VertexArray,
    t: f32,
}

impl Renderer {
    const BUNNY: &'static [u8] = include_bytes!("../../static/bunny.obj");

    fn init(ctx: GlCtx) -> Self {
        let (models, _) = tobj::load_obj_buf(
            &mut std::io::BufReader::new(Renderer::BUNNY),
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
            |_| Err(tobj::LoadError::OpenFileFailed),
        )
        .unwrap();

        let model = models.first().unwrap();
        let mesh = &model.mesh;

        let vertices: Buffer<Vertex> = unsafe {
            ctx.mk_buffer_unchecked(&mesh.positions, mesh.positions.len() / 3)
                .unwrap()
        };
        let normals: Buffer<Vertex> = unsafe {
            ctx.mk_buffer_unchecked(&mesh.normals, mesh.normals.len() / 3)
                .unwrap()
        };

        let indices = ctx
            .mk_element_buffer(
                &mesh
                    .indices
                    .iter()
                    .copied()
                    .map(|x| x as u16)
                    .collect::<Vec<_>>(),
            )
            .unwrap();

        let program = program!(
            ctx,
            "../../glsl/vert.glsl",
            "../../glsl/frag.glsl",
            { position, normal },
            { mvp, m_inv }
        )
        .unwrap();

        let rabbit = vertex_array!(
            &program,
            &indices,
            {
                position: &vertices,
                normal: &normals,
            }
        )
        .unwrap();

        Self {
            ctx,
            rabbit,
            t: 0.0,
        }
    }

    fn update(&mut self, dt: f64) {
        self.t = dt as f32;
    }

    fn render(&mut self) {
        let mut frame = Frame::new(&mut self.ctx);

        let model_matrix = ModelMatrix::translation(0.0, -0.5, 0.0)
            .then_scale(0.3, 0.3, 0.3)
            .then_rotate(0.0, 1.0, 0.0, Angle::radians(self.t * 1e-3))
            .then_translate(Vector3D::new(0.0, 0.0, -2.0));
        let view_matrix = ViewMatrix::identity();
        let projection_matrix = frame.perspective(30.0, 0.5, 10.0);
        let mvp = model_matrix.then(&view_matrix).then(&projection_matrix);
        let model_inv = model_matrix.inverse().unwrap();

        frame.draw(draw!(
            &self.rabbit,
            {
                mvp: mvp,
                m_inv: model_inv,
            }
        ));

        frame.render();
    }
}
