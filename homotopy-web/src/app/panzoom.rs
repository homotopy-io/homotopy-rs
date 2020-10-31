use closure::closure;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::Sum;
use std::ops::Add;
use std::ops::Sub;
use std::rc::Rc;
use yew::prelude::*;
use yew_functional::*;
use yew_functional_macro::functional_component;
use web_sys::Element;
use Default;

type Finger = i32;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
struct Point(i32, i32);

impl Add for Point {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Point(self.0 + other.0, self.1 + other.1)
    }
}

impl Sum for Point {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Default::default(), |acc, p| acc + p)
    }
}

impl Sub for Point {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Point(self.0 - other.0, self.1 - other.1)
    }
}

impl Point {
    fn scale(&self, scalar: f64) -> Self {
        Point(
            ((self.0 as f64) * scalar) as i32,
            ((self.1 as f64) * scalar) as i32,
        )
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
enum Action {
    TouchDown(Finger, Point),
    TouchMove(Finger, Point),
    TouchUp(Finger),
    MouseWheel(Point, f64),
    MouseDown,
    MouseMove(Point),
    MouseUp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct State {
    translate: Point,
    touch: HashMap<Finger, Point>,
    mouse: bool,
    scale: f64,
}

impl Default for State {
    fn default() -> Self {
        State {
            translate: Default::default(),
            touch: Default::default(),
            mouse: Default::default(),
            scale: 1.0,
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub children: Children,
}

#[functional_component]
pub fn panZoom(props: &Props) -> Html {
    let node_ref = use_ref(|| NodeRef::default());
    let (state, dispatch) = use_reducer(
        |prev: Rc<State>, action: Action| -> State {
            match action {
                Action::TouchDown(f, p) => State {
                    touch: {
                        let mut next = prev.touch.clone();
                        next.insert(f, p);
                        next
                    },
                    ..State::clone(&prev)
                },
                Action::TouchMove(f, d) => {
                    let mut touch = prev.touch.clone();
                    touch.get_mut(&f).map(|p| *p = *p + d);
                    if prev.touch.len() != 2 {
                        State {
                            touch,
                            ..State::clone(&prev)
                        }
                    } else {
                        let distance = |t: &HashMap<Finger, Point>| {
                            let p = t.get(&0).unwrap();
                            let q = t.get(&1).unwrap();
                            f64::max(
                                0.01,
                                (((p.0 - q.0).pow(2) + (p.1 - q.1).pow(2)) as f64).sqrt(),
                            )
                        };
                        let scale = prev.scale * (distance(&touch) / distance(&prev.touch));
                        let avg = touch.values().map(|x| *x).sum::<Point>().scale(0.5);
                        let zoom_point = avg.scale(1.0 / prev.scale);
                        let translate = prev.translate
                            + zoom_point.scale(prev.scale - scale)
                            + d.scale(0.5);
                        State {
                            touch,
                            translate,
                            scale,
                            ..State::clone(&prev)
                        }
                    }
                }
                Action::TouchUp(f) => State {
                    touch: {
                        let mut next = prev.touch.clone();
                        next.remove(&f);
                        next
                    },
                    ..State::clone(&prev)
                },
                Action::MouseWheel(p, d) => {
                    let scale = prev.scale * if d < 0.0 { 1.1 } else { 1.0 / 1.1 };
                    let zoom_point = p.scale(1.0 / prev.scale);
                    let translate = prev.translate + zoom_point.scale(prev.scale - scale);
                    State {
                        scale,
                        translate,
                        ..State::clone(&prev)
                    }
                }
                Action::MouseDown => State {
                    mouse: true,
                    ..State::clone(&prev)
                },
                Action::MouseMove(d) => if prev.mouse {
                    State {
                        translate: prev.translate + d,
                        ..State::clone(&prev)
                    }
                } else {
                    State::clone(&prev)
                }
                Action::MouseUp => State {
                    mouse: false,
                    ..State::clone(&prev)
                },
            }
        },
        Default::default(),
    );

    let cancel_callback = Callback::from(closure!(clone dispatch, |evt: PointerEvent| {
        match evt.pointer_type().as_str() {
            "touch" => dispatch(Action::TouchUp(evt.pointer_id())),
            _ => {}
        };
    }));

    html! {
        <div class="panzoom_child"
             style={format!(
                     "transform-origin: 0 0; transform:translate({x}px, {y}px) scale({s});",
                     x=state.translate.0,
                     y=state.translate.1,
                     s=state.scale
                   )}
             onpointerdown=Callback::from(closure!(clone dispatch, clone node_ref, |evt: PointerEvent| {
                 let boundingrect = node_ref.borrow().cast::<Element>().unwrap().get_bounding_client_rect();
                 let top = boundingrect.top();
                 let left = boundingrect.left();
                 let pos = Point(evt.client_x() - left as i32, evt.client_y() - top as i32);
                 match (evt.pointer_type().as_str(), evt.ctrl_key()) {
                     ("touch", _) => dispatch(Action::TouchDown(evt.pointer_id(), pos)),
                     ("mouse", true) => dispatch(Action::MouseDown),
                     _ => {}
                 };
             }))
             onpointerout=&cancel_callback
             onpointerleave=&cancel_callback
             onpointercancel=&cancel_callback
             onpointerup=Callback::from(closure!(clone dispatch, |evt: PointerEvent| {
                 match evt.pointer_type().as_str() {
                     "touch" => dispatch(Action::TouchUp(evt.pointer_id())),
                     "mouse" => dispatch(Action::MouseUp),
                     _ => {}
                 };
             }))
             onpointermove=Callback::from(closure!(clone dispatch, |evt: PointerEvent| {
                 let pos = Point(evt.movement_x(), evt.movement_y());
                 match evt.pointer_type().as_str() {
                     "touch" => dispatch(Action::TouchMove(evt.pointer_id(), pos)),
                     "mouse" => dispatch(Action::MouseMove(pos)),
                     _ => {}
                 };
             }))
             onwheel=Callback::from(closure!(clone dispatch, clone node_ref, |evt: WheelEvent| {
                 evt.prevent_default();
                 if evt.ctrl_key() {
                     let boundingrect = node_ref.borrow().cast::<Element>().unwrap().get_bounding_client_rect();
                     let top = boundingrect.top();
                     let left = boundingrect.left();
                     let pos = Point(evt.client_x() - left as i32, evt.client_y() - top as i32);
                     dispatch(Action::MouseWheel(pos, evt.delta_y()))
                 }
             }))
             ref=node_ref.borrow().clone()
        >
        { props.children.clone() }
        </div>
    }
}

