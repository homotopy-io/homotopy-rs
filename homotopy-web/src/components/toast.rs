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

declare_toast_kinds![(Success, success, "success"), (Error, error, "error"),];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    pub kind: ToastKind,
    pub message: String,
}

#[derive(Clone, PartialEq, Properties)]
pub struct ToasterProps {
    #[prop_or(1500)]
    pub timeout: u32,
}

#[derive(Clone)]
pub enum ToasterMsg {
    Toast(Toast),
    Clear,
}

#[derive(Default)]
pub struct ToasterState {
    toasts: Vec<Toast>,
    animating: usize,
}

impl State for ToasterState {
    type Action = ToasterMsg;

    fn update(&mut self, action: &Self::Action) {
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
        }
    }
}

pub struct ToasterComponent {
    props: ToasterProps,
    _delta: Delta<ToasterState>,
    state: ToasterState,
}

impl Component for ToasterComponent {
    type Properties = ToasterProps;
    type Message = ToasterMsg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let delta = Delta::new();
        delta.register({
            let timeout = props.timeout;
            Box::new(move |_, e: &ToasterMsg| {
                if let ToasterMsg::Toast(_) = e {
                    let link = link.clone();
                    Timeout::new(timeout, move || {
                        link.send_message(ToasterMsg::Clear);
                    })
                    .forget();
                }

                link.send_message(e.clone());
            })
        });

        Self {
            props,
            _delta: delta,
            state: Default::default(),
        }
    }

    fn view(&self) -> Html {
        let toasts: Html = self
            .state
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
            .collect();

        html! {
            <div class="toaster">{toasts}</div>
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.state.update(&msg);
        match msg {
            ToasterMsg::Clear if self.state.animating > 1 => false,
            _ => true,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        false
    }
}

pub struct Toaster(Delta<ToasterState>);

impl Toaster {
    pub fn new() -> Self {
        Self(Delta::new())
    }

    pub fn toast(&mut self, toast: Toast) {
        self.0.emit(ToasterMsg::Toast(toast))
    }
}
