use std::{cell::RefCell, rc::Rc};

use homotopy_common::hash::FastHashSet;
use homotopy_core::{debug::Drawable, Diagram};
use homotopy_gl::{
    array::VertexArray,
    draw,
    frame::{DepthTest, Frame},
    vertex_array, GlCtx, Result,
};
use homotopy_graphics::geom::{CubicalGeometry, SimplicialGeometry};
use ultraviolet::{Vec3, Vec4};
use web_sys::CanvasRenderingContext2d;
use yew::prelude::*;

use super::diagram_gl::{renderer::shaders::Shaders, OrbitCamera};
use crate::{
    buffers::buffer_debug,
    components::{
        delta::{Delta, DeltaAgent},
        touch_interface::TouchInterface,
    },
    two_d::TwoDCtx,
};

pub struct DebugScene {
    // rendered by 2D canvas context
    pub vertex_labels: Vec<(Vec3, String)>,
    // rendered by GL
    pub wireframe_components: Vec<VertexArray>,
}

impl DebugScene {
    pub fn new(ctx: &GlCtx, diagram: &Diagram) -> Result<Self> {
        let cubical = match diagram.dimension() {
            0 => CubicalGeometry::new::<0>(diagram),
            1 => CubicalGeometry::new::<1>(diagram),
            2 => CubicalGeometry::new::<2>(diagram),
            _ => CubicalGeometry::new::<3>(diagram),
        }
        .unwrap();
        let simplicial = SimplicialGeometry::from(cubical);
        let mut vertex_labels = Vec::new();
        let mut verts: FastHashSet<_> = Default::default();
        for (tri, _) in simplicial.areas.values() {
            verts.insert(tri[0]);
            verts.insert(tri[1]);
            verts.insert(tri[2]);
        }

        for (_, data) in simplicial
            .verts
            .iter()
            .filter(|(vert, _)| verts.contains(vert))
        {
            vertex_labels.push((data.position.xyz(), format!("{:?}", data.generator)));
        }
        let mut wireframe_components = vec![];
        for projected_buffers in buffer_debug(&simplicial, ctx)? {
            wireframe_components.push(vertex_array!(
                ctx,
                &projected_buffers.element_buffer,
                [&projected_buffers.vert_buffer]
            )?);
        }
        Ok(Self {
            vertex_labels,
            wireframe_components,
        })
    }
}

pub struct DebugRenderer {
    ctx: GlCtx,
    shaders: Shaders,
    scene: DebugScene,
}

impl DebugRenderer {
    fn new(ctx: GlCtx, props: &DebugGlProps) -> Result<Self> {
        // generate a scene from the props
        match &props.drawable {
            Drawable::Diagram(d) => Ok(Self {
                shaders: Shaders::new(&ctx)?,
                scene: DebugScene::new(&ctx, d)?,
                ctx,
            }),
            Drawable::Rewrite(_, _, _) => todo!(),
        }
    }

    pub fn render(&mut self, camera: &OrbitCamera) {
        let v = camera.view_transform(&self.ctx);
        let p = camera.perspective_transform(&self.ctx);
        let mut frame = Frame::new(&mut self.ctx).with_clear_color(Vec4::broadcast(1.));
        for array in &self.scene.wireframe_components {
            frame.draw(draw! {
                &self.shaders.wireframe,
                array,
                &[],
                DepthTest::Disable,
                {
                    mv: v,
                    p: p,
                }
            });
        }
    }
}

#[derive(Properties, Clone, PartialEq, Eq)]
pub struct DebugGlProps {
    pub drawable: Drawable,
}

pub enum DebugGlMessage {
    Render,
    Camera(f32, f32, f32, Vec3),
}

pub struct DebugGl {
    canvas: NodeRef,
    text: NodeRef,
    // how much the camera has changed while dragging
    _camera_delta: Delta<OrbitCamera>,

