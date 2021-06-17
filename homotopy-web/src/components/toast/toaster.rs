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
    toasts: Vec<Toast>,
    animating: usize,
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
            toasts: vec![],
            animating: 0,
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

    fn rendered(&mut self, _: bool) {}

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            ToasterMsg::Toast(props) => {
                self.animating += 1;
                self.toasts.push(props);
                {
                    let link = self.link.clone();
                    Timeout::new(self.props.timeout, move || {
                        link.send_message(ToasterMsg::Clear);
                    })
                    .forget();
                    true
                }
            }
            ToasterMsg::Clear if self.animating > 1 => {
                self.animating -= 1;
                false
            }
            ToasterMsg::Clear => {
                // Batch clear toasts when none are left animating
                self.animating = 0;
                self.toasts.clear();
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        false
    }
}
