use yew::prelude::*;
use yew_macro::function_component;

use super::icon::{Icon, IconSize};

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub id: String,
    pub header: String,
    #[prop_or(false)]
    pub persistent: bool,
    pub children: Children,
}

#[function_component(Modal)]
pub fn modal(props: &Props) -> Html {
    html! {
        <div id={props.id.clone()} class="modal">
            if !props.persistent {
                <a href="#">
                    <div class="modal-close"></div>
                </a>
            }
            <div class="modal-dialog">
                <div class="modal-content">
                    <header>
                        <h2>{props.header.clone()}</h2>
                        if !props.persistent {
                            <a href="#" class="modal-button"><Icon name="close" size={IconSize::Icon24}/></a>
                        }
                    </header>
                    {props.children.clone()}
                </div>
            </div>
        </div>
    }
}
