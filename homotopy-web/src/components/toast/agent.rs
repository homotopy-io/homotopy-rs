use std::collections::HashSet;
use yew::worker::{Agent, AgentLink, Context, HandlerId};

use super::Toast;

pub struct ToastAgent {
    link: AgentLink<Self>,
    handlers: HashSet<HandlerId>,
}

impl Agent for ToastAgent {
    type Reach = Context<Self>;
    type Message = ();
    type Input = Toast;
    type Output = Toast;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            handlers: HashSet::new(),
        }
    }

    fn update(&mut self, _: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _: HandlerId) {
        for handler in &self.handlers {
            self.link.respond(*handler, msg.clone());
        }
    }

    fn connected(&mut self, id: HandlerId) {
        if id.is_respondable() {
            self.handlers.insert(id);
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.handlers.remove(&id);
    }
}
