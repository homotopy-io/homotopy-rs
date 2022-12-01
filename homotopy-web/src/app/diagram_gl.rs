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
    app::settings::AppSettingsDispatch,
    components::{
        delta::Delta,
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
    ScrubCallback(Callback<ScrubAction>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct DiagramGlProps {
    pub diagram: Diagram,
    pub signature: Signature,
    pub view: View,
    pub settings: AppSettingsDispatch,
}

pub struct DiagramGl {
    canvas: NodeRef,
    toaster: Toaster,
    // Nasty hack to make ScrubControls own its state
    // We don't want to redraw the DiagramGl
    scrub_dispatch: Option<Callback<ScrubAction>>,

    camera: OrbitCamera,
    renderer: Rc<RefCell<Option<Renderer>>>,
    global_t: f32,
    t_coord: f32,

    // If the render task is dropped, we won't get notified about `requestAnimationFrame()`
    // calls, so store a reference to the task here
    render_loop: Option<AnimationFrame>,
}

impl Component for DiagramGl {
    type Message = DiagramGlMessage;
    type Properties = DiagramGlProps;

    fn create(_ctx: &Context<Self>) -> Self {
        // When orbit camera changes
        // Need to get event DiagramGlMessage::Camera
        // When scrub_state changes
        // Need to get event DiagramGlMessage::Scrub with state.t
        Self {
            canvas: Default::default(),
            toaster: Toaster::new(),
            scrub_dispatch: None,

            camera: Default::default(),
            renderer: Default::default(),
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
                if ctx.props().view.dimension() == 4 {
                    // Slow the animation such that we get 1s per cospan
                    if let Some(dispatch) = &self.scrub_dispatch {
                        dispatch.emit(ScrubAction::Advance(
                            1e-3 * dt / ctx.props().diagram.size().unwrap() as f32,
                        ));
                    }
                }
                // Update camera settings
                self.camera
                    .set_ortho(*ctx.props().settings.inner.get_orthographic_3d());

                if let Some(renderer) = &mut *self.renderer.borrow_mut() {
                    renderer.update(&ctx.props().settings.inner).unwrap();
                    renderer.render(&self.camera, &ctx.props().settings.inner, self.t_coord);
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
            DiagramGlMessage::ScrubCallback(c) => {
                self.scrub_dispatch = Some(c);
            }
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

        let scrub = if ctx.props().view.dimension() == 4 {
            html! {
                <ScrubComponent
                    slices={ctx.props().diagram.size().unwrap()}
                    dispatch={ctx.link().callback(|x| x)}
                />
            }
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
                *self.renderer.borrow_mut() = Some(Renderer::new(gl_ctx, ctx.props()).unwrap());
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
    fn schedule_frame(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        self.render_loop = Some(request_animation_frame(move |t| {
            link.send_message(DiagramGlMessage::Render(t));
        }));
    }
}
