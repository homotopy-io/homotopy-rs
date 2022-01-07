use std::{cell::RefCell, rc::Rc};

use gloo::render::{request_animation_frame, AnimationFrame};
use homotopy_core::DiagramN;
use homotopy_graphics::gl::GlCtx;
use yew::prelude::*;

pub use self::orbit_camera::{OrbitCamera, OrbitControl};
use self::renderer::Renderer;
use crate::{
    app::AppSettings,
    components::{
        delta::{Delta, DeltaAgent},
        settings::{KeyStore, Settings, Store},
        toast::{Toast, Toaster},
        touch_interface::TouchInterface,
    },
    model::proof::{Signature, View},
};

mod orbit_camera;
mod renderer;

pub enum GlDiagramMessage {
    Render(f64),
    Delta(f32, f32, f32),
    Setting(<Store<AppSettings> as KeyStore>::Message),
}

#[derive(Properties, PartialEq, Clone)]
pub struct GlDiagramProps {
    pub diagram: DiagramN,
    pub signature: Signature,
    pub view: View,
}

pub struct GlDiagram {
    canvas: NodeRef,
    toaster: Toaster,
    _settings: AppSettings,
    _delta: Delta<OrbitCamera>,

    camera: OrbitCamera,
    renderer: Rc<RefCell<Option<Renderer>>>,
    local: Store<AppSettings>,
    t: f32,

    // If the render task is dropped, we won't get notified about `requestAnimationFrame()`
    // calls, so store a reference to the task here
    render_loop: Option<AnimationFrame>,
}

impl Component for GlDiagram {
    type Message = GlDiagramMessage;
    type Properties = GlDiagramProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut settings = AppSettings::connect(ctx.link().callback(GlDiagramMessage::Setting));

        settings.subscribe(AppSettings::ALL);

        let delta = Delta::new();
        let link = ctx.link().clone();
        delta.register(Box::new(move |agent: &DeltaAgent<OrbitCamera>, _| {
            let state = agent.state();
            link.send_message(GlDiagramMessage::Delta(
                state.phi,
                state.theta,
                state.distance,
            ));
        }));

        Self {
            canvas: Default::default(),
            toaster: Toaster::new(),
            _settings: settings,
            _delta: delta,

            camera: Default::default(),
            renderer: Default::default(),
            local: Default::default(),
            t: Default::default(),

            render_loop: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            GlDiagramMessage::Render(t) => {
                let t = t as f32;
                let dt = t - self.t;
                // Update current time
                self.t = t;
                // Update camera settings
                self.camera.set_ortho(*self.local.get_orthographic_3d());

                if let Some(renderer) = &mut *self.renderer.borrow_mut() {
                    renderer.update(&self.local, dt).unwrap();
                    renderer.render(ctx.props().view.dimension(), &self.camera, &self.local);
                }

                // Schedule the next frame
                self.schedule_frame(ctx);
            }
            GlDiagramMessage::Delta(phi, theta, distance) => {
                self.camera.phi = phi;
                self.camera.theta = theta;
                self.camera.distance = distance;
            }
            GlDiagramMessage::Setting(msg) => self.local.set(&msg),
        }

        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let on_mouse_move = OrbitCamera::on_mouse_move();
        let on_mouse_up = OrbitCamera::on_mouse_up();
        let on_mouse_down = OrbitCamera::on_mouse_down();
        let on_wheel = OrbitCamera::on_wheel(&self.canvas);
        let on_touch_move = OrbitCamera::on_touch_move(&self.canvas);
        let on_touch_update = OrbitCamera::on_touch_update(&self.canvas);

        html! {
            <canvas
                style="width: 100%; height: 100%; display: block"
                onmousemove={on_mouse_move}
                onmouseup={on_mouse_up}
                onmousedown={on_mouse_down}
                onwheel={on_wheel}
                ontouchmove={on_touch_move}
                ontouchcancel={on_touch_update.clone()}
                ontouchend={on_touch_update.clone()}
                ontouchstart={on_touch_update}
                ref={self.canvas.clone()}
            />
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if let Ok(gl_ctx) = GlCtx::attach(&self.canvas) {
            {
                *self.renderer.borrow_mut() =
                    Some(Renderer::new(gl_ctx, &self.local, ctx.props()).unwrap());
            }

            if first_render {
                self.schedule_frame(ctx);
            }
        } else {
            self.render_loop = None;
            self.toaster
                .toast(Toast::error("Failed to get WebGL 2.0 context"));
        }
    }
}

impl GlDiagram {
    fn schedule_frame(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        self.render_loop = Some(request_animation_frame(move |t| {
            link.send_message(GlDiagramMessage::Render(t));
        }));
    }
}
