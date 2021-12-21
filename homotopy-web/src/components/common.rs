use std::fmt;

use euclid::default::{Point2D, Vector2D};
use web_sys::{DomRect, DomTokenList, Element, TouchList};
use yew::NodeRef;

pub type Finger = i32;
pub type Point = Point2D<f64>;
pub type Vector = Vector2D<f64>;

pub fn read_touch_list(touch_list: &'_ TouchList) -> impl Iterator<Item = (Finger, Point)> + '_ {
    (0..touch_list.length()).filter_map(move |i| {
        touch_list.item(i).map(|touch| {
            let finger = touch.identifier();
            let x = f64::from(touch.client_x());
            let y = f64::from(touch.client_y());
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

pub fn bounding_rect(node_ref: &NodeRef) -> Option<DomRect> {
    node_ref
        .cast::<Element>()
        .map(|el| el.get_bounding_client_rect())
}

pub fn node_midpoint(node_ref: &NodeRef) -> Option<Point> {
    let rect = bounding_rect(node_ref)?;
    let x = rect.left() + 0.5 * rect.width();
    let y = rect.top() + 0.5 * rect.height();
    Some((x, y).into())
}

pub fn document() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}

pub fn class_list(node_ref: &NodeRef) -> DomTokenList {
    node_ref.cast::<Element>().unwrap().class_list()
}

pub fn add_class<S: AsRef<str>>(node_ref: &NodeRef, class: S) {
    class_list(node_ref).add_1(class.as_ref()).unwrap();
}

pub fn remove_class<S: AsRef<str>>(node_ref: &NodeRef, class: S) {
    class_list(node_ref).remove_1(class.as_ref()).unwrap();
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Visibility {
    Visible,
    Hidden,
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "visibility: {}",
            match self {
                Self::Visible => "visible",
                Self::Hidden => "hidden",
            }
        )
    }
}

impl From<bool> for Visibility {
    fn from(b: bool) -> Self {
        if b {
            Self::Visible
        } else {
            Self::Hidden
        }
    }
}
