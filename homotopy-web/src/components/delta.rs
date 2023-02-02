use std::cell::RefCell;

use homotopy_common::idx::{Idx, IdxVec};
use yew::callback::Callback;

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct CallbackIdx(usize);

impl Idx for CallbackIdx {
    fn index(&self) -> usize {
        self.0
    }

    fn new(index: usize) -> Self {
        Self(index)
    }
}

pub trait State: Default + Clone + Sized + 'static {
    type Action;

    fn update(&mut self, action: &Self::Action) -> bool;
}

#[derive(Default)]
pub struct Delta<T>(RefCell<DeltaInner<T>>)
where
    T: State;

#[derive(Default, Clone)]
pub struct DeltaInner<T>
where
    T: State,
{
    state: T,
    handlers: IdxVec<CallbackIdx, Option<Callback<T>>>,
}

impl<T> Delta<T>
where
    T: State,
{
    pub fn emit(&self, msg: &T::Action) {
        let state = {
            let mut inner = self.0.borrow_mut();
            // If the state update did not do anything, do not emit to handlers.
            if !inner.state.update(msg) {
                return;
            }
            inner.state.clone()
        };
        let handlers = {
            let inner = self.0.borrow();
            inner.handlers.clone()
        };
        for handler in handlers.values().flatten() {
            handler.emit(state.clone());
        }
    }

    // Instead of popping from the array, we use a tombstone system.
    // This does not invalidate other indexes.
    pub fn register(&self, callback: Callback<T>) -> CallbackIdx {
        let mut inner = self.0.borrow_mut();
        for (i, handler) in inner.handlers.iter_mut() {
            match handler {
                None => {
                    *handler = Some(callback);
                    return i;
                }
                Some(h) if *h == callback => {
                    return i;
                }
                Some(_) => {}
            }
        }
        inner.handlers.push(Some(callback))
    }

    pub fn unregister(&self, idx: CallbackIdx) {
        let mut inner = self.0.borrow_mut();
        if let Some(h) = inner.handlers.get_mut(idx) {
            *h = None;
        }
    }

    pub fn state(&self) -> T {
        self.0.borrow().state.clone()
    }
}
