use std::{cell::RefCell, rc::Rc};

use gloo::render::{request_animation_frame, AnimationFrame};
use homotopy_core::Diagram;
use homotopy_gl::GlCtx;
use ultraviolet::Vec3;
use yew::prelude::*;

pub use self::orbit_camera::OrbitCamera;
use self::{
    renderer::Renderer,
    scrub_controls::{ScrubAction, ScrubComponent, ScrubState, SCRUB},
};
use crate::{
    app::{AppSettings, AppSettingsKeyStore, AppSettingsMsg},
    components::{
        delta::{CallbackIdx, Delta},
        toast::{toast, Toast},
        touch_interface::{TouchAction, TouchInterface},
    },
    model::proof::{Signature, View},
};

mod buffers;
mod orbit_camera;
mod renderer;
mod scrub_controls;

std::thread_local! {
    pub static CAMERA: Delta<OrbitCamera> = Default::default();
}

pub struct GlViewControl {}

impl GlViewControl {
    pub fn zoom_in() {
        CAMERA.with(|c| c.emit(&TouchAction::MouseWheel(Default::default(), -20.0)));
    }

    pub fn zoom_out() {
        CAMERA.with(|c| c.emit(&TouchAction::MouseWheel(Default::default(), 20.0)));
    }

    pub fn reset() {
        CAMERA.with(|c| c.emit(&TouchAction::Reset));
        SCRUB.with(|s| s.emit(&ScrubAction::Scrub(0.)));
    }
}

pub enum DiagramGlMessage {
    Render(f64),
    Camera(f32, f32, f32, Vec3),
    Scrub(f32),
    Setting(AppSettingsMsg),
    Noop,
}

#[derive(Properties, Clone, PartialEq, Eq)]
pub struct DiagramGlProps {
    pub diagram: Diagram,
    pub signature: Signature,
    pub view: View,
}

pub struct DiagramGl {
    canvas: NodeRef,
    camera: OrbitCamera,
    renderer: Rc<RefCell<Option<Renderer>>>,
    local: AppSettingsKeyStore,
    global_t: f32,
    t_coord: f32,

    // If the render task is dropped, we won't get notified about `requestAnimationFrame()`
    // calls, so store a reference to the task here
    render_loop: Option<AnimationFrame>,

    camera_callback: CallbackIdx,
    scrub_callback: CallbackIdx,
}

impl Component for DiagramGl {
    type Message = DiagramGlMessage;
    type Properties = DiagramGlProps;

    fn create(ctx: &Context<Self>) -> Self {
        AppSettings::subscribe(
            AppSettings::ALL,
            ctx.link().callback(DiagramGlMessage::Setting),
        );

        let camera_callback = CAMERA.with(|c| {
            c.register(ctx.link().callback(|state: OrbitCamera| {
                DiagramGlMessage::Camera(state.phi, state.theta, state.distance, state.target)
            }))
        });

        let scrub_callback = SCRUB.with(|s| {
            s.register(
                ctx.link()
                    .callback(|state: ScrubState| DiagramGlMessage::Scrub(state.t)),
            )
        });

        Self {
            canvas: Default::default(),

            camera: Default::default(),
            renderer: Default::default(),
            local: Default::default(),
            global_t: Default::default(),
            t_coord: Default::default(),

            render_loop: None,

            camera_callback,
            scrub_callback,
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
                    SCRUB.with(|s| {
                        s.emit(&ScrubAction::Advance(
                            1e-3 * dt / ctx.props().diagram.size().unwrap() as f32,
                        ));
                    });
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
            DiagramGlMessage::Noop => {}
        }

        false
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        CAMERA.with(|c| c.unregister(self.camera_callback));

        SCRUB.with(|s| s.unregister(self.scrub_callback));
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let interface_callback = ctx.link().callback(|e: TouchAction| {
            CAMERA.with(|c| c.emit(&e));
            DiagramGlMessage::Noop
        });
        let on_mouse_move = OrbitCamera::on_mouse_move(interface_callback.clone());
        let on_mouse_up = OrbitCamera::on_mouse_up(interface_callback.clone());
        let on_mouse_down = OrbitCamera::on_mouse_down(interface_callback.clone());
        let on_wheel = OrbitCamera::on_wheel(&self.canvas, interface_callback.clone());
        let on_touch_move = OrbitCamera::on_touch_move(&self.canvas, interface_callback.clone());
        let on_touch_update = OrbitCamera::on_touch_update(&self.canvas, interface_callback);

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
            toast(Toast::error("Failed to get WebGL 2.0 context"));
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
