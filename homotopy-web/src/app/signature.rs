use folder::FolderView;
use yew::prelude::*;
use yew_macro::function_component;

use crate::{
    app::sidebar::DrawerViewSize,
    model::proof::{Action, Signature},
};

mod folder;
mod item;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub signature: Signature,
    pub dispatch: Callback<Action>,
    pub drawer_view_size: DrawerViewSize,
}

#[function_component(SignatureView)]
pub fn signature_view(props: &Props) -> Html {
    // TODO: Add search
    // TODO: On mobile, drag to the side to delete
    html! {
        <FolderView
            dispatch={props.dispatch.clone()}
            signature={props.signature.clone()}
            drawer_view_size={props.drawer_view_size}
        />
    }
}
