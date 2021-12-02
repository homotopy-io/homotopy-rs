use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use gloo::render::{request_animation_frame, AnimationFrame};
use homotopy_core::DiagramN;
use homotopy_graphics::{
    clay::{Scene, ViewDimension},
    draw,
    gl::{frame::Frame, GlCtx, Result},
};
use ultraviolet::{
    projection::rh_yup::{orthographic_gl, perspective_gl},
    Mat4, Vec2, Vec3,
};
use yew::prelude::*;

use crate::{
    app::AppSettings,
    components::{
        read_touch_list,
        settings::{KeyStore, Settings, Store},
        toast::{Toast, Toaster},
        Finger,
    },
    model::proof::{Signature, View},
};

pub enum GlDiagramMessage {
    Render(f64),
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

    renderer: Rc<RefCell<Option<GlDiagramRenderer>>>,
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

        Self {
            canvas: Default::default(),
            toaster: Toaster::new(),
            _settings: settings,

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

                if let Some(renderer) = &mut *self.renderer.borrow_mut() {
                    renderer.update(&self.local, dt).unwrap();
                    renderer.render(ctx.props().view.dimension(), &self.local);
                }

                // Schedule the next frame
                self.schedule_frame(ctx);
            }
            GlDiagramMessage::Setting(msg) => self.local.set(&msg),
        }

        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let on_mouse_move = {
            let renderer = Rc::clone(&self.renderer);
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                if let Some(r) = &mut *renderer.borrow_mut() {
                    r.on_mouse_move((e.client_x() as f32, e.client_y() as f32).into());
                }
            })
        };
        let on_mouse_up = {
            let renderer = Rc::clone(&self.renderer);
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                if let Some(r) = &mut *renderer.borrow_mut() {
                    r.on_mouse_up();
                }
            })
        };
        let on_mouse_down = {
            let renderer = Rc::clone(&self.renderer);
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                if let Some(r) = &mut *renderer.borrow_mut() {
                    r.on_mouse_down((e.client_x() as f32, e.client_y() as f32).into());
                }
            })
        };
        let on_wheel = {
            let renderer = Rc::clone(&self.renderer);
            Callback::from(move |e: WheelEvent| {
                e.prevent_default();
                if let Some(r) = &mut *renderer.borrow_mut() {
                    r.on_mouse_wheel(
                        (e.client_x() as f32, e.client_y() as f32).into(),
                        e.delta_y() as f32,
                    );
                }
            })
        };
        let on_touch_move = {
            let renderer = Rc::clone(&self.renderer);
            let node_ref = self.canvas.clone();
            Callback::from(move |e: TouchEvent| {
                e.prevent_default();
                if let Some(r) = &mut *renderer.borrow_mut() {
                    let touches = read_touch_list(&e.touches(), &node_ref)
                        .map(|(f, p)| (f, Vec2::new(p.x as f32, p.y as f32)))
                        .collect::<Vec<_>>();
                    r.on_touch_move(&touches);
                }
            })
        };
        let on_touch_update = {
            let renderer = Rc::clone(&self.renderer);
            let node_ref = self.canvas.clone();
            Callback::from(move |e: TouchEvent| {
                e.prevent_default();
                if let Some(r) = &mut *renderer.borrow_mut() {
                    let touches = read_touch_list(&e.touches(), &node_ref)
                        .map(|(f, p)| (f, Vec2::new(p.x as f32, p.y as f32)))
                        .collect::<Vec<_>>();
                    r.on_touch_update(&touches);
                }
            })
        };

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
                    Some(GlDiagramRenderer::new(gl_ctx, &self.local, ctx.props()).unwrap());
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

struct GlDiagramRenderer {
    gl_ctx: GlCtx,
    scene: Scene,
    signature: Signature,
    camera: OrbitCamera,
    subdivision_depth: u8,
    geometry_samples: u8,
    mouse: Option<Vec2>,
    t: f32,
}

pub struct OrbitCamera {
    target: Vec3,
    phi: f32,
    theta: f32,
    distance: f32,
    fov: f32,
    ortho: bool,
}

impl GlDiagramRenderer {
    fn new(gl_ctx: GlCtx, settings: &Store<AppSettings>, props: &GlDiagramProps) -> Result<Self> {
        let depth = *settings.get_subdivision_depth() as u8;
        let samples = *settings.get_geometry_samples() as u8;

        Ok(Self {
            scene: Scene::build(
                &gl_ctx,
                &props.diagram,
                if props.view.dimension() <= 3 {
                    ViewDimension::Three
                } else {
                    ViewDimension::Four
                },
                depth,
                samples,
            )?,
            signature: props.signature.clone(),
            camera: OrbitCamera::new(Vec3::zero(), 30.0),
            subdivision_depth: depth,
            geometry_samples: samples,
            mouse: None,
            gl_ctx,
            t: 0.0,
        })
    }

