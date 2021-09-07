use std::collections::HashMap;

use yew_agent::{Agent, AgentLink, Context, HandlerId};

use super::KeyStore;

pub enum Settings<S: KeyStore> {
    Subscribe(S::Key),
    Unsubscribe(S::Key),
    Update(S::Message),
}

pub struct SettingsAgent<S>
where
    S: KeyStore + 'static,
{
    link: AgentLink<Self>,
    store: S,
    handlers: HashMap<S::Key, Vec<HandlerId>>,
}

impl<S> Agent for SettingsAgent<S>
where
    S: KeyStore + 'static,
{
    type Input = Settings<S>;
    type Message = ();
    type Output = S::Message;
    type Reach = Context<Self>;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            store: Default::default(),
            handlers: HashMap::new(),
        }
    }

    fn update(&mut self, _: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            Settings::Subscribe(k) => {
                if let Some(handlers) = self.handlers.get_mut(&k) {
                    if !handlers.contains(&id) {
                        handlers.push(id);
                    }
                } else {
                    self.handlers.insert(k, vec![id]);
                }
                self.link.respond(id, self.store.get(k));
            }
            Settings::Unsubscribe(k) => {
                if let Some(handlers) = self.handlers.get_mut(&k) {
                    handlers.retain(|handler_id| *handler_id != id);
                }
            }
            Settings::Update(msg) => {
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
