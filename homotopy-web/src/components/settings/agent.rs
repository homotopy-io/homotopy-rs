use homotopy_common::hash::FastHashMap;
use serde::{Deserialize, Serialize};
use yew::Context;
use yew_agent::{HandlerId, Private, Public, Worker, WorkerLink};

use super::KeyStore;

#[derive(Serialize, Deserialize)]
pub enum SettingsInput<S: KeyStore> {
    Subscribe(S::Key),
    Unsubscribe(S::Key),
    Update(S::Message),
}

pub struct SettingsAgent<S>
where
    S: KeyStore + 'static,
{
    link: WorkerLink<Self>,
    store: S,
    handlers: FastHashMap<S::Key, Vec<HandlerId>>,
}

impl<S> Worker for SettingsAgent<S>
where
    S: KeyStore + 'static,
{
    type Input = SettingsInput<S>;
    type Message = ();
    type Output = S::Message;
    type Reach = Public<Self>;

    fn create(link: WorkerLink<Self>) -> Self {
        Self {
            link,
            store: Default::default(),
            handlers: Default::default(),
        }
    }

    fn update(&mut self, _: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            SettingsInput::Subscribe(k) => {
                if let Some(handlers) = self.handlers.get_mut(&k) {
                    if !handlers.contains(&id) {
                        handlers.push(id);
                    }
                } else {
                    self.handlers.insert(k, vec![id]);
                }
                self.link.respond(id, self.store.get(k));
            }
            SettingsInput::Unsubscribe(k) => {
                if let Some(handlers) = self.handlers.get_mut(&k) {
                    handlers.retain(|handler_id| *handler_id != id);
                }
            }
            SettingsInput::Update(msg) => {
                self.store.set(&msg);
                self.broadcast(&msg);
            }
        }
    }

    fn connected(&mut self, _: HandlerId) {}

    fn disconnected(&mut self, id: HandlerId) {
        for v in self.handlers.values_mut() {
            v.retain(|handler_id| *handler_id != id);
        }
    }
}

impl<S> SettingsAgent<S>
where
    S: KeyStore + 'static,
{
    fn broadcast(&self, msg: &S::Message) {
        if let Some(handlers) = self.handlers.get(&S::key_of(msg)) {
            for handler in handlers {
                self.link.respond(*handler, msg.clone());
            }
        }
    }
}
