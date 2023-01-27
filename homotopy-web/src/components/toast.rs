use std::fmt;

use gloo_timers::callback::Timeout;
use yew::prelude::*;

use crate::components::delta::{Delta, State};

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

declare_toast_kinds![
    (Success, success, "success"),
    (Warning, warn, "warning"),
    (Error, error, "error"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    pub kind: ToastKind,
    pub message: String,
}

#[derive(Clone, Properties, PartialEq, Eq)]
pub struct ToasterProps {
    #[prop_or(1500)]
    pub timeout: u32,
}

#[derive(Default, Clone)]
pub enum ToasterMsg {
    Toast(Toast),
    Clear,
    SetTimer,
    #[default]
    Noop,
}

#[derive(Default, Clone)]
pub struct ToasterState {
    toasts: Vec<Toast>,
    animating: usize,
    last_msg: ToasterMsg,
}

impl State for ToasterState {
    type Action = ToasterMsg;

    fn update(&mut self, action: &Self::Action) {
        self.last_msg = action.clone();
        match action {
            ToasterMsg::Toast(props) => {
                self.animating += 1;
                self.toasts.push(props.clone());
            }
            ToasterMsg::Clear if self.animating > 1 => {
                self.animating -= 1;
            }
            ToasterMsg::Clear => {
                // Batch clear toasts when none are left animating
                self.animating = 0;
                self.toasts.clear();
            }
            _ => {}
        }
    }
}

std::thread_local! {
    pub static TOASTER: Delta<ToasterState> = Default::default();
}

pub struct ToasterComponent {}

impl Component for ToasterComponent {
    type Message = ToasterMsg;
    type Properties = ToasterProps;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        TOASTER.with(move |t| {
            t.register(link.callback(|e: ToasterState| {
                if let ToasterMsg::Toast(_) = e.last_msg {
                    ToasterMsg::SetTimer
                } else {
                    e.last_msg
                }
            }));
        });

        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        TOASTER
            .with(Delta::state)
            .toasts
            .iter()
            .map(|props| {
                let class = format!("toaster__toast toaster__toast--{}", props.kind);
                html! {
                    <div class={class}>
                        <div class="toaster__toast__inner">
                            {props.message.clone()}
                        </div>
                    </div>
                }
            })
            .collect()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ToasterMsg::SetTimer => {
                let timeout = ctx.props().timeout;
                Timeout::new(timeout, move || {
                    TOASTER.with(|t| t.emit(&ToasterMsg::Clear));
                })
                .forget();
                true
            }
            ToasterMsg::Clear => !TOASTER.with(|t| t.state().animating > 1),
            _ => false,
        }
    }
}

pub fn toast(toast: Toast) {
    TOASTER.with(|t| t.emit(&ToasterMsg::Toast(toast)));
}
