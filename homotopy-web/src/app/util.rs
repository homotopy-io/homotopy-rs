use euclid::default::Point2D;
use web_sys::{DomRect, Element, TouchList};
use yew::NodeRef;

pub type Finger = i32;
type Point = Point2D<f64>;

pub fn read_touch_list<'a>(
    touch_list: &'a TouchList,
    node_ref: &NodeRef,
) -> impl Iterator<Item = (Finger, Point)> + 'a {
    let rect = bounding_rect(node_ref);
    let (rect_left, rect_top) = (rect.left(), rect.top());

    (0..touch_list.length()).filter_map(move |i| {
        touch_list.item(i).map(|touch| {
            let finger = touch.identifier();
            let x = f64::from(touch.client_x()) - rect_left;
            let y = f64::from(touch.client_y()) - rect_top;
            (finger, (x, y).into())
        })
    })
}

pub fn read_touch_list_abs(
    touch_list: &'_ TouchList,
) -> impl Iterator<Item = (Finger, Point)> + '_ {
    (0..touch_list.length()).filter_map(move |i| {
        touch_list.item(i).map(|touch| {
            let finger = touch.identifier();
            let x = f64::from(touch.client_x());
            let y = f64::from(touch.client_y());
            (finger, (x, y).into())
        })
    })
}

pub fn bounding_rect(node_ref: &NodeRef) -> DomRect {
    node_ref
        .cast::<Element>()
        .unwrap()
        .get_bounding_client_rect()
}