    fn update(&mut self, settings: &Store<AppSettings>, dt: f32) -> Result<()> {
        let depth = *settings.get_subdivision_depth() as u8;
        let samples = *settings.get_geometry_samples() as u8;

        if self.subdivision_depth != depth || self.geometry_samples != samples {
            self.subdivision_depth = depth;
            self.geometry_samples = samples;
            self.scene.reload_meshes(&self.gl_ctx, depth, samples)?;
        }

        self.camera.ortho = *settings.get_orthographic_3d();
        self.t += dt;

        Ok(())
    }

    fn render(&mut self, dimension: u8, settings: &Store<AppSettings>) {
        let mut frame = Frame::new(&mut self.gl_ctx);

        let vp = self.camera.transform(&*frame);

        if !*settings.get_mesh_hidden() {
            let normals = *settings.get_debug_normals();
            let camera = self.camera.position();
            let signature = &self.signature;

            if dimension <= 3 {
                self.scene.draw(&mut frame, |generator, array| {
                    let color = signature
                        .generator_info(generator)
                        .unwrap()
                        .color
                        .0
                        .into_format();
                    draw!(array, {
                        mvp: vp,
                        debug_normals: normals,
                        camera_pos: camera,
                        d: Vec3::new(color.red, color.green, color.blue),
                    })
                });
            } else {
                // TODO(@doctorn) something sensible for time control
                let t = f32::sin(0.00025 * self.t);

                self.scene.draw(&mut frame, |_, array| {
                    draw!(array, {
                        mvp: vp,
                        debug_normals: normals,
                        camera_pos: camera,
                        t: t,
                    })
                });
            }
        }

        if *settings.get_wireframe_3d() {
            self.scene.draw_wireframe(&mut frame, &vp);
        }

        if *settings.get_debug_axes() {
            self.scene.draw_axes(&mut frame, &vp);
        }
    }

    fn on_mouse_down(&mut self, point: Vec2) {
        self.mouse = Some(point);
    }

    fn on_mouse_up(&mut self) {
        self.mouse = None;
    }

    fn on_mouse_move(&mut self, next: Vec2) {
        if let Some(prev) = self.mouse {
            let delta = 4. * (next - prev) / self.gl_ctx.size();
            self.camera.apply_angle_delta(delta);
            self.mouse = Some(next);
        }
    }

    fn on_mouse_wheel(&mut self, _: Vec2, delta: f32) {
        self.camera.apply_distance_delta(delta);
    }

    #[allow(clippy::unused_self)]
    fn on_touch_move(&mut self, _touches: &[(Finger, Vec2)]) {
        // TODO(@doctorn) touch contorls
    }

    #[allow(clippy::unused_self)]
    fn on_touch_update(&mut self, _touches: &[(Finger, Vec2)]) {
        // TODO(@doctorn) touch contorls
    }
}

impl OrbitCamera {
    const DEFAULT_DISTANCE: f32 = 12.;
    const DEFAULT_PHI: f32 = 0.5 * PI;
    const DEFAULT_THETA: f32 = 0.5 * PI;
    const EPSILON: f32 = 0.05;
    const FAR: f32 = 100.;
    const NEAR: f32 = 0.01;

    fn new(target: Vec3, fov: f32) -> Self {
        Self {
            target,
            phi: Self::DEFAULT_PHI,
            theta: Self::DEFAULT_THETA,
            distance: Self::DEFAULT_DISTANCE,
            fov,
            ortho: false,
        }
    }

    fn position(&self) -> Vec3 {
        let sin_phi = f32::sin(self.phi);
        let cos_phi = f32::cos(self.phi);
        let sin_theta = f32::sin(self.theta);
        let cos_theta = f32::cos(self.theta);

        self.distance * Vec3::new(cos_phi * sin_theta, -cos_theta, -sin_phi * sin_theta)
            + self.target
    }

    fn transform(&self, gl_ctx: &GlCtx) -> Mat4 {
        let perspective = if self.ortho {
            let scale = self.distance / 10.;
            let aspect = gl_ctx.aspect_ratio();
            orthographic_gl(
                -aspect * scale,
                aspect * scale,
                -scale,
                scale,
                Self::NEAR,
                Self::FAR,
            )
        } else {
            perspective_gl(
                f32::to_radians(self.fov),
                gl_ctx.aspect_ratio(),
                Self::NEAR,
                Self::FAR,
            )
        };
        let view = Mat4::look_at(self.position(), self.target, Vec3::unit_y());

        perspective * view
    }

    fn apply_angle_delta(&mut self, delta: Vec2) {
        self.phi -= delta.x;
        self.theta = (self.theta + delta.y).clamp(Self::EPSILON, PI - Self::EPSILON);
    }

    fn apply_distance_delta(&mut self, delta: f32) {
        self.distance *= if delta > 0. { 1.1 } else { 1.0 / 1.1 };
    }
}
