// use std::collections::HashMap;

// use serde::{Deserialize, Serialize};

use yew::prelude::*;
// use yew::worker::{Agent, AgentLink, Context, HandlerId};
use yew_functional::function_component;

use crate::components::drawer::Drawer;

/*
pub enum SettingsMsg {
    Subscribe(Key),
    Unsubscribe(Key),
}

pub struct SettingsAgent<S: KeyStore + 'static> {
    link: AgentLink<Self>,
    store: S,
    handlers: HashMap<Key, Vec<HandlerId>>,
}

impl<S: KeyStore> Agent for SettingsAgent<S> {
    type Reach = Context<Self>;
    type Message = (Key, Box<dyn Any + 'static>);
    type Input = SettingsMsg;
    type Output = ();

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            store: Default::default(),
            handlers: HashMap::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        let (k, v) = msg;

        if let Some(handler_set) = self.handlers.get(&k) {
            for handler in handler_set {
                self.link.respond(*handler, ());
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            SettingsMsg::Subscribe(k) => {
                if let Some(handler_set) = self.handlers.get_mut(&k) {
                    if !handler_set.contains(&id) {
                        handler_set.push(id);
                    }
                }
            }
            SettingsMsg::Unsubscribe(k) => {
                if let Some(handler_set) = self.handlers.get_mut(&k) {
                    handler_set.retain(|handler_id| *handler_id != id);
                }
            }
        }
    }

    fn connected(&mut self, _: HandlerId) {}

    fn disconnected(&mut self, id: HandlerId) {
        for (_, v) in self.handlers.iter_mut() {
            v.retain(|handler_id| *handler_id != id);
        }
    }
}
*/

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

#[function_component(SettingsView)]
pub fn settings_view(_: &Props) -> Html {
    html! {
        <Drawer title="Settings" class="settings">
            {"Hello, World!"}
        </Drawer>
    }
}
