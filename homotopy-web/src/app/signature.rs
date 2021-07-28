mod folder;
mod generator;

use crate::model::proof::{Action, Signature};

use folder::FolderView;
use yew::prelude::*;
use yew_macro::function_component;

const COLOR_RGB: [[[u8; 3]; 4]; 2] = [
    [
        [41, 128, 185],
        [192, 57, 43],
        [243, 156, 18],
        [142, 68, 173],
    ],
    [[39, 174, 96], [241, 196, 15], [255, 255, 255], [0, 0, 0]],
];

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub signature: Signature,
    pub dispatch: Callback<Action>,
}

#[function_component(SignatureView)]
pub fn signature_view(props: &Props) -> Html {
    // TODO: Add search
    // TODO: On mobile, drag to the side to delete
    html! {
        <FolderView
            dispatch={props.dispatch.clone()}
            contents={props.signature.as_tree()}
        />
    }
}
