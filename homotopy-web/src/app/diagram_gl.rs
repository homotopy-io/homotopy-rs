use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use futures::future::try_join_all;
use gloo::render::{request_animation_frame, AnimationFrame};
use homotopy_core::Diagram;
use homotopy_gl::GlCtx;
use js_sys::Uint8Array;
use ultraviolet::Vec3;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{Blob, HtmlCanvasElement, OffscreenCanvas, WebGl2RenderingContext};
use yew::prelude::*;

pub use self::orbit_camera::OrbitCamera;
use self::{
    renderer::Renderer,
    scrub_controls::{ScrubAction, ScrubComponent, ScrubState, SCRUB},
};
use crate::{
    app::{AppSettings, AppSettingsKey, AppSettingsMsg},
    components::{
        delta::{CallbackIdx, Delta},
        toast::{toast, Toast},
        touch_interface::{TouchAction, TouchInterface},
    },
    model::{
        generate_download,
        proof::{Signature, View},
        zip_files,
    },
};

mod buffers;
mod orbit_camera;
mod renderer;
mod scrub_controls;

std::thread_local! {
    pub static CAMERA: Delta<OrbitCamera> = Default::default();
    pub static FRAME_CAPTURE: RefCell<Callback<FrameCaptureControl>> = Default::default();
}

