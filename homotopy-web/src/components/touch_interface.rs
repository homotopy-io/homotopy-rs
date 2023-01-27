use closure::closure;
use yew::prelude::*;

use crate::components::{delta::State, node_midpoint, read_touch_list, Finger, Point};

#[derive(Debug, Clone)]
pub enum TouchAction {
    TouchUpdate(Vec<(Finger, Point)>),
    TouchMove(Vec<(Finger, Point)>),
    MouseWheel(Point, f64),
    MouseDown(bool, Point),
    MouseMove(bool, Point),
    MouseUp,
    Reset,
}

pub trait TouchInterface: Default + Clone + 'static {
    fn mouse_down(&mut self, alt_key: bool, point: Point);

    fn mouse_up(&mut self);

    fn mouse_move(&mut self, alt_key: bool, next: Point);

    fn mouse_wheel(&mut self, point: Point, delta: f64);

    fn touch_move(&mut self, touches: &[(Finger, Point)]);

    fn touch_update(&mut self, touches: &[(Finger, Point)]);

    fn reset(&mut self);

    fn on_mouse_move(callback: Callback<TouchAction>) -> Callback<MouseEvent> {
        callback.reform(|e: MouseEvent| {
            e.prevent_default();
            TouchAction::MouseMove(
                e.alt_key(),
                (f64::from(e.client_x()), f64::from(e.client_y())).into(),
            )
        })
    }

    fn on_mouse_up(callback: Callback<TouchAction>) -> Callback<MouseEvent> {
        callback.reform(closure!(|e: MouseEvent| {
            e.prevent_default();
            TouchAction::MouseUp
        }))
    }

    fn on_mouse_down(callback: Callback<TouchAction>) -> Callback<MouseEvent> {
        callback.reform(closure!(|e: MouseEvent| {
            TouchAction::MouseDown(
                e.alt_key(),
                (f64::from(e.client_x()), f64::from(e.client_y())).into(),
            )
        }))
    }

    fn on_wheel(node_ref: &NodeRef, callback: Callback<TouchAction>) -> Callback<WheelEvent> {
        let node_ref = node_ref.clone();
        callback.reform(move |e: WheelEvent| {
            e.prevent_default();

            let midpoint = node_midpoint(&node_ref).unwrap();

            // Offset the observed x and y by half the dimensions of the panzoom view to
            // account for centering (not required on mouse moves as that information is
            // only used relatively)
            let x = f64::from(e.client_x()) - midpoint.x;
            let y = f64::from(e.client_y()) - midpoint.y;

            TouchAction::MouseWheel((x, y).into(), e.delta_y())
        })
    }

    fn on_touch_move(node_ref: &NodeRef, callback: Callback<TouchAction>) -> Callback<TouchEvent> {
        let node_ref = node_ref.clone();
        callback.reform(closure!(|e: TouchEvent| {
            e.prevent_default();
            let midpoint = node_midpoint(&node_ref).unwrap();
            TouchAction::TouchMove(
                read_touch_list(&e.touches())
                    .map(|(finger, point)| (finger, (point - midpoint).to_point()))
                    .collect(),
            )
        }))
    }

    fn on_touch_update(
        node_ref: &NodeRef,
        callback: Callback<TouchAction>,
    ) -> Callback<TouchEvent> {
        let node_ref = node_ref.clone();
        callback.reform(closure!(|e: TouchEvent| {
            let midpoint = node_midpoint(&node_ref).unwrap();
            TouchAction::TouchUpdate(
                read_touch_list(&e.touches())
                    .map(|(finger, point)| (finger, (point - midpoint).to_point()))
                    .collect(),
            )
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
            TouchAction::MouseDown(alt_key, point) => self.mouse_down(*alt_key, *point),
            TouchAction::MouseUp => self.mouse_up(),
            TouchAction::MouseMove(alt_key, next) => self.mouse_move(*alt_key, *next),
            TouchAction::MouseWheel(point, delta) => self.mouse_wheel(*point, *delta),
            TouchAction::TouchMove(touches) => self.touch_move(touches),
            TouchAction::TouchUpdate(touches) => self.touch_update(touches),
            TouchAction::Reset => self.reset(),
        }
    }
}
