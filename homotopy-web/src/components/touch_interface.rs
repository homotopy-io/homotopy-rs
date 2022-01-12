use closure::closure;
use yew::prelude::*;

use crate::components::{
    delta::{Delta, State},
    node_midpoint, read_touch_list, Finger, Point,
};

#[derive(Debug, Clone)]
pub enum TouchAction {
    TouchUpdate(Vec<(Finger, Point)>),
    TouchMove(Vec<(Finger, Point)>),
    MouseWheel(Point, f64),
    MouseDown(Point),
    MouseMove(Point),
    MouseUp,
    Reset,
}

pub trait TouchInterface: Default + 'static {
    fn mouse_down(&mut self, point: Point);

    fn mouse_up(&mut self);

    fn mouse_move(&mut self, next: Point);

    fn mouse_wheel(&mut self, point: Point, delta: f64);

    fn touch_move(&mut self, touches: &[(Finger, Point)]);

    fn touch_update(&mut self, touches: &[(Finger, Point)]);

    fn reset(&mut self);

    fn on_mouse_move() -> Callback<MouseEvent> {
        let delta = Delta::<Self>::new();
        Callback::from(closure!(|e: MouseEvent| {
            e.prevent_default();
            delta.emit(TouchAction::MouseMove(
                (f64::from(e.client_x()), f64::from(e.client_y())).into(),
            ));
        }))
    }

    fn on_mouse_up() -> Callback<MouseEvent> {
        let delta = Delta::<Self>::new();
        Callback::from(closure!(|e: MouseEvent| {
            e.prevent_default();
            delta.emit(TouchAction::MouseUp);
        }))
    }

    fn on_mouse_down() -> Callback<MouseEvent> {
        let delta = Delta::<Self>::new();
        Callback::from(closure!(|e: MouseEvent| {
            delta.emit(TouchAction::MouseDown(
                (f64::from(e.client_x()), f64::from(e.client_y())).into(),
            ));
        }))
    }

    fn on_wheel(node_ref: &NodeRef) -> Callback<WheelEvent> {
        let delta = Delta::<Self>::new();
        let node_ref = node_ref.clone();
        Callback::from(move |e: WheelEvent| {
            e.prevent_default();

            let midpoint = node_midpoint(&node_ref).unwrap();

            // Offset the observed x and y by half the dimensions of the panzoom view to
            // account for centering (not required on mouse moves as that information is
            // only used relatively)
            let x = f64::from(e.client_x()) - midpoint.x;
            let y = f64::from(e.client_y()) - midpoint.y;

            delta.emit(TouchAction::MouseWheel((x, y).into(), e.delta_y()));
        })
    }

    fn on_touch_move(node_ref: &NodeRef) -> Callback<TouchEvent> {
        let delta = Delta::<Self>::new();
        let node_ref = node_ref.clone();
        Callback::from(closure!(|e: TouchEvent| {
            e.prevent_default();
            let midpoint = node_midpoint(&node_ref).unwrap();
            delta.emit(TouchAction::TouchMove(
                read_touch_list(&e.touches())
                    .map(|(finger, point)| (finger, (point - midpoint).to_point()))
                    .collect(),
            ));
        }))
    }

    fn on_touch_update(node_ref: &NodeRef) -> Callback<TouchEvent> {
        let delta = Delta::<Self>::new();
        let node_ref = node_ref.clone();
        Callback::from(closure!(|e: TouchEvent| {
            let midpoint = node_midpoint(&node_ref).unwrap();
            delta.emit(TouchAction::TouchUpdate(
                read_touch_list(&e.touches())
                    .map(|(finger, point)| (finger, (point - midpoint).to_point()))
                    .collect(),
            ));
        }))
    }
}

impl<T> State for T
where
    T: TouchInterface,
{
    type Action = TouchAction;

    fn update(&mut self, action: &Self::Action) {
        match action {
            TouchAction::MouseDown(point) => self.mouse_down(*point),
            TouchAction::MouseUp => self.mouse_up(),
            TouchAction::MouseMove(next) => self.mouse_move(*next),
            TouchAction::MouseWheel(point, delta) => self.mouse_wheel(*point, *delta),
            TouchAction::TouchMove(touches) => self.touch_move(touches),
            TouchAction::TouchUpdate(touches) => self.touch_update(touches),
            TouchAction::Reset => self.reset(),
        }
    }
}
