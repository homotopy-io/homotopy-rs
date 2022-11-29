#![allow(unused)]

use std::cell::RefCell;

use serde::{Deserialize, Serialize};
use yew::Context;
use yew_agent::{Dispatched, Dispatcher, HandlerId, Public, Worker, WorkerLink};

pub trait State: Default + Sized + 'static {
    type Action: Serialize + for<'de> Deserialize<'de>;

    fn update(&mut self, action: &Self::Action);
}

#[derive(Serialize, Deserialize)]
pub enum DeltaInput<T>
where
    T: State,
{
    Register,
    Emit(T::Action),
}

pub type DeltaCallback<T> = Box<dyn Fn(&DeltaAgent<T>, &<T as State>::Action)>;

pub struct DeltaAgent<T>
where
    T: State,
{
    link: WorkerLink<Self>,
    state: T,
    //handlers: Vec<DeltaCallback<T>>,
    handlers: Vec<HandlerId>,
}

impl<T> Worker for DeltaAgent<T>
where
    T: State,
{
    type Input = DeltaInput<T>;
    type Message = ();
    type Output = ();
    type Reach = Public<Self>;

    fn create(link: WorkerLink<Self>) -> Self {
        Self {
            link,
            state: Default::default(),
            handlers: vec![],
        }
    }

    fn update(&mut self, _: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            DeltaInput::Register => {
                self.handlers.push(id);
            }
            DeltaInput::Emit(msg) => {
                self.state.update(&msg);
                //TODO actually do callbacks
                /*
                for callback in &self.handlers {
                    callback(self, &msg);
                }
                */
            }
        }
    }

    fn connected(&mut self, _: HandlerId) {}

    fn disconnected(&mut self, _: HandlerId) {}
}

impl<T> DeltaAgent<T>
where
    T: State,
{
    pub fn state(&self) -> &T {
        &self.state
    }

    pub fn emit(&self, msg: T::Action) {
        self.link.send_input(DeltaInput::Emit(msg));
    }
}

pub struct Delta<T>(RefCell<Dispatcher<DeltaAgent<T>>>)
where
    T: State;

impl<T> Delta<T>
where
    T: State,
{
    pub fn new() -> Self {
        Self(RefCell::new(DeltaAgent::dispatcher()))
    }

    pub fn emit(&self, msg: T::Action) {
        self.0.borrow_mut().send(DeltaInput::Emit(msg));
    }

    pub fn register(&self, callback: DeltaCallback<T>) {
        //TODO fix this
        // DeltaInput::Register(callback)
        // Temp fix to make clippy shut up
        std::mem::drop(callback);
        self.0.borrow_mut().send(DeltaInput::Register);
    }
}
