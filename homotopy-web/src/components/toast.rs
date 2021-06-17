use std::fmt;

mod agent;
mod toaster;

pub use agent::*;
pub use toaster::*;

macro_rules! declare_toast_kinds {
    ($(($name:ident, $method:ident, $class:literal),)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum ToastKind {
            $($name),*
        }

        impl fmt::Display for ToastKind {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $(Self::$name => write!(f, $class)),*
                }
            }
        }

        impl Toast {
            $(
                #[allow(unused)]
                pub fn $method<S: AsRef<str>>(s: S) -> Self {
                    Self {
                        kind: ToastKind::$name,
                        message: s.as_ref().to_owned(),
                    }
                }
            )*
        }
    }
}

declare_toast_kinds![(Success, success, "success"), (Error, error, "error"),];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    pub kind: ToastKind,
    pub message: String,
}
