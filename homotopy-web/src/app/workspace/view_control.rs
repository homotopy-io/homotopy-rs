use yew::prelude::*;
use yew_functional::function_component;

use crate::app::{Icon, IconSize};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ViewEvent {
    ZoomIn,
    ZoomOut,
    Reset,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct ViewControlProps {
    pub handler: Callback<ViewEvent>,
}

#[function_component(ViewControl)]
pub fn view_control(props: &ViewControlProps) -> Html {
    html! {
        <div class="workspace__toolbar__segment">
            <span
                class="workspace__toolbar__button"
                onclick={props.handler.reform(|_| ViewEvent::ZoomIn)}
            >
                <Icon name="zoom_in" size={IconSize::Icon24} />
            </span>
            <span
                class="workspace__toolbar__button"
                onclick={props.handler.reform(|_| ViewEvent::ZoomOut)}
            >
                <Icon name="zoom_out" size={IconSize::Icon24} />
            </span>
            <span
                class="workspace__toolbar__button"
                onclick={props.handler.reform(|_| ViewEvent::Reset)}
            >
                <Icon name="filter_center_focus" size={IconSize::Icon24} />
            </span>

        </div>
    }
}
