use std::{cell::RefCell, rc::Rc};

use gloo::render::{request_animation_frame, AnimationFrame};
use homotopy_core::Diagram;
use homotopy_gl::GlCtx;
use ultraviolet::Vec3;
use yew::prelude::*;

pub use self::orbit_camera::OrbitCamera;
use self::{
    renderer::Renderer,
    scrub_controls::{ScrubAction, ScrubComponent, ScrubState},
};
use crate::{
    app::AppSettings,
    components::{
        delta::{Delta, DeltaAgent},
        settings::{KeyStore, Settings, Store},
        toast::{Toast, Toaster},
        touch_interface::{TouchAction, TouchInterface},
    },
    model::proof::{Signature, View},
};

mod buffers;
mod orbit_camera;
mod renderer;
mod scrub_controls;

pub struct GlViewControl {
    camera: Delta<OrbitCamera>,
    scrub_control: Delta<ScrubState>,
}

impl GlViewControl {
    pub fn new() -> Self {
        Self {
            camera: Delta::new(),
            scrub_control: Delta::new(),
        }
    }

    pub fn zoom_in(&self) {
        self.camera
            .emit(TouchAction::MouseWheel(Default::default(), -20.0));
    }

    pub fn zoom_out(&self) {
        self.camera
            .emit(TouchAction::MouseWheel(Default::default(), 20.0));
    }

    pub fn reset(&self) {
        self.camera.emit(TouchAction::Reset);
        self.scrub_control.emit(ScrubAction::Scrub(0.));
    }
}

pub enum DiagramGlMessage {
    Render(f64),
    Camera(f32, f32, f32, Vec3),
    Scrub(f32),
    Setting(<Store<AppSettings> as KeyStore>::Message),
}

#[derive(Properties, Clone, PartialEq, Eq)]
pub struct DiagramGlProps {
    pub diagram: Diagram,
    pub signature: Signature,
    pub view: View,
}

pub struct DiagramGl {
    canvas: NodeRef,
    toaster: Toaster,
    _settings: AppSettings,
    _camera_delta: Delta<OrbitCamera>,
    scrub_delta: Delta<ScrubState>,

    camera: OrbitCamera,
    renderer: Rc<RefCell<Option<Renderer>>>,
    local: Store<AppSettings>,
    global_t: f32,
    t_coord: f32,

    // If the render task is dropped, we won't get notified about `requestAnimationFrame()`
    // calls, so store a reference to the task here
    render_loop: Option<AnimationFrame>,
}

impl Component for DiagramGl {
    type Message = DiagramGlMessage;
    type Properties = DiagramGlProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut settings = AppSettings::connect(ctx.link().callback(DiagramGlMessage::Setting));

        settings.subscribe(AppSettings::ALL);

        let camera_delta = Delta::new();
        let link = ctx.link().clone();
        camera_delta.register(Box::new(move |agent: &DeltaAgent<OrbitCamera>, _| {
            let state = agent.state();
            link.send_message(DiagramGlMessage::Camera(
                state.phi,
                state.theta,
                state.distance,
                state.target,
            ));
        }));

        let scrub_delta = Delta::new();
        let link = ctx.link().clone();
        scrub_delta.register(Box::new(move |agent: &DeltaAgent<ScrubState>, _| {
            let state = agent.state();
            link.send_message(DiagramGlMessage::Scrub(state.t));
        }));

        Self {
            canvas: Default::default(),
            toaster: Toaster::new(),
            _settings: settings,
            _camera_delta: camera_delta,
            scrub_delta,

            camera: Default::default(),
            renderer: Default::default(),
            local: Default::default(),
            global_t: Default::default(),
            t_coord: Default::default(),

            render_loop: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            DiagramGlMessage::Render(t) => {
                let t = t as f32;
                let dt = t - self.global_t;
                self.global_t = t;
                if self.is_animated(ctx) {
                    // Slow the animation such that we get 1s per cospan
                    self.scrub_delta.emit(ScrubAction::Advance(
                        1e-3 * dt / ctx.props().diagram.size().unwrap() as f32,
                    ));
                }
                // Update camera settings
                self.camera.set_ortho(*self.local.get_orthographic_3d());

                if let Some(renderer) = &mut *self.renderer.borrow_mut() {
                    renderer.update(&self.local).unwrap();
                    renderer.render(&self.camera, &self.local, self.t_coord);
                }

                // Schedule the next frame
                self.schedule_frame(ctx);
            }
            DiagramGlMessage::Camera(phi, theta, distance, target) => {
                self.camera.phi = phi;
                self.camera.theta = theta;
                self.camera.distance = distance;
                self.camera.target = target;
            }
            DiagramGlMessage::Scrub(t) => {
                // Scrub controls are [0,1], but animation is [-1,1] so map between
                self.t_coord = 2. * t - 1.;
            }
            DiagramGlMessage::Setting(msg) => self.local.set(&msg),
        }

        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_mouse_move = OrbitCamera::on_mouse_move();
        let on_mouse_up = OrbitCamera::on_mouse_up();
        let on_mouse_down = OrbitCamera::on_mouse_down();
        let on_wheel = OrbitCamera::on_wheel(&self.canvas);
        let on_touch_move = OrbitCamera::on_touch_move(&self.canvas);
        let on_touch_update = OrbitCamera::on_touch_update(&self.canvas);

        let scrub = if self.is_animated(ctx) {
            html! { <ScrubComponent slices={ctx.props().diagram.size().unwrap()} /> }
        } else {
            Default::default()
        };

        html! {
            <>
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
                {scrub}
            </>
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

impl DiagramGl {
    fn is_animated(&self, ctx: &Context<Self>) -> bool {
        let n = ctx.props().view.dimension();
        n == 4 || n == 3 && *self.local.get_animated_3d()
    }

    fn schedule_frame(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        self.render_loop = Some(request_animation_frame(move |t| {
            link.send_message(DiagramGlMessage::Render(t));
        }));
    }
}
