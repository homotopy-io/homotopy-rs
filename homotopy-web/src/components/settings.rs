#[macro_export]
macro_rules! declare_settings {
    ($vis:vis struct $name:ident {
        $(
            $key:ident: $ty:ty = $default:expr,
        )*
    }) => {
        paste::paste! {
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

            #[derive(Default)]
            pub struct $name {}

            thread_local! {
                $(static [<SETTINGS_DELTA_$key:upper>]: $crate::components::delta::Delta<$ty> = $crate::components::delta::Delta::new($default);)*
            }

            impl $name {
                pub const ALL: &'static [[<$name Key>]] = &[
                    $([<$name Key>]::$key),*
                ];

                pub fn subscribe(keys: &[[<$name Key>]], callback: yew::callback::Callback<[<$name Msg>]>) -> Vec<$crate::components::delta::CallbackIdx> {
                    let mut idxs = Vec::with_capacity(keys.len());
                    for k in keys {
                        match k {
                            $([<$name Key>]::$key => {idxs.push([<SETTINGS_DELTA_$key:upper>].with(|s| s.register(callback.reform(|v| [<$name Msg>]::$key(v)))));})*
                        }
                    }
                    idxs
                }

                pub fn unsubscribe(keys: &[[<$name Key>]], idxs: &[$crate::components::delta::CallbackIdx]) {
                    assert_eq!(keys.len(), idxs.len(), "idx must correspond to keys");
                    for (k, &idx) in keys.iter().zip(idxs) {
                        match k {
                            $([<$name Key>]::$key => {[<SETTINGS_DELTA_$key:upper>].with(|s| s.unregister(idx));})*
                        }
                    }
                }

                $(
                    #[allow(unused)]
                    #[inline(always)]
                    pub fn [<set_ $key>](v: $ty) {
                        [<SETTINGS_DELTA_$key:upper>].with(|s| {
                            s.emit(&v);
                        });
                    }

                    #[allow(unused)]
                    #[inline(always)]
                    pub fn [<get_ $key>]() -> $ty {
                        [<SETTINGS_DELTA_$key:upper>].with(|s| {
                            s.state()
                        })
                    }
                )*
            }
        }
    }
}
