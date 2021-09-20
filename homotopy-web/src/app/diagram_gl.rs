use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use gloo::render::{request_animation_frame, AnimationFrame};
use homotopy_core::DiagramN;
use homotopy_graphics::{
    clay::{
        geom::Mesh,
        subdivision::{subdivide_3, subdivide_4},
    },
    draw,
    gl::{array::VertexArray, buffer::ElementKind, frame::Frame, shader::Program, GlCtx, Result},
    program, vertex_array,
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
    model::proof::View,
};

pub enum GlDiagramMessage {
    Render(f64),
    Setting(<Store<AppSettings> as KeyStore>::Message),
}

#[derive(Properties, PartialEq, Clone)]
pub struct GlDiagramProps {
    pub diagram: DiagramN,
    pub view: View,
}

pub struct GlDiagram {
    link: ComponentLink<Self>,
    props: GlDiagramProps,
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

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut settings = AppSettings::connect(link.callback(GlDiagramMessage::Setting));

        settings.subscribe(AppSettings::ALL);

        Self {
            link,
            props,
            canvas: Default::default(),
            toaster: Toaster::new(),
            _settings: settings,

            renderer: Default::default(),
            local: Default::default(),
            t: Default::default(),

            render_loop: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            GlDiagramMessage::Render(t) => {
                let t = t as f32;
                let dt = t - self.t;
                // Update current time
                self.t = t;

                if let Some(renderer) = &mut *self.renderer.borrow_mut() {
                    renderer.update(&self.local, dt).unwrap();
                    renderer.render(&self.local);
                }

                // Schedule the next frame
                self.schedule_frame();
            }
            GlDiagramMessage::Setting(msg) => self.local.set(&msg),
        }

