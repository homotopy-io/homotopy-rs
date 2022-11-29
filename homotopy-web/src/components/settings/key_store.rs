use std::hash::Hash;

use serde::{Deserialize, Serialize};

pub trait KeyStore: Serialize + Deserialize<'static> + Default + Clone {
    type Key: Copy + Eq + Serialize + for<'a> Deserialize<'a> + Hash + 'static;
    type Message: Clone + Serialize + for<'a> Deserialize<'a>;

    fn get(&self, k: Self::Key) -> Self::Message;

    fn set(&mut self, msg: &Self::Message);

    fn key_of(msg: &Self::Message) -> Self::Key;
}

pub trait Settings {
    type Store: KeyStore;

    const ALL: &'static [<Self::Store as KeyStore>::Key];

    fn connect(callback: yew::callback::Callback<<Self::Store as KeyStore>::Message>) -> Self;

    fn subscribe(&mut self, keys: &[<Self::Store as KeyStore>::Key]);

    fn unsubscribe(&mut self, keys: &[<Self::Store as KeyStore>::Key]);
}

pub type Store<S> = <S as Settings>::Store;

#[macro_export]
macro_rules! declare_settings {
    ($vis:vis struct $name:ident {
        $(
            $key:ident: $ty:ty = $default:expr,
        )*
    }) => {
        paste::paste! {
            #[derive(serde::Serialize, serde::Deserialize, Clone)]
            #[allow(non_snake_case)]
            $vis struct [<$name KeyStore>] {
                $(
                    [<__ $key>]: $ty
                ),*
            }

            #[allow(non_snake_case)]
            impl Default for [<$name KeyStore>] {
                fn default() -> Self {
                    Self {
                        $(
                            [<__ $key>]: $default
                        ),*
                    }
                }
            }

            #[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
            #[allow(non_camel_case_types)]
            $vis enum [<$name Key>] {
                $(
                    #[allow(unused)]
                    $key
                ),*
            }

            #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
            #[allow(non_camel_case_types)]
            $vis enum [<$name Msg>] {
                $(
                    #[allow(unused)]
                    $key($ty)
                ),*
            }

            impl $crate::components::settings::KeyStore for [<$name KeyStore>] {
                type Key = [<$name Key>];
                type Message = [<$name Msg>];

                fn get(
                    &self,
                    k: Self::Key,
                ) -> Self::Message{
                    match k {
                        $(Self::Key::$key => {
                            Self::Message::$key(self.[<__ $key>].clone())
                        }),*
                    }
                }

                fn set(&mut self, msg: &Self::Message) {
                    match msg {
                        $(Self::Message::$key(v) => {
                            self.[<__ $key>] = v.clone();
                        }),*
                    }
                }

                fn key_of(msg: &Self::Message) -> Self::Key {
                    match msg {
                        $(Self::Message::$key(_) => Self::Key::$key),*
                    }
                }
            }

            impl [<$name KeyStore>] {
                $(
                    #[allow(unused)]
                    #[inline(always)]
                    pub fn [<get_ $key>](&self) -> &$ty {
                        &self.[<__ $key>]
                    }
                )*
            }

            pub struct $name {
                bridge: Box<dyn yew_agent::Bridge<
                    $crate::components::settings::SettingsAgent<[<$name KeyStore>]>
                >>,
            }

            pub struct [<$name Dispatch>] {
                dispatch: std::cell::RefCell<yew_agent::Dispatcher<
                    $crate::components::settings::SettingsAgent<[<$name KeyStore>]>
                >>,
            }

            impl $crate::components::settings::Settings for $name {
                type Store = [<$name KeyStore>];

                const ALL: &'static [[<$name Key>]] = &[
                    $([<$name Key>]::$key),*
                ];

                fn connect(callback: yew::callback::Callback<[<$name Msg>]>) -> Self {
                    use $crate::components::settings::SettingsAgent;
                    use yew_agent::Bridged;

                    let bridge = SettingsAgent::<[<$name KeyStore>]>::bridge(
                        std::rc::Rc::new((|m| callback.emit(m)))
                    );

                    Self {
                        bridge,
                    }
                }

                fn subscribe(&mut self, keys: &[[<$name Key>]]) {
                    use $crate::components::settings::SettingsInput;
                    for key in keys.iter().copied() {
                        self.bridge.send(SettingsInput::Subscribe(key))
                    }
                }

                fn unsubscribe(&mut self, keys: &[[<$name Key>]]) {
                    use $crate::components::settings::SettingsInput;
                    for key in keys.iter().copied() {
                        self.bridge.send(SettingsInput::Unsubscribe(key))
                    }
                }
            }

            impl $name {
                $(
                    #[allow(unused)]
                    #[inline(always)]
                    pub fn [<set_ $key>](&mut self, v: $ty) {
                        use $crate::components::settings::SettingsInput;
                        self.bridge.send(SettingsInput::Update([<$name Msg>]::$key(v)))
                    }
                )*
            }

            impl [<$name Dispatch>] {
                fn new() -> Self {
                    use $crate::components::settings::SettingsAgent;
                    use yew_agent::Dispatched;

                    Self {
                        dispatch: std::cell::RefCell::new(
                            SettingsAgent::<[<$name KeyStore>]>::dispatcher()
                        ),
                    }
                }

                $(
                    #[allow(unused)]
                    #[inline(always)]
                    pub fn [<set_ $key>](&self, v: $ty) {
                        use $crate::components::settings::SettingsInput;
                        self.dispatch.borrow_mut()
                            .send(SettingsInput::Update([<$name Msg>]::$key(v)))
                    }
                )*
            }
        }
    }
}
