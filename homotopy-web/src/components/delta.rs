#![allow(unused)]

use std::cell::RefCell;

use yew::agent::{Agent, AgentLink, Context, Dispatched, Dispatcher, HandlerId};

pub trait State: Default + Sized + 'static {
    type Action;

    fn update(&mut self, action: &Self::Action);
}

pub enum DeltaInput<T>
where
    T: State,
{
    Register(DeltaCallback<T>),
    Emit(T::Action),
}

pub type DeltaCallback<T> = Box<dyn Fn(&DeltaAgent<T>, &<T as State>::Action)>;

pub struct DeltaAgent<T>
where
    T: State,
{
    link: AgentLink<Self>,
    state: T,
    handlers: Vec<DeltaCallback<T>>,
}

impl<T> Agent for DeltaAgent<T>
where
    T: State,
{
    type Reach = Context<Self>;
    type Message = ();
    type Input = DeltaInput<T>;
    type Output = ();

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            state: Default::default(),
            handlers: vec![],
        }
    }

    fn update(&mut self, _: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            DeltaInput::Register(callback) => {
                self.handlers.push(callback);
            }
            DeltaInput::Emit(msg) => {
                self.state.update(&msg);
                for callback in &self.handlers {
                    callback(self, &msg);
                }
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
        self.0.borrow_mut().send(DeltaInput::Register(callback));
    }
}
