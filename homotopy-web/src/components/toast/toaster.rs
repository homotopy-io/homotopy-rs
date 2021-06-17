use std::collections::VecDeque;

use gloo_timers::callback::Timeout;
use yew::prelude::*;

use super::{Toast, ToastAgent};

#[derive(Clone, PartialEq, Properties)]
pub struct ToasterProps {
    #[prop_or(1500)]
    pub timeout: u32,
}

pub enum ToasterMsg {
    Toast(Toast),
    Clear,
}

pub struct Toaster {
    props: ToasterProps,
    link: ComponentLink<Self>,
    toasts: VecDeque<Toast>,
    _bridge: Box<dyn Bridge<ToastAgent>>,
}

impl Component for Toaster {
    type Properties = ToasterProps;
    type Message = ToasterMsg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let bridge = ToastAgent::bridge(link.callback(ToasterMsg::Toast));
        Self {
            props,
            link,
            toasts: VecDeque::new(),
            _bridge: bridge,
        }
    }

    fn view(&self) -> Html {
        let toasts: Html = self
            .toasts
            .iter()
            .map(|props| {
                let class = format!("toaster__toast toaster__toast--{}", props.kind);
                html! {
                    <div class={class}>{props.message.clone()}</div>
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
