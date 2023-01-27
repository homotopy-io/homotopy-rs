use std::cell::RefCell;

use yew::callback::Callback;

pub trait State: Default + Clone + Sized + 'static {
    type Action;

    fn update(&mut self, action: &Self::Action);
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
    handlers: Vec<Callback<T>>,
}

impl<T> Delta<T>
where
    T: State,
{
    pub fn emit(&self, msg: &T::Action) {
        let state = {
            let mut inner = self.0.borrow_mut();
            inner.state.update(msg);
            inner.state.clone()
        };
        let handlers = {
            let inner = self.0.borrow();
            inner.handlers.clone()
        };
        for handler in handlers {
            handler.emit(state.clone());
        }
    }

    pub fn register(&self, callback: Callback<T>) {
        let should_register = {
            let inner = self.0.borrow();
            !inner.handlers.contains(&callback)
        };
        if should_register {
            let mut inner = self.0.borrow_mut();
            inner.handlers.push(callback);
        }
    }

    pub fn state(&self) -> T {
        self.0.borrow().state.clone()
    }
}