    camera: OrbitCamera,
    renderer: Rc<RefCell<Option<DebugRenderer>>>,
    text_renderer: Option<CanvasRenderingContext2d>,
}

impl Component for DebugGl {
    type Message = DebugGlMessage;
    type Properties = DebugGlProps;

    fn create(ctx: &Context<Self>) -> Self {
        let camera_delta = Delta::new();
        let link = ctx.link().clone();
        camera_delta.register(Box::new(move |agent: &DeltaAgent<OrbitCamera>, _| {
            let state = agent.state();
            link.send_message(DebugGlMessage::Camera(
                state.phi,
                state.theta,
                state.distance,
                state.target,
            ));
        }));
        Self {
            canvas: Default::default(),
            text: Default::default(),
            _camera_delta: camera_delta,
            camera: Default::default(),
            renderer: Default::default(),
            text_renderer: Default::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            DebugGlMessage::Render => {
                if let Some(renderer) = &mut *self.renderer.borrow_mut() {
                    renderer.render(&self.camera);
                    if let Some(text_renderer) = &mut self.text_renderer {
                        text_renderer.clear_rect(
                            0.0,
                            0.0,
                            text_renderer.canvas().unwrap().width().into(),
                            text_renderer.canvas().unwrap().height().into(),
                        );
                        for (pos, label) in &renderer.scene.vertex_labels {
                            let clip = self
                                .camera
                                .perspective_transform(&renderer.ctx)
                                .transform_point3(
                                    self.camera
                                        .view_transform(&renderer.ctx)
                                        .transform_point3(*pos),
                                );
                            let x = f64::from(clip[0] * 0.5 + 0.5)
                                * f64::from(text_renderer.canvas().unwrap().width());
                            let y = f64::from(clip[1] * -0.5 + 0.5)
                                * f64::from(text_renderer.canvas().unwrap().height());
                            text_renderer
                                .fill_text_with_max_width(
                                    label,
                                    x,
                                    y,
                                    // TODO: compute the actual gap available
                                    f64::from(
                                        6.0 * OrbitCamera::DEFAULT_DISTANCE / self.camera.distance,
                                    ),
                                )
                                .unwrap();
                        }
                    }
                }
            }
            DebugGlMessage::Camera(phi, theta, distance, target) => {
                self.camera.phi = phi;
                self.camera.theta = theta;
                self.camera.distance = distance;
                self.camera.target = target;
                ctx.link().send_message(DebugGlMessage::Render);
            }
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
            <div style="position: relative; width: 100%; height: 100%">
                <canvas
                    style="width: 100%; height: 100%; display: block"
                    ref={self.canvas.clone()}
                />
                <canvas
                    style="width: 100%; height: 100%; display: block; position: absolute; left: 0px; top: 0px; z-index: 10"
                    onmousemove={on_mouse_move}
                    onmouseup={on_mouse_up}
                    onmousedown={on_mouse_down}
                    onwheel={on_wheel}
                    ontouchmove={on_touch_move}
                    ontouchcancel={on_touch_update.clone()}
                    ontouchend={on_touch_update.clone()}
                    ontouchstart={on_touch_update}
                    ref={self.text.clone()}
                />
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if let Ok(gl_ctx) = GlCtx::attach(&self.canvas) {
            {
                *self.renderer.borrow_mut() =
                    Some(DebugRenderer::new(gl_ctx, ctx.props()).unwrap());
            }

            if first_render {
                ctx.link().send_message(DebugGlMessage::Render);
            }
        } else {
            log::error!("Failed to get WebGL 2.0 context");
        }

        if let Ok(two_d_ctx) = TwoDCtx::attach(&self.text) {
            two_d_ctx.ctx.set_text_align("center");
            two_d_ctx.ctx.set_text_baseline("middle");
            self.text_renderer = Some(two_d_ctx.ctx);
        } else {
            log::error!("Failed to get a 2D context");
        }
    }
}
