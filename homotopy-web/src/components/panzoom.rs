use homotopy_core::Direction;
use yew::prelude::*;

use crate::components::{
    delta::{CallbackIdx, Delta},
    touch_interface::{TouchAction, TouchInterface},
    Finger, Point, Vector,
};

#[derive(Clone)]
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

impl TouchInterface for PanZoomState {
    fn mouse_down(&mut self, alt_key: bool, point: Point) {
        if alt_key {
            self.mouse = Some(point);
        }
    }

    fn mouse_up(&mut self) {
        self.mouse = None;
    }

    fn mouse_move(&mut self, _alt_key: bool, next: Point) {
        if let Some(prev) = self.mouse {
            self.translate += next - prev;
            self.mouse = Some(next);
        }
    }

    fn mouse_wheel(&mut self, point: Point, delta: f64) {
        let scale = self.scale * if delta < 0.0 { 1.1 } else { 1.0 / 1.1 };
        self.translate = point - (point - self.translate) * (scale / self.scale);
        self.scale = scale;
    }

    fn touch_move(&mut self, touches: &[(Finger, Point)]) {
        let mut touches = touches.to_vec();
        touches.sort_by_key(|(finger, _)| *finger);

        if touches.len() != 2 || self.touches.len() != 2 {
            self.touches = touches;
            return;
        }

        let average_next = (touches[0].1.to_vector() + touches[1].1.to_vector()) * 0.5;
        let average_prev = (self.touches[0].1.to_vector() + self.touches[1].1.to_vector()) * 0.5;

        let scale = {
            let distance_prev = (self.touches[0].1 - self.touches[1].1).length().max(0.01);
            let distance_next = (touches[0].1 - touches[1].1).length().max(0.01);
            self.scale * (distance_next / distance_prev)
        };

        self.translate = average_next - (average_prev - self.translate) * (scale / self.scale);
        self.scale = scale;
        self.touches = touches;
    }

    fn touch_update(&mut self, touches: &[(Finger, Point)]) {
        let mut touches = touches.to_vec();
        touches.sort_by_key(|(finger, _)| *finger);
        self.touches = touches;
    }

    fn reset(&mut self) {
        self.translate = Default::default();
        self.scale = 1.0;
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
    Noop,
}

pub struct PanZoomComponent {
    node_ref: NodeRef,
    translate: Vector,
    scale: f64,
    callback_idx: CallbackIdx,
}

impl Component for PanZoomComponent {
    type Message = PanZoomMessage;
    type Properties = PanZoomProps;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        let callback_idx = PANZOOM_STATE.with(|s| {
            s.register(link.callback(|state: PanZoomState| {
                PanZoomMessage::Delta(state.translate, state.scale)
            }))
        });

        Self {
            node_ref: Default::default(),
            translate: Default::default(),
            scale: 1.0,
            callback_idx,
        }
    }

    #[allow(clippy::float_cmp)]
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        if let PanZoomMessage::Delta(translate, scale) = msg {
            if self.translate == translate && self.scale == scale {
                return false;
            }

            self.translate = translate;
            self.scale = scale;
            true
        } else {
            false
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        PANZOOM_STATE.with(|s| {
            s.unregister(self.callback_idx);
        });
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let style = format!(
            "transform: translate(calc({x}px - 50%), calc({y}px - 50%)) scale({s})",
            x = self.translate.x,
            y = self.translate.y,
            s = self.scale
        );

        let interface_callback = ctx.link().callback(|e: TouchAction| {
            PANZOOM_STATE.with(|s| s.emit(&e));
            PanZoomMessage::Noop
        });
        let on_mouse_move = PanZoomState::on_mouse_move(interface_callback.clone());
        let on_mouse_up = PanZoomState::on_mouse_up(interface_callback.clone());
        let on_mouse_down = PanZoomState::on_mouse_down(interface_callback.clone());
        let on_touch_move = PanZoomState::on_touch_move(&self.node_ref, interface_callback.clone());
        let on_touch_update =
            PanZoomState::on_touch_update(&self.node_ref, interface_callback.clone());
        let on_wheel = {
            let on_scroll = ctx.props().on_scroll.clone();
            let node_ref = self.node_ref.clone();
            Callback::from(move |e: WheelEvent| {
                if e.alt_key() {
                    PanZoomState::on_wheel(&node_ref, interface_callback.clone()).emit(e);
                } else if e.delta_y() < 0.0 {
                    on_scroll.emit(Direction::Forward);
                } else {
                    on_scroll.emit(Direction::Backward);
                }
            })
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
                    id="panzoom__inner__0"
                    class="panzoom__inner"
                    style={style}
                >
                    { for ctx.props().children.iter() }
                </div>
            </content>
        }
    }
}

std::thread_local! {
    pub static PANZOOM_STATE: Delta<PanZoomState> = Default::default();
}

// Delta<PanZoomState>
pub struct PanZoom();

impl PanZoom {
    pub fn zoom_in() {
        PANZOOM_STATE.with(|s| s.emit(&TouchAction::MouseWheel(Default::default(), -20.0)));
    }

    pub fn zoom_out() {
        PANZOOM_STATE.with(|s| s.emit(&TouchAction::MouseWheel(Default::default(), 20.0)));
    }

    pub fn reset() {
        PANZOOM_STATE.with(|s| s.emit(&TouchAction::Reset));
    }

    pub fn register(callback: Callback<PanZoomState>) -> CallbackIdx {
        PANZOOM_STATE.with(|s| s.register(callback))
    }

    pub fn unregister(idx: CallbackIdx) {
        PANZOOM_STATE.with(|s| s.unregister(idx));
    }
}
