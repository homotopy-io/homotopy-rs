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
        type Key = $key_ty:ident;
        type Message = $msg_ty:ident;
        $(
            $key:ident: $ty:ty;
        )*
    }) => {
        use serde::{Serialize, Deserialize};

        #[derive(Serialize, Deserialize, Default, Clone)]
        #[allow(non_snake_case)]
        $vis struct $name {
            $(
                $key: $ty
            ),*
        }

        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
        #[allow(non_camel_case_types)]
        $vis enum $key_ty {
            $(
                #[allow(unused)]
                $key
            ),*
        }

        #[derive(Clone, Debug)]
        #[allow(non_camel_case_types)]
        $vis enum $msg_ty {
            $(
                #[allow(unused)]
                $key($ty)
            ),*
        }

        impl $crate::components::settings::KeyStore for $name {
            type Key = $key_ty;
            type Message = $msg_ty;

            fn get(
                &self,
                k: Self::Key,
            ) -> Self::Message{
                match k {
                    $(Self::Key::$key => Self::Message::$key(self.$key.clone()),)*
                }
            }

            fn set(&mut self, msg: &Self::Message) {
                match msg {
                    $(Self::Message::$key(v) => { self.$key = v.clone(); }),*
                }
            }

            fn key_of(msg: &Self::Message) -> Self::Key {
                match msg {
                    $(Self::Message::$key(_) => Self::Key::$key,)*
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::KeyStore;

    declare_settings! {
        struct ExampleSettings {
            type Key = ExampleSettingsKey;
            type Message = ExampleSettingsMsg;

            renderer_wireframe: bool;
            renderer_smoothing: bool;
            solver_search_depth: u32;
            global_seed: Vec<u32>;
        }
    }

    #[test]
    fn key_of_matches() {
        assert_eq!(
            ExampleSettings::key_of(&ExampleSettingsMsg::renderer_wireframe(true)),
            ExampleSettingsKey::renderer_wireframe,
        );
        assert_eq!(
            ExampleSettings::key_of(&ExampleSettingsMsg::renderer_smoothing(false)),
            ExampleSettingsKey::renderer_smoothing,
        );
        assert_eq!(
            ExampleSettings::key_of(&ExampleSettingsMsg::solver_search_depth(42)),
            ExampleSettingsKey::solver_search_depth,
        );
        assert_eq!(
            ExampleSettings::key_of(&ExampleSettingsMsg::global_seed(vec![1, 2, 3, 4])),
            ExampleSettingsKey::global_seed,
        );
    }
}
