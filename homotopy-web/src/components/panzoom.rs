use closure::closure;
use yew::prelude::*;

use homotopy_core::Direction;

use crate::components::delta::{Delta, DeltaCallback, State};
use crate::components::{bounding_rect, read_touch_list, Finger, Point, Vector};

use super::delta::DeltaAgent;

#[derive(Debug, Clone)]
pub enum PanZoomAction {
    TouchUpdate(Vec<(Finger, Point)>),
    TouchMove(Vec<(Finger, Point)>),
    MouseWheel(Point, f64),
    MouseDown(Point),
    MouseMove(Point),
    MouseUp,
    Reset,
}

pub struct PanZoomState {
    pub translate: Vector,
    pub scale: f64,
    mouse: Option<Point>,
    touches: Vec<(Finger, Point)>,
}

impl Default for PanZoomState {
    fn default() -> Self {
        Self {
            translate: Default::default(),
            scale: 1.0,
            mouse: None,
            touches: vec![],
        }
    }
}

impl State for PanZoomState {
    type Action = PanZoomAction;

    fn update(&mut self, action: &Self::Action) {
        match *action {
            PanZoomAction::MouseDown(point) => self.mouse = Some(point),
            PanZoomAction::MouseUp => self.mouse = None,
            PanZoomAction::MouseMove(next) => {
                if let Some(prev) = self.mouse {
                    self.translate += next - prev;
                    self.mouse = Some(next);
                }
            }
            PanZoomAction::MouseWheel(point, delta) => {
                let scale = self.scale * if delta < 0.0 { 1.1 } else { 1.0 / 1.1 };
                self.translate = point - (point - self.translate) * (scale / self.scale);
                self.scale = scale;
            }
            PanZoomAction::TouchMove(ref touches) => {
                let mut touches = touches.clone();
                touches.sort_by_key(|(finger, _)| *finger);

                if touches.len() != 2 || self.touches.len() != 2 {
                    self.touches = touches;
                    return;
                }

                let average_next = (touches[0].1.to_vector() + touches[1].1.to_vector()) * 0.5;
                let average_prev =
                    (self.touches[0].1.to_vector() + self.touches[1].1.to_vector()) * 0.5;

                let scale = {
                    let distance_prev = (self.touches[0].1 - self.touches[1].1).length().max(0.01);
                    let distance_next = (touches[0].1 - touches[1].1).length().max(0.01);
                    self.scale * (distance_next / distance_prev)
                };

                self.translate =
                    average_next - (average_prev - self.translate) * (scale / self.scale);
                self.scale = scale;
                self.touches = touches;
            }
            PanZoomAction::TouchUpdate(ref touches) => {
                let mut touches = touches.clone();
                touches.sort_by_key(|(finger, _)| *finger);
                self.touches = touches;
            }
            PanZoomAction::Reset => {
                self.translate = Default::default();
                self.scale = 1.0;
            }
        }
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct PanZoomProps {
    #[prop_or_default]
    pub on_scroll: Callback<Direction>,
    #[prop_or_default]
    pub children: Children,
}

pub enum PanZoomMessage {
    Delta(Vector, f64),
}

pub struct PanZoomComponent {
    props: PanZoomProps,
    node_ref: NodeRef,
    translate: Vector,
    scale: f64,
    _delta: Delta<PanZoomState>,
}

impl Component for PanZoomComponent {
    type Properties = PanZoomProps;
    type Message = PanZoomMessage;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let delta = Delta::new();
        delta.register(Box::new(move |agent: &DeltaAgent<PanZoomState>, _| {
            let state = agent.state();
            link.send_message(PanZoomMessage::Delta(state.translate, state.scale));
        }));

        Self {
            props,
            node_ref: Default::default(),
            translate: Default::default(),
            scale: 1.0,
            _delta: delta,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        let PanZoomMessage::Delta(translate, scale) = msg;
        self.translate = translate;
        self.scale = scale;
        true
    }

    fn view(&self) -> Html {
        let style = format!(
            "transform: translate(calc({x}px - 50%), calc({y}px - 50%)) scale({s})",
            x = self.translate.x,
            y = self.translate.y,
            s = self.scale
        );

        let on_mouse_move = {
            let delta = Delta::<PanZoomState>::new();
            Callback::from(closure!(|e: MouseEvent| {
                e.prevent_default();
                delta.emit(PanZoomAction::MouseMove(
                    (f64::from(e.client_x()), f64::from(e.client_y())).into(),
                ));
            }))
        };

        let on_mouse_up = {
            let delta = Delta::<PanZoomState>::new();
            Callback::from(closure!(|e: MouseEvent| {
                e.prevent_default();
                delta.emit(PanZoomAction::MouseUp);
            }))
        };

        let on_mouse_down = {
            let delta = Delta::<PanZoomState>::new();
            Callback::from(closure!(|e: MouseEvent| {
                e.prevent_default();
                if e.alt_key() {
                    delta.emit(PanZoomAction::MouseDown(
                        (f64::from(e.client_x()), f64::from(e.client_y())).into(),
                    ));
                }
            }))
        };

        let on_wheel = {
            let delta = Delta::<PanZoomState>::new();
            let node_ref = self.node_ref.clone();
            let on_scroll = self.props.on_scroll.clone();

            Callback::from(closure!(|e: WheelEvent| {
                let dy = e.delta_y();
                if e.alt_key() {
                    e.prevent_default();

                    let rect = bounding_rect(&node_ref).unwrap();

                    // Offset the observed x and y by half the dimensinos of the panzoom view to
                    // account for centering (not required on mouse moves as that information is
                    // only used relatively)
                    let x = f64::from(e.client_x()) - rect.left() - 0.5 * rect.width();
                    let y = f64::from(e.client_y()) - rect.top() - 0.5 * rect.height();

                    delta.emit(PanZoomAction::MouseWheel((x, y).into(), dy));
                } else if dy > 0.0 {
                    on_scroll.emit(Direction::Forward);
                } else {
                    on_scroll.emit(Direction::Backward);
                }
            }))
        };

        let on_touch_move = {
            let delta = Delta::<PanZoomState>::new();
            let node_ref = self.node_ref.clone();
            Callback::from(closure!(|e: TouchEvent| {
                e.prevent_default();
                delta.emit(PanZoomAction::TouchMove(
                    read_touch_list(&e.touches(), &node_ref).collect(),
                ));
            }))
        };

        let on_touch_update = {
            let delta = Delta::<PanZoomState>::new();
            let node_ref = self.node_ref.clone();
            Callback::from(closure!(|e: TouchEvent| {
                // e.prevent_default();
                delta.emit(PanZoomAction::TouchUpdate(
                    read_touch_list(&e.touches(), &node_ref).collect(),
                ));
            }))
        };

        html! {
            <content
                class="panzoom"
                onmousemove={on_mouse_move}
                onmouseup={on_mouse_up}
                onmousedown={on_mouse_down}
                onwheel={on_wheel}
                ontouchmove={on_touch_move}
                ontouchcancel={on_touch_update.clone()}
                ontouchend={on_touch_update.clone()}
                ontouchstart={on_touch_update}
                ref={self.node_ref.clone()}
            >
                <div
                    class="panzoom__inner"
                    style={style}
                    ref={self.node_ref.clone()}
                >
                    { for self.props.children.iter() }
                </div>
            </content>
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        props != self.props && {
            self.props = props;
            true
        }
    }
}

pub type PanZoomAgent = DeltaAgent<PanZoomState>;

pub struct PanZoom(Delta<PanZoomState>);

impl PanZoom {
    pub fn new() -> Self {
        Self(Delta::new())
    }

    pub fn zoom_in(&self) {
        self.0
            .emit(PanZoomAction::MouseWheel(Default::default(), -20.0));
    }

    pub fn zoom_out(&self) {
        self.0
            .emit(PanZoomAction::MouseWheel(Default::default(), 20.0));
    }

    pub fn reset(&self) {
        self.0.emit(PanZoomAction::Reset);
    }

    pub fn register(&self, callback: DeltaCallback<PanZoomState>) {
        self.0.register(callback);
    }
}
