use std::hash::Hash;

use serde::{Deserialize, Serialize};

pub trait KeyStore: Serialize + Deserialize<'static> + Default + Clone {
    type Key: Copy + Eq + Hash;
    type Message: Clone;

    fn get(&self, k: Self::Key) -> Self::Message;

    fn set(&mut self, msg: &Self::Message);

    fn key_of(msg: &Self::Message) -> Self::Key;
}

#[macro_export]
macro_rules! declare_settings {
    ($vis:vis struct $name:ident {
        $(
            $key:ident: $ty:ty,
        )*
    }) => {
        paste::paste! {
            #[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
            #[allow(non_snake_case)]
            $vis struct [<$name KeyStore>] {
                $(
                    [<__ $key>]: $ty
                ),*
            }

            #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
            #[allow(non_camel_case_types)]
            $vis enum [<$name Key>] {
                $(
                    #[allow(unused)]
                    $key
                ),*
            }

            #[derive(Clone, Debug)]
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
                    #[inline(always)]
                    pub fn [<get_ $key>](&self) -> &$ty {
                        &self.[<__ $key>]
                    }
                )*
            }

            pub struct $name {
                bridge: Box<dyn yew::agent::Bridge<
                    $crate::components::settings::SettingsAgent<[<$name KeyStore>]>
                >>,
            }

            impl $name {
                const ALL: &'static [[<$name Key>]] = &[
                    $([<$name Key>]::$key),*
                ];

                pub fn connect(callback: yew::callback::Callback<[<$name Msg>]>) -> Self {
                    use $crate::components::settings::SettingsAgent;
                    use yew::Bridged;

                    let bridge = SettingsAgent::<[<$name KeyStore>]>::bridge(callback);

                    $name {
                        bridge,
                    }
                }

                $(
                    #[allow(unused)]
                    pub fn subscribe(&mut self, keys: &[[<$name Key>]]) {
                        use $crate::components::settings::Settings;
                        for key in keys.iter().copied() {
                            self.bridge.send(Settings::Subscribe(key))
                        }
                    }

                    #[allow(unused)]
                    pub fn unsubscribe(&mut self, keys: &[[<$name Key>]]) {
                        use $crate::components::settings::Settings;
                        for key in keys.iter().copied() {
                            self.bridge.send(Settings::Unsubscribe(key))
                        }
                    }

                    #[allow(unused)]
                    #[inline(always)]
                    pub fn [<set_ $key>](&mut self, v: $ty) {
                        use $crate::components::settings::Settings;
                        self.bridge.send(Settings::Update([<$name Msg>]::$key(v)))
                    }
                )*
            }
        }
    }
}
