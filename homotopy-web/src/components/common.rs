use std::cell::RefCell;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

use yew::html::{Component, ComponentLink};

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

pub struct WeakComponentLink<COMP: Component>(Rc<RefCell<Option<ComponentLink<COMP>>>>);

impl<COMP: Component> Clone for WeakComponentLink<COMP> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<COMP: Component> Default for WeakComponentLink<COMP> {
    fn default() -> Self {
        Self(Rc::default())
    }
}

impl<COMP: Component> Deref for WeakComponentLink<COMP> {
    type Target = Rc<RefCell<Option<ComponentLink<COMP>>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<COMP: Component> PartialEq for WeakComponentLink<COMP> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
