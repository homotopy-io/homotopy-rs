use folder::FolderView;
use homotopy_model::proof;
use yew::prelude::*;
use yew_macro::function_component;

use crate::{
    app::sidebar::DrawerViewSize,
    model::{proof::Signature, Action},
};

mod folder;
mod item;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub signature: Signature,
    pub dispatch: Callback<Action>,
    pub drawer_view_size: DrawerViewSize,
}

#[function_component]
pub fn SignatureView(props: &Props) -> Html {
    // TODO: Add search
    // TODO: On mobile, drag to the side to delete
    let suspension_controls = if props.signature.has_generators() {
        html! {
            <div>
                <button onclick={props.dispatch.reform(move |_| proof::Action::SuspendSignature.into())}>{"Suspend"}</button>
            </div>
        }
    } else {
        Default::default()
    };
    html! {
        <div>
            {suspension_controls}
            <FolderView
                dispatch={props.dispatch.clone()}
                signature={props.signature.clone()}
                drawer_view_size={props.drawer_view_size}
            />
        </div>
    }
}