        false
    }

    fn view(&self) -> Html {
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

    fn rendered(&mut self, first_render: bool) {
        if let Ok(ctx) = GlCtx::attach(&self.canvas) {
            {
                *self.renderer.borrow_mut() =
                    Some(GlDiagramRenderer::new(ctx, &self.local, &self.props).unwrap());
            }

            if first_render {
                self.schedule_frame();
            }
        } else {
            self.render_loop = None;
            self.toaster
                .toast(Toast::error("Failed to get WebGL 2.0 context"));
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props != props && {
            self.props = props;
            true
        }
    }
}

impl GlDiagram {
    fn schedule_frame(&mut self) {
        let link = self.link.clone();
        self.render_loop = Some(request_animation_frame(move |t| {
            link.send_message(GlDiagramMessage::Render(t));
        }));
    }
}

struct GlDiagramRenderer {
    ctx: GlCtx,
    props: GlDiagramProps,
    scene: Scene,
    camera: OrbitCamera,
    subdivision_depth: u8,
    mouse: Option<Vec2>,
    t: f32,
}

struct Scene {
    solid_program_3: Program,
    solid_program_4: Program,
    wireframe_program: Program,

    solid_mesh: VertexArray,
    wireframe_mesh: VertexArray,

    axes: VertexArray,
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
    fn new(mut ctx: GlCtx, settings: &Store<AppSettings>, props: &GlDiagramProps) -> Result<Self> {
        Ok(Self {
            props: props.clone(),
            scene: Scene::new(
                &mut ctx,
                &props.diagram,
                props.view,
                *settings.get_subdivision_depth() as u8,
            )?,
            camera: OrbitCamera::new(Vec3::zero(), 30.0),
            subdivision_depth: *settings.get_subdivision_depth() as u8,
            mouse: None,
            ctx,
            t: 0.0,
        })
    }

    fn update(&mut self, settings: &Store<AppSettings>, dt: f32) -> Result<()> {
        let depth = *settings.get_subdivision_depth() as u8;

        if self.subdivision_depth != depth {
            self.subdivision_depth = depth;
            self.scene.reload_meshes(
                &mut self.ctx,
                &self.props.diagram,
                self.props.view,
                self.subdivision_depth,
            )?;
        }

        self.camera.ortho = *settings.get_orthographic_3d();
        self.t += dt;

        Ok(())
    }

    fn render(&mut self, settings: &Store<AppSettings>) {
        let mut frame = Frame::new(&mut self.ctx);

        let vp = self.camera.transform(&*frame);

        // TODO(@doctorn) light relative to camera up direciton
        if !*settings.get_mesh_hidden() {
            if self.props.view.dimension() <= 3 {
                frame.draw(draw! {
                    &self.scene.solid_mesh,
                    {
                        mvp: vp,
                        debug_normals: *settings.get_debug_normals(),
                        light_pos: self.camera.position(),
                    }
                });
            } else {
                frame.draw(draw! {
                    &self.scene.solid_mesh,
                    {
                        mvp: vp,
                        debug_normals: *settings.get_debug_normals(),
                        light_pos: self.camera.position(),
                        // TODO(@doctorn) something sensible for time control
                        t: f32::sin(0.00025 * self.t),
                    }
                });
            }
        }

        if *settings.get_wireframe_3d() {
            frame.draw(draw! {
                &self.scene.wireframe_mesh,
                { mvp: vp }
            });
        }

        if *settings.get_debug_axes() {
            frame.draw(draw! {
                &self.scene.axes,
                { mvp: vp }
            });
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
            let delta = 4. * (next - prev) / self.ctx.size();
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

impl Scene {
    fn new(ctx: &mut GlCtx, diagram: &DiagramN, view: View, subdivision_depth: u8) -> Result<Self> {
        let solid_program_3 = program!(
            ctx,
            "../../glsl/vert.glsl",
            "../../glsl/frag.glsl",
            { position, normal },
            { mvp, debug_normals, light_pos },
        )?;
        let solid_program_4 = program!(
            ctx,
            "../../glsl/vert_4d.glsl",
            "../../glsl/frag.glsl",
            { position_start, position_end, normal_start, normal_end },
            { mvp, debug_normals, light_pos, t },
        )?;
        let wireframe_program = program!(
            ctx,
            "../../glsl/wireframe_vert.glsl",
            "../../glsl/wireframe_frag.glsl",
            { position, color },
            { mvp },
        )?;
        let (solid_mesh, wireframe_mesh) = Self::buffer_meshes(
            ctx,
            diagram,
            view,
            &solid_program_3,
            &solid_program_4,
            &wireframe_program,
            subdivision_depth,
        )?;
        let axes = Self::buffer_axes(ctx, &wireframe_program)?;

        Ok(Self {
            solid_program_3,
            solid_program_4,
            wireframe_program,

            solid_mesh,
            wireframe_mesh,

            axes,
        })
    }

    fn reload_meshes(
        &mut self,
        ctx: &mut GlCtx,
        diagram: &DiagramN,
        view: View,
        subdivision_depth: u8,
    ) -> Result<()> {
        let (solid_mesh, wireframe_mesh) = Self::buffer_meshes(
            ctx,
            diagram,
            view,
            &self.solid_program_3,
            &self.solid_program_4,
            &self.wireframe_program,
            subdivision_depth,
        )?;

        self.solid_mesh = solid_mesh;
        self.wireframe_mesh = wireframe_mesh;

        Ok(())
    }

    fn buffer_meshes(
        ctx: &mut GlCtx,
        diagram: &DiagramN,
        view: View,
        solid_program_3: &Program,
        solid_program_4: &Program,
        wireframe_program: &Program,
        subdivision_depth: u8,
    ) -> Result<(VertexArray, VertexArray)> {
        if view.dimension() <= 3 {
            let subdivided = subdivide_3(Mesh::build(diagram).unwrap().into(), subdivision_depth);
            let buffers = subdivided.buffer(ctx)?;

            let solid_mesh = vertex_array!(
                solid_program_3,
                &buffers.element_buffer,
                {
                    position: &buffers.vertex_buffer,
                    normal: &buffers.normal_buffer,
                }
            )?;
            let wireframe_mesh = vertex_array!(
                wireframe_program,
                &buffers.wireframe_element_buffer,
                { position: &buffers.vertex_buffer }
            )?;

            Ok((solid_mesh, wireframe_mesh))
        } else {
            // TODO(@doctorn) use real diagram
            let subdivided = subdivide_4(Mesh::build(diagram).unwrap().into(), subdivision_depth);
            let buffers = subdivided.buffer(ctx)?;

            let solid_mesh = vertex_array!(
                solid_program_4,
                &buffers.element_buffer,
                {
                    position_start: &buffers.vertex_start_buffer,
                    position_end: &buffers.vertex_end_buffer,
                    normal_start: &buffers.normal_start_buffer,
                    normal_end: &buffers.normal_end_buffer,
                }
            )?;
            let wireframe_mesh = vertex_array!(
                wireframe_program,
                &buffers.wireframe_element_buffer,
                { position: &buffers.wireframe_vertex_buffer }
            )?;

            Ok((solid_mesh, wireframe_mesh))
        }
    }

    fn buffer_axes(ctx: &mut GlCtx, program: &Program) -> Result<VertexArray> {
        let axes_elements = ctx.mk_element_buffer(&[0, 1, 2, 3, 4, 5], ElementKind::Lines)?;
        let axes_verts = ctx.mk_buffer(&[
            Vec3::zero(),
            Vec3::unit_x(),
            Vec3::zero(),
            Vec3::unit_y(),
            Vec3::zero(),
            Vec3::unit_z(),
        ])?;
        let axes_colors = ctx.mk_buffer(&[
            Vec3::unit_x(),
            Vec3::unit_x(),
            Vec3::unit_y(),
            Vec3::unit_y(),
            Vec3::unit_z(),
            Vec3::unit_z(),
        ])?;

        vertex_array!(
            program,
            &axes_elements,
            {
                position: &axes_verts,
                color: &axes_colors,
            }
        )
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

    fn transform(&self, ctx: &GlCtx) -> Mat4 {
        let perspective = if self.ortho {
            orthographic_gl(
                -ctx.aspect_ratio(),
                ctx.aspect_ratio(),
                -1.,
                1.,
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
