use closure::closure;
use yew::prelude::*;

use euclid::default::{Point2D, Vector2D};

use super::util::{bounding_rect, read_touch_list, Finger};

type Point = Point2D<f64>;
type Vector = Vector2D<f64>;

#[derive(Debug, Clone)]
pub enum Message {
    TouchUpdate(Vec<(Finger, Point)>),
    TouchMove(Vec<(Finger, Point)>),
    MouseWheel(Point, f64),
    MouseDown(Point),
    MouseMove(Point),
    MouseUp,
    Reset,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PanZoom {
    translate: Vector,
    scale: f64,
    mouse: Option<Point>,
    touches: Vec<(Finger, Point)>,
    node_ref: NodeRef,
    on_mouse_move: Callback<MouseEvent>,
    on_mouse_down: Callback<MouseEvent>,
    on_mouse_up: Callback<MouseEvent>,
    on_wheel: Callback<WheelEvent>,
    on_touch_move: Callback<TouchEvent>,
    on_touch_update: Callback<TouchEvent>,
}

impl PanZoom {
    pub fn new(node_ref: NodeRef, callback: &Callback<Message>) -> Self {
        let on_mouse_down = Callback::from(closure!(clone callback, |e: MouseEvent| {
            e.prevent_default();
            if e.alt_key() {
                let x = f64::from(e.client_x());
                let y = f64::from(e.client_y());
                callback.emit(Message::MouseDown((x, y).into()));
            }
        }));

        let on_mouse_move = Callback::from(closure!(clone callback, |e: MouseEvent| {
            e.prevent_default();
            let x = f64::from(e.client_x());
            let y = f64::from(e.client_y());
            callback.emit(Message::MouseMove((x, y).into()));
        }));

        let on_mouse_up = Callback::from(closure!(clone callback, |e: MouseEvent| {
            e.prevent_default();
            callback.emit(Message::MouseUp);
        }));

        let on_wheel = Callback::from(closure!(clone callback, clone node_ref, |e: WheelEvent| {
            if e.alt_key() {
                e.prevent_default();
                let rect = bounding_rect(&node_ref);
                let x = f64::from(e.client_x()) - rect.left();
                let y = f64::from(e.client_y()) - rect.top();
                let delta = e.delta_y();
                callback.emit(Message::MouseWheel((x, y).into(), delta));
            }
        }));

        let on_touch_move =
            Callback::from(closure!(clone callback, clone node_ref, |e: TouchEvent| {
                e.prevent_default();
                let touches = read_touch_list(&e.touches(), &node_ref).collect();
                callback.emit(Message::TouchMove(touches));
            }));

        let on_touch_update =
            Callback::from(closure!(clone callback, clone node_ref, |e: TouchEvent| {
                // e.prevent_default();
                let touches = read_touch_list(&e.touches(), &node_ref).collect();
                callback.emit(Message::TouchUpdate(touches));
            }));

        Self {
            node_ref,
            on_mouse_down,
            on_mouse_move,
            on_mouse_up,
            on_wheel,
            on_touch_move,
            on_touch_update,
            translate: Vector::zero(),
            scale: 1.0,
            mouse: Default::default(),
            touches: Default::default(),
        }
    }

    pub fn update(&mut self, msg: Message) -> bool {
        match msg {
            Message::MouseDown(point) => {
                self.mouse = Some(point);
                false
            }
            Message::MouseUp => {
                self.mouse = None;
                false
            }
            Message::MouseMove(next) => match self.mouse {
                Some(prev) => {
                    self.translate += next - prev;
                    self.mouse = Some(next);
                    true
                }
                None => false,
            },
            Message::MouseWheel(point, delta) => {
                let scale = if delta < 0.0 { 1.1 } else { 1.0 / 1.1 };
                self.translate =
                    (point + self.translate).to_vector() / (scale * self.scale) - point.to_vector();
                self.scale *= scale;
                true
            }
            Message::TouchMove(mut touches) => {
                touches.sort_by_key(|(finger, _)| *finger);

                if touches.len() != 2 || self.touches.len() != 2 {
                    self.touches = touches;
                    return false;
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
                true
            }
            Message::TouchUpdate(mut touches) => {
                touches.sort_by_key(|(finger, _)| *finger);
                self.touches = touches;
                false
            }
            Message::Reset => {
                self.translate = Default::default();
                self.scale = 1.0;
                true
            }
        }
    }

    pub fn translate(&self) -> Vector {
        self.translate
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn node_ref(&self) -> NodeRef {
        self.node_ref.clone()
    }

    /// Event listener for the `mousemove` event.
    pub fn on_mouse_move(&self) -> Callback<MouseEvent> {
        self.on_mouse_move.clone()
    }

    /// Event listener for the `mousedown` event.
    pub fn on_mouse_down(&self) -> Callback<MouseEvent> {
        self.on_mouse_down.clone()
    }

    /// Event listener for the `mouseup` event.
    pub fn on_mouse_up(&self) -> Callback<MouseEvent> {
        self.on_mouse_up.clone()
    }

    /// Event listener for the `wheel` event.
    pub fn on_wheel(&self) -> Callback<WheelEvent> {
        self.on_wheel.clone()
    }

    /// Event listener for the `touchmove` event.
    pub fn on_touch_move(&self) -> Callback<TouchEvent> {
        self.on_touch_move.clone()
    }

    /// Event listener for the `touchstart`, `touchend` and `touchcancel` events.
    pub fn on_touch_update(&self) -> Callback<TouchEvent> {
        self.on_touch_update.clone()
    }
}
