use crate::model::{Toast, ToastKind};
use im::Vector;
use yew::prelude::*;
use yew_functional::function_component;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    pub toasts: Vector<(usize, Toast)>,
}

#[function_component(Toaster)]
pub fn toaster(props: &Props) -> Html {
    let toasts: Html = props
        .toasts
        .iter()
        .map(|(_, toast)| {
            let class = match toast.kind {
                ToastKind::Success => "toaster__toast toaster__toast--success",
                ToastKind::Error => "toaster__toast toaster__toast--error",
            };

            html! {
                <div class={class}>
                    {toast.message.clone()}
                </div>
            }
        })
        .collect();

    html! {
        <div class="toaster">{toasts}</div>
    }
}
