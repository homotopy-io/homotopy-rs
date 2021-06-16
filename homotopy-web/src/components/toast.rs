use std::collections::VecDeque;
use std::fmt;

use gloo_timers::callback::Timeout;
use yew::prelude::*;
use yew_functional::function_component;

use super::WeakComponentLink;

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

        pub trait ToasterLink {
            $(fn $method<S: AsRef<str>>(&self, msg: S);)*
        }

        impl ToasterLink for ComponentLink<Toaster> {
            $(fn $method<S: AsRef<str>>(&self, msg: S) {
                self.send_message(
                    ToasterMsg::Toast(ToastProps {
                        message: msg.as_ref().to_owned(),
                        kind: ToastKind::$name,
                    }),
                );
            })*
        }

        impl ToasterLink for WeakComponentLink<Toaster> {
            $(fn $method<S: AsRef<str>>(&self, msg: S) {
                if let Some(ref toaster) = *self.borrow() {
                    toaster.$method(msg);
                }
            })*
        }
    }
}

declare_toast_kinds![
    (Success, toast_success, "success"),
    (Error, toast_error, "error"),
];

#[derive(Debug, Clone, PartialEq, Eq, Properties)]
pub struct ToastProps {
    pub kind: ToastKind,
    pub message: String,
}

#[function_component(Toast)]
fn toast(props: &ToastProps) -> Html {
    let class = format!("toaster__toast toaster__toast--{}", props.kind);
    html! {
        <div class={class}>{props.message.clone()}</div>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct ToasterProps {
    #[prop_or(1500)]
    pub timeout: u32,
    pub weak_link: WeakComponentLink<Toaster>,
}

pub enum ToasterMsg {
    Toast(ToastProps),
    Clear,
}

pub struct Toaster {
    props: ToasterProps,
    link: ComponentLink<Self>,
    toasts: VecDeque<ToastProps>,
}

impl Component for Toaster {
    type Properties = ToasterProps;
    type Message = ToasterMsg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        *props.weak_link.borrow_mut() = Some(link.clone());
        Self {
            props,
            link,
            toasts: VecDeque::new(),
        }
    }

    fn view(&self) -> Html {
        let toasts: Html = self
            .toasts
            .iter()
            .map(|props| {
                html! {
                    <Toast
                        kind={props.kind}
                        message={&props.message}
                    />
                }
            })
            .collect();

        html! {
            <div class="toaster">{toasts}</div>
        }
    }

    fn rendered(&mut self, _: bool) {}

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            ToasterMsg::Toast(props) => {
                self.toasts.push_back(props);
                {
                    let link = self.link.clone();
                    Timeout::new(self.props.timeout, move || {
                        link.send_message(ToasterMsg::Clear);
                    })
                    .forget();
                }
            }
            ToasterMsg::Clear => {
                self.toasts.pop_front();
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        false
    }
}
