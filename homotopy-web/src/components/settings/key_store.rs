use serde::{Deserialize, Serialize};

pub type Key = &'static str;

pub trait KeyStore: Serialize + Deserialize<'static> + Default + Clone {
    type Message: Clone;

    fn get<V: 'static>(&self, k: Key) -> Option<&V>;

    fn set<V: 'static>(&mut self, k: Key, v: V);

    fn key_of(msg: &Self::Message) -> Key;
}

macro_rules! declare_settings {
    ($vis:vis struct $name:ident {
        type Message = $msg:ident;
        $(
            const $setting:ident: $ty:ty = $key:ident;
        )*
    }) => {
        use std::any::{type_name, Any};
        use serde::{Serialize, Deserialize};

        #[derive(Serialize, Deserialize, Default, Clone)]
        #[allow(non_snake_case)]
        $vis struct $name {
            $(
                $key: $ty
            ),*
        }

        #[derive(Clone)]
        #[allow(non_camel_case_types)]
        $vis enum $msg {
            $(
                #[allow(unused)]
                $setting($ty)
            ),*
        }

        impl $name {
            $(
                #[allow(unused)]
                const $setting: $crate::components::settings::Key = stringify!($key);

                #[allow(unused)]
                $vis fn $key(v: $ty) -> $msg {
                    $msg::$setting(v)
                }
            )*
        }

        impl $crate::components::settings::KeyStore for $name {
            type Message = $msg;

            fn get<V: 'static>(
                &self,
                k: $crate::components::settings::Key,
            ) -> Option<&V>{
                match k {
                    $(stringify!($key) => (&self.$key as &dyn Any).downcast_ref(),)*
                    _ => None,
                }
            }

            fn set<V: 'static>(
                &mut self,
                k: $crate::components::settings::Key,
                v: V,
            ) {
                match k {
                    $(stringify!($key) => {
                        if let Some(store) = (&mut self.$key as &mut dyn Any).downcast_mut() {
                            *store = v;
                        } else {
                            log::warn!(
                                "Could not write value of type `{}` to setting\\
                                `{}` (type: `{}`)",
                                type_name::<V>(),
                                stringify!($key),
                                type_name::<$ty>(),
                            );
                        }
                    }),*
                    _ => {
                        log::warn!(
                            "Tried to write to key `{}` when no such key exists",
                            k,
                        );
                    },
                }
            }

            fn key_of(msg: &Self::Message) -> $crate::components::settings::Key {
                match msg {
                    $(Self::Message::$setting(_) => stringify!($key),)*
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
            type Message = ExampleSettingsMsg;
            const WIREFRAME: bool = renderer_wireframe;
            const SMOOTHING: bool = renderer_smoothing;
            const SEARCH_DEPTH: u32 = solver_search_depth;
            const SEED: Vec<u32> = global_seed;
        }
    }

    #[test]
    fn key_of_matches() {
        assert_eq!(
            ExampleSettings::key_of(&ExampleSettings::renderer_wireframe(true)),
            ExampleSettings::WIREFRAME,
        );
        assert_eq!(
            ExampleSettings::key_of(&ExampleSettings::renderer_smoothing(false)),
            ExampleSettings::SMOOTHING,
        );
        assert_eq!(
            ExampleSettings::key_of(&ExampleSettings::solver_search_depth(42)),
            ExampleSettings::SEARCH_DEPTH,
        );
        assert_eq!(
            ExampleSettings::key_of(&ExampleSettings::global_seed(vec![1, 2, 3, 4])),
            ExampleSettings::SEED,
        );
    }

    #[test]
    fn valid_write_updates() {
        let mut key_store = ExampleSettings::default();
        // Check we see the default initially
        assert_eq!(
            key_store.get(ExampleSettings::WIREFRAME).copied(),
            Some(false),
        );
        // Perform a valid update
        key_store.set(ExampleSettings::WIREFRAME, true);
        // Check we observe the change
        assert_eq!(
            key_store.get(ExampleSettings::WIREFRAME).copied(),
            Some(true),
        );
    }

    #[test]
    fn invalid_read_returns_none() {
        let key_store = ExampleSettings::default();
        assert_eq!(
            key_store.get::<u32>(ExampleSettings::WIREFRAME).copied(),
            None,
        );
    }

    #[test]
    fn invalid_write_ignored() {
        let mut key_store = ExampleSettings::default();
        // Check we see the default initially
        assert_eq!(
            key_store.get(ExampleSettings::WIREFRAME).copied(),
            Some(false),
        );
        // Perform invalid write
        key_store.set(ExampleSettings::WIREFRAME, 42);
        // Check we don't observe a change
        assert_eq!(
            key_store.get(ExampleSettings::WIREFRAME).copied(),
            Some(false),
        );
    }
}