#[derive(Debug, Copy, Clone)]
pub enum FrameCaptureControl {
    One,
    All(u16),
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

#[derive(Debug)]
pub enum DiagramGlMessage {
    Render(f64),
    Camera(f32, f32, f32, Vec3),
    Scrub(f32),
    Setting(AppSettingsMsg),
    FrameCapture(FrameCaptureControl),
    FrameCaptureScrub(f32),
    FrameCaptureDump,
    FrameCaptureFlush,
    Noop,
}

impl DiagramGlMessage {
    const fn requires_rerender(&self) -> bool {
        matches!(self, Self::FrameCaptureScrub(_))
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct DiagramGlProps {
    pub diagram: Diagram,
    pub signature: Signature,
    pub view: View,
}

pub struct DiagramGl {
    canvas: NodeRef,
    camera: OrbitCamera,
    renderer: Rc<RefCell<Option<Renderer>>>,
    global_t: f32,
    t_coord: f32,

    // If the render task is dropped, we won't get notified about `requestAnimationFrame()`
    // calls, so store a reference to the task here
    render_loop: Option<AnimationFrame>,

    setting_callback: CallbackIdx,
    camera_callback: CallbackIdx,
    scrub_callback: CallbackIdx,

    // drawing surface for frame capture
    offscreen_canvas: Option<OffscreenCanvas>,
    clip_rect: Option<(i32, i32, i32, i32)>,
    // stores (many) frames in PNG format
    export_frame_buffer: Vec<JsFuture>,
    // message queue to be processed after render completes
    pending_queue: VecDeque<DiagramGlMessage>,
}

impl Component for DiagramGl {
    type Message = DiagramGlMessage;
    type Properties = DiagramGlProps;

    fn create(ctx: &Context<Self>) -> Self {
        let setting_callback = AppSettings::subscribe(
            &[AppSettingsKey::animated_3d],
            ctx.link().callback(DiagramGlMessage::Setting),
        )[0];

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

        FRAME_CAPTURE.set(ctx.link().callback(DiagramGlMessage::FrameCapture));

        Self {
            canvas: Default::default(),

            camera: Default::default(),
            renderer: Default::default(),
            global_t: Default::default(),
            t_coord: Default::default(),

            render_loop: None,

            setting_callback,
            camera_callback,
            scrub_callback,

            offscreen_canvas: Default::default(),
            clip_rect: Default::default(),
            export_frame_buffer: Default::default(),
            pending_queue: Default::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            DiagramGlMessage::Render(t) => {
                let t = t as f32;
                let dt = t - self.global_t;
                self.global_t = t;
                if Self::is_animated(ctx) {
                    // Slow the animation such that we get 1s per cospan
                    SCRUB.with(|s| {
                        s.emit(&ScrubAction::Advance(
                            1e-3 * dt / ctx.props().diagram.size().unwrap() as f32,
                        ));
                    });
                }
                // Update camera settings
                self.camera.set_ortho(AppSettings::get_orthographic_3d());

                if let Some(renderer) = &mut *self.renderer.borrow_mut() {
                    renderer.update().unwrap();
                    renderer.render(&self.camera, self.t_coord);
                }

                // frame capture
                if let (Some(offscreen_canvas), Some((min_x, min_y, max_x, max_y))) =
                    (&self.offscreen_canvas, self.clip_rect)
                {
                    let canvas = self
                        .canvas
                        .get()
                        .expect("no canvas")
                        .dyn_into::<HtmlCanvasElement>()
                        .unwrap();
                    let context = offscreen_canvas
                        .get_context("2d")
                        .expect("no 2d context")
                        .unwrap()
                        .dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()
                        .expect("failed to cast to OffscreenCanvasRenderingContext2d");
                    let width = f64::from(max_x - min_x);
                    let height = f64::from(max_y - min_y);
                    context
                        .draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(&canvas, min_x.into(), min_y.into(), width, height, 0.0, 0.0, width, height)
                        .expect("failed to draw image from canvas");
                }
                if let Some(first) = self.pending_queue.pop_front() {
                    // always pop at least one message
                    ctx.link().send_message(first);

                    while let Some(m) =
                        pop_front_if(&mut self.pending_queue, |m| !m.requires_rerender())
                    {
                        ctx.link().send_message(m);
                    }
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
            DiagramGlMessage::Setting(msg) => {
                if let AppSettingsMsg::animated_3d(_) = msg {
                    return true;
                }
            }
            DiagramGlMessage::FrameCapture(FrameCaptureControl::One) => {
                tracing::info!("Capturing single frame");
                self.clip_rect = self.bounding_box();
                if let Some((min_x, min_y, max_x, max_y)) = self.clip_rect {
                    self.offscreen_canvas = Some(
                        OffscreenCanvas::new((max_x - min_x) as u32, (max_y - min_y) as u32)
                            .unwrap(),
                    );
                    self.export_frame_buffer = Vec::with_capacity(1);
                    self.pending_queue
                        .push_back(DiagramGlMessage::FrameCaptureDump);
                    self.pending_queue
                        .push_back(DiagramGlMessage::FrameCaptureFlush);
                }
            }
            DiagramGlMessage::FrameCapture(FrameCaptureControl::All(n_frames)) => {
                tracing::info!("Starting frame capture for {n_frames} frames");
                self.clip_rect = self.bounding_box();
                if let Some((min_x, min_y, max_x, max_y)) = self.clip_rect {
                    self.offscreen_canvas = Some(
                        OffscreenCanvas::new((max_x - min_x) as u32, (max_y - min_y) as u32)
                            .unwrap(),
                    );
                    self.export_frame_buffer = Vec::with_capacity(n_frames as usize);
                    let old = SCRUB.with(|s| s.state().t);
                    self.pending_queue
                        .push_back(DiagramGlMessage::FrameCaptureScrub(0.0));
                    for t in (0..n_frames).map(|i| f32::from(i) / f32::from(n_frames)) {
                        self.pending_queue
                            .push_back(DiagramGlMessage::FrameCaptureScrub(t));
                        self.pending_queue
                            .push_back(DiagramGlMessage::FrameCaptureDump);
                    }
                    self.pending_queue
                        .push_back(DiagramGlMessage::FrameCaptureScrub(old));
                    self.pending_queue
                        .push_back(DiagramGlMessage::FrameCaptureFlush);
                }
            }
            DiagramGlMessage::FrameCaptureScrub(t) => {
                tracing::debug!("Scrubbing to t={t}");
                SCRUB.with(|s| {
                    s.emit(&ScrubAction::Scrub(t));
                });
            }
            DiagramGlMessage::FrameCaptureDump => {
                tracing::debug!("Dumping frame");
                let canvas = self.offscreen_canvas.as_mut().unwrap();

                self.export_frame_buffer
                    .push(JsFuture::from(canvas.convert_to_blob().unwrap()));
            }
            DiagramGlMessage::FrameCaptureFlush => {
                let frame_count = self.export_frame_buffer.len();
                match frame_count {
                    0 => {
                        unreachable!("tried to download empty export frame buffer");
                    }
                    1 => {
                        let promise = self.export_frame_buffer.remove(0);
                        spawn_local(async move {
                            let blob = promise
                                .await
                                .expect("failed to get blob")
                                .dyn_into::<Blob>()
                                .expect("failed to cast to blob");
                            let frame = JsFuture::from(blob.array_buffer())
                                .await
                                .expect("failed to convert blob to array buffer");
                            generate_download(
                                "homotopy_io_export",
                                "png",
                                &Uint8Array::new(&frame).to_vec(),
                            )
                            .expect("failed to generate download for single frame");
                        });
                    }
                    n => {
                        let promises = self.export_frame_buffer.drain(..).collect::<Vec<_>>();
                        spawn_local(async move {
                            let blobs = try_join_all(promises).await.expect("failed to get blobs");
                            let frames = try_join_all(
                                blobs
                                    .into_iter()
                                    .map(|b| {
                                        let blob =
                                            b.dyn_into::<Blob>().expect("failed to cast to blob");
                                        JsFuture::from(blob.array_buffer())
                                    })
                                    .collect::<Vec<_>>(),
                            )
                            .await
                            .expect("failed to convert blobs to array buffers");
                            tracing::info!("Exporting {} frames to download", n);
                            let zip = zip_files((0..n).map(|f| {
                                (
                                    format!("frame-{f}.png"),
                                    Uint8Array::new(&frames[f]).to_vec(),
                                )
                            }));
                            generate_download("homotopy_io_export", "zip", &zip)
                                .expect("failed to generate download for multiple frames");
                        });
                        tracing::info!("Finished frame capture");
                    }
                };
            }
            DiagramGlMessage::Noop => {}
        }

        false
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        AppSettings::unsubscribe(&[AppSettingsKey::animated_3d], &[self.setting_callback]);
        CAMERA.with(|c| c.unregister(self.camera_callback));
        SCRUB.with(|s| s.unregister(self.scrub_callback));
        FRAME_CAPTURE.take();
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

        let scrub = if Self::is_animated(ctx) {
            html! { <ScrubComponent slices={ctx.props().diagram.size().unwrap()} /> }
        } else {
            Default::default()
        };

        html! {
            <>
                <canvas
                    id="canvas"
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
            toast(Toast::error("Failed to get WebGL 2.0 context"));
        }
    }
}

impl DiagramGl {
    fn is_animated(ctx: &Context<Self>) -> bool {
        let n = ctx.props().view.dimension();
        n == 4 || n == 3 && AppSettings::get_animated_3d()
    }

    fn schedule_frame(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        self.render_loop = Some(request_animation_frame(move |t| {
            link.send_message(DiagramGlMessage::Render(t));
        }));
    }

    /// Returns the bounding box of this [`DiagramGl`] in Canvas coordinates (Y-axis goes down).
    fn bounding_box(&self) -> Option<(i32, i32, i32, i32)> {
        let canvas = self
            .canvas
            .get()
            .and_then(|c| c.dyn_into::<HtmlCanvasElement>().ok())?;
        let gl: WebGl2RenderingContext = canvas
            .get_context("webgl2")
            .ok()??
            .dyn_into::<WebGl2RenderingContext>()
            .ok()?;

        let width = canvas.width() as i32;
        let height = canvas.height() as i32;
        tracing::info!("Canvas size: {}x{}", width, height);

        // Read the pixels from the WebGL context
        let mut pixels = vec![0; (width * height * 4) as usize];
        gl.read_pixels_with_opt_u8_array(
            0,
            0,
            width,
            height,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(&mut pixels),
        )
        .ok()?;
        // NOTE: In WebGL, Y-axis goes up, so we need to flip the y-axis
        let mut min_x = width;
        let mut min_y = height;
        let mut max_x = 0;
        let mut max_y = 0;
        for canvas_y in 0..height {
            let y = height - 1 - canvas_y;
            for x in 0..width {
                let r = pixels[(y * width + x) as usize * 4];
                let g = pixels[(y * width + x) as usize * 4 + 1];
                let b = pixels[(y * width + x) as usize * 4 + 2];
                let a = pixels[(y * width + x) as usize * 4 + 3];
                if !(r == u8::MAX && g == u8::MAX && b == u8::MAX && a == u8::MAX) {
                    min_x = min_x.min(x);
                    min_y = min_y.min(canvas_y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(canvas_y);
                }
            }
        }
        Some((min_x, min_y, max_x, max_y))
    }
}

// https://github.com/rust-lang/rust/issues/135889
fn pop_front_if<T>(vd: &mut VecDeque<T>, predicate: impl FnOnce(&mut T) -> bool) -> Option<T> {
    let first = vd.front_mut()?;
    if predicate(first) {
        vd.pop_front()
    } else {
        None
    }
}
