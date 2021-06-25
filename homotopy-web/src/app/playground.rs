// TODO(@doctorn) remove

use yew::prelude::*;

use crate::graphics::shader::ShaderKind;
use crate::graphics::GraphicsCtx;

pub struct GraphicsPlayground {
    canvas: NodeRef,
    graphics_ctx: Option<GraphicsCtx>,
}

impl Component for GraphicsPlayground {
    type Properties = ();
    type Message = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {
            canvas: NodeRef::default(),
            graphics_ctx: None,
        }
    }

    fn view(&self) -> Html {
        html! {
            <canvas ref={self.canvas.clone()}></canvas>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let mut ctx = GraphicsCtx::attach(self.canvas.clone()).unwrap();

            let vert_shader = ctx
                .compile_shader(
                    ShaderKind::Vert,
                    r#"
                        attribute vec4 position;
                        void main() {
                            gl_Position = position;
                        }
                    "#,
                )
                .unwrap();
            let frag_shader = ctx
                .compile_shader(
                    ShaderKind::Frag,
                    r#"
                        void main() {
                            gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
                        }
                    "#,
                )
                .unwrap();
            let program = ctx.link_program(vert_shader, frag_shader).unwrap();

            let verts = ctx
                .mk_vertex_buffer(&[-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0])
                .unwrap();
            let mut frame = ctx.mk_frame();
            frame.build_draw(verts).with_program(program).commit();
            frame.render();
            self.graphics_ctx = Some(ctx);
        }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }
}
