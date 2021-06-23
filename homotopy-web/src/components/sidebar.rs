use yew::prelude::*;
use yew_functional::function_component;

use super::common::Visibility;
use super::icon::{Icon, IconSize};
use crate::model;

#[derive(Properties, Clone, PartialEq)]
pub struct SidebarButtonDesc {
    pub label: &'static str,
    pub icon: &'static str,
    pub action: model::Action,
    pub shortcut: Option<char>,
}

#[derive(Properties, Clone, PartialEq)]
pub struct SidebarButtonProps {
    pub desc: SidebarButtonDesc,
    pub dispatch: Callback<model::Action>,
    #[prop_or(Visibility::Visible)]
    pub visibility: Visibility,
}

#[function_component(SidebarButton)]
pub fn sidebar_button(props: &SidebarButtonProps) -> Html {
    let action = props.desc.action.clone();

    html! {
        <div
            class="sidebar__button tooltip tooltip--right"
            onclick={props.dispatch.reform(move |_| action.clone())}
            data-tooltip={
                if let Some(shortcut) = props.desc.shortcut {
                    format!("{} ({})", props.desc.label, shortcut.to_uppercase())
                } else {
                    props.desc.label.to_owned()
                }
            }
            style={&format!("{}", props.visibility)}
        >
            <Icon name={props.desc.icon} size={IconSize::Icon24} />
        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct SidebarDrawerProps {}
