use homotopy_common::hash::FastHashMap;

use super::KeyStore;

pub enum SettingsInput<S: KeyStore> {
    Subscribe(S::Key),
    Unsubscribe(S::Key),
    Update(S::Message),
}

pub struct SettingsAgent<S>
where
    S: KeyStore + 'static,
{
    link: (),
    store: S,
    handlers: FastHashMap<S::Key, Vec<()>>,
}

impl<S> SettingsAgent<S>
where
    S: KeyStore + 'static,
{
    type Input = SettingsInput<S>;
    type Message = ();
    type Output = S::Message;

    fn create(link: ()) -> Self {
        Self {
            link,
            store: Default::default(),
            handlers: Default::default(),
        }
    }

    fn update(&mut self, _: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, id: ()) {
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

    fn connected(&mut self, _: ()) {}

    fn disconnected(&mut self, id: ()) {
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
