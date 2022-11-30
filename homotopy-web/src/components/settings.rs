use std::hash::Hash;

use serde::{Deserialize, Serialize};

pub trait KeyStore: Serialize + Deserialize<'static> + Default + Clone {
    type Key: Copy + Eq + Serialize + for<'a> Deserialize<'a> + Hash + 'static;
    type Message: Clone + Serialize + for<'a> Deserialize<'a>;

    fn get(&self, k: Self::Key) -> Self::Message;

    fn set(&mut self, msg: &Self::Message);

    fn key_of(msg: &Self::Message) -> Self::Key;
}

#[macro_export]
macro_rules! declare_settings {
    ($vis:vis struct $name:ident {
        $(
            $key:ident: $ty:ty = $default:expr,
        )*
    }) => {
        paste::paste! {
            #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
            #[allow(non_snake_case)]
            $vis struct $name {
                $(
                    [<__ $key>]: $ty
                ),*
            }

            #[allow(non_snake_case)]
            impl Default for $name {
                fn default() -> Self {
                    Self {
                        $(
                            [<__ $key>]: $default
                        ),*
                    }
                }
            }
            #[derive(Debug, Clone, PartialEq)]
            #[allow(non_snake_case)]
            $vis struct [<$name Dispatch>] {
                pub inner: $name,
                pub dispatch: yew::callback::Callback<[<$name Msg>]>
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

            impl $crate::components::settings::KeyStore for $name {
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

            impl $name {
                $(
                    #[allow(unused)]
                    #[inline(always)]
                    pub fn [<get_ $key>](&self) -> &$ty {
                        &self.[<__ $key>]
                    }
                )*
           }

           impl [<$name Dispatch>] {
               #[allow(unused)]
               pub fn new(inner: $name, dispatch: yew::callback::Callback<[<$name Msg>]>) -> Self {
                   Self {
                       inner,
                       dispatch
                   }
               }

               $(
                   #[allow(unused)]
                   #[inline(always)]
                   pub fn [<set_ $key>](&self, v: $ty) {
                       self.dispatch.emit([<$name Msg>]::$key(v));
                   }
               )*
           }
        }
    }
}
