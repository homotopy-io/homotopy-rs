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
        delta::{Delta, DeltaAgent},
        settings::{KeyStore, Settings, Store},
        toast::{Toast, Toaster},
        touch_interface::{TouchAction, TouchInterface},
        Finger, Point,
    },
    model::proof::{Signature, View},
};

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
                self.camera.ortho = *self.local.get_orthographic_3d();

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
    ctx: GlCtx,
    scene: Scene,
    signature: Signature,
    subdivision_depth: u8,
    geometry_samples: u8,
    t: f32,
}

pub struct OrbitCamera {
    target: Vec3,
    phi: f32,
    theta: f32,
    distance: f32,
    fov: f32,
    ortho: bool,
    mouse: Option<Vec2>,
}

impl GlDiagramRenderer {
    fn new(ctx: GlCtx, settings: &Store<AppSettings>, props: &GlDiagramProps) -> Result<Self> {
        let depth = *settings.get_subdivision_depth() as u8;
        let samples = *settings.get_geometry_samples() as u8;

        Ok(Self {
            scene: Scene::build(
                &ctx,
                &props.diagram,
                if props.view.dimension() <= 3 {
                    ViewDimension::Three
                } else {
                    ViewDimension::Four
                },
                depth,
                samples,
            )?,
            ctx,
            signature: props.signature.clone(),
            subdivision_depth: depth,
            geometry_samples: samples,
            t: 0.0,
        })
    }

    fn update(&mut self, settings: &Store<AppSettings>, dt: f32) -> Result<()> {
        let depth = *settings.get_subdivision_depth() as u8;
        let samples = *settings.get_geometry_samples() as u8;

        if self.subdivision_depth != depth || self.geometry_samples != samples {
            self.subdivision_depth = depth;
            self.geometry_samples = samples;
            self.scene.reload_meshes(&self.ctx, depth, samples)?;
        }

        self.t += dt;

        Ok(())
    }

    fn render(&mut self, dimension: u8, camera: &OrbitCamera, settings: &Store<AppSettings>) {
        let mut frame = Frame::new(&mut self.ctx);
        let vp = camera.transform(&*frame);

        if !*settings.get_mesh_hidden() {
            let normals = *settings.get_debug_normals();
            let lighting = *settings.get_disable_lighting();
            let camera = camera.position();
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
                        lighting_disable: lighting,
                        camera_pos: camera,
                        d: Vec3::new(color.red, color.green, color.blue),
                    })
                });
            } else {
                // TODO(@doctorn) something sensible for time control
                let t = f32::sin(0.00025 * self.t);

                self.scene.draw(&mut frame, |generator, array| {
                    let color = if generator.id == 0 {
                        Vec3::new(30. / 255., 144. / 255., 1.)
                    } else {
                        Vec3::zero()
                    };

                    draw!(array, {
                        mvp: vp,
                        debug_normals: normals,
                        lighting_disable: lighting,
                        camera_pos: camera,
                        t: t,
                        d: color,
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
}

impl OrbitCamera {
    const DEFAULT_DISTANCE: f32 = 12.;
    const DEFAULT_FOV: f32 = 30.0;
    const DEFAULT_PHI: f32 = 0.5 * PI;
    const DEFAULT_THETA: f32 = 0.5 * PI;
    const EPSILON: f32 = 0.05;
    const FAR: f32 = 1000.;
    const NEAR: f32 = 0.01;

    fn position(&self) -> Vec3 {
        let sin_phi = f32::sin(self.phi);
        let cos_phi = f32::cos(self.phi);
        let sin_theta = f32::sin(self.theta);
        let cos_theta = f32::cos(self.theta);

        self.distance * Vec3::new(cos_phi * sin_theta, -cos_theta, -sin_phi * sin_theta)
            + self.target
    }

    fn transform(&self, ctx: &GlCtx) -> Mat4 {
        let perspective = if self.ortho {
            let scale = self.distance / 10.;
            let aspect = ctx.aspect_ratio();
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
                ctx.aspect_ratio(),
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

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: Default::default(),
            phi: Self::DEFAULT_PHI,
            theta: Self::DEFAULT_THETA,
            distance: Self::DEFAULT_DISTANCE,
            fov: Self::DEFAULT_FOV,
            ortho: false,
            mouse: None,
        }
    }
}

impl TouchInterface for OrbitCamera {
    fn mouse_down(&mut self, point: Point) {
        self.mouse = Some(Vec2::new(point.x as f32, point.y as f32));
    }

    fn mouse_up(&mut self) {
        self.mouse = None;
    }

    fn mouse_move(&mut self, next: Point) {
        let next = Vec2::new(next.x as f32, next.y as f32);
        if let Some(prev) = self.mouse {
            let delta = 4. * (next - prev) / 1000.;
            // TODO(@doctorn) divide by `self.gl_ctx.size()`
            self.apply_angle_delta(delta);
            self.mouse = Some(next);
        }
    }

    fn mouse_wheel(&mut self, _: Point, delta: f64) {
        self.apply_distance_delta(delta as f32);
    }

    fn touch_move(&mut self, _touches: &[(Finger, Point)]) {
        // TODO(@doctorn) touch contorls
    }

    fn touch_update(&mut self, _touches: &[(Finger, Point)]) {
        // TODO(@doctorn) touch contorls
    }

    fn reset(&mut self) {
        *self = Default::default();
    }
}

pub struct OrbitControl(Delta<OrbitCamera>);

impl OrbitControl {
    pub fn new() -> Self {
        Self(Delta::new())
    }

    pub fn zoom_in(&self) {
        self.0
            .emit(TouchAction::MouseWheel(Default::default(), -20.0));
    }

    pub fn zoom_out(&self) {
        self.0
            .emit(TouchAction::MouseWheel(Default::default(), 20.0));
    }

    pub fn reset(&self) {
        self.0.emit(TouchAction::Reset);
    }
}
