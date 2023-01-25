use std::hash::Hash;

use serde::{Deserialize, Serialize};

pub trait KeyStore: Serialize + Deserialize<'static> + Default + Clone {
    type Key: Copy + Eq + Hash + 'static;
    type Message: Clone;

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
                    #[allow(unused)]
                    #[inline(always)]
                    pub fn [<get_ $key>](&self) -> &$ty {
                        &self.[<__ $key>]
                    }
                )*
            }

            #[derive(Default)]
            pub struct $name {
                store: [<$name KeyStore>],
                handlers: homotopy_common::hash::FastHashMap<[<$name Key>], Vec<Box<yew::callback::Callback<[<$name Msg>]>>>>
            }

            static SETTINGS : std::sync::RwLock<$name> = std::sync::RwLock::new();

            impl $name {
                const ALL: &'static [[<$name Key>]] = &[
                    $([<$name Key>]::$key),*
                ];

                fn subscribe(keys: &[[<$name Key>]], callback: yew::callback::Callback<[<$name Msg>]>) {
                    for key in keys.iter().copied() {
                        {
                            let settings = SETTINGS.write().unwrap();
                            if let Some(handlers) = settings.handlers.get_mut(&key) {
                                handlers.push(callback);
                            } else {
                                settings.handlers.insert(key, vec![callback]);
                            }
                        }
                        {
                            let settings = SETTINGS.read().unwrap();
                            for handler in settings.handlers.as_ref().into_iter() {
                                handler.dispatch(settings.store.get(key));
                            }
                        }
                    }
                }

                fn broadcast(msg: &[<$name Msg>]) {
                    {
                        let settings = SETTINGS.read().unwrap();
                        if let Some(handlers) = settings.handlers.get(&[<$name Msg>]::key_of(msg)) {
                            for handler in handlers {
                                handler.dispatch(settings.store.get(msg));
                            }
                        }
                    }
                }

                $(
                    #[allow(unused)]
                    #[inline(always)]
                    pub fn [<set_ $key>](v: $ty) {
                        let msg = [<$name Msg>]::$key(v);
                        {
                            let settings = SETTINGS.write().unwrap();
                            settings.settings.set(msg);
                        }
                        Self::broadcast(msg)
                    }
                )*
            }
        }
    }
}
