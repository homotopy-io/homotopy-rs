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

            impl [<$name KeyStore>] {
                pub fn get(
                    &self,
                    k: [<$name Key>],
                ) -> [<$name Msg>] {
                    match k {
                        $([<$name Key>]::$key => {
                            [<$name Msg>]::$key(self.[<__ $key>].clone())
                        }),*
                    }
                }

                pub fn set(&mut self, msg: &[<$name Msg>]) {
                    match msg {
                        $([<$name Msg>]::$key(v) => {
                            self.[<__ $key>] = v.clone();
                        }),*
                    }
                }

                pub fn key_of(msg: &[<$name Msg>]) -> [<$name Key>] {
                    match msg {
                        $([<$name Msg>]::$key(_) => [<$name Key>]::$key),*
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
                handlers: homotopy_common::hash::FastHashMap<[<$name Key>], Vec<yew::callback::Callback<[<$name Msg>]>>>
            }

            thread_local! {
                static SETTINGS: std::cell::RefCell<$name> = Default::default();
            }

            impl $name {
                pub const ALL: &'static [[<$name Key>]] = &[
                    $([<$name Key>]::$key),*
                ];

                pub fn subscribe(keys: &[[<$name Key>]], callback: yew::callback::Callback<[<$name Msg>]>) {
                    let msgs: Vec<_> = SETTINGS.with(|s| {
                        let mut s = s.borrow_mut();
                        for key in keys.iter() {
                            if let Some(handlers) = s.handlers.get_mut(&key) {
                                if !handlers.contains(&callback) {
                                    handlers.push(callback.clone());
                                }
                            } else {
                                s.handlers.insert(*key, vec![callback.clone()]);
                            }
                        }
                        keys.iter().map(|key| s.store.get(*key)).collect()
                    });
                    for msg in &msgs {
                        Self::broadcast(msg);
                    }
                }

                pub fn broadcast(msg: &[<$name Msg>]) {
                    {
                        let key = [<$name KeyStore>]::key_of(msg);
                        SETTINGS.with(|s| {
                            let s = s.borrow();
                            if let Some(handlers) = s.handlers.get(&key) {
                                for handler in handlers {
                                    handler.emit(s.store.get(key));
                                }
                            }
                        })
                    }
                }

                $(
                    #[allow(unused)]
                    #[inline(always)]
                    pub fn [<set_ $key>](v: $ty) {
                        let msg = [<$name Msg>]::$key(v);
                        SETTINGS.with(|s| {
                            let mut s = s.borrow_mut();
                            s.store.set(&msg);
                        });
                        Self::broadcast(&msg)
                    }
                )*
            }
        }
    }
}
