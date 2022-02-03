use yew::prelude::*;

use crate::model::Proof;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub proof: Proof,
}

#[function_component(DebugView)]
pub fn debug_view(props: &Props) -> Html {
    let workspace = props.proof.workspace().cloned();
    let signature = props.proof.signature().clone();
    html! {
        <>
            <button onclick={Callback::from(move |_| log::debug!("Workspace: {:?}", workspace))}>{"Dump workspace"}</button>
            <button onclick={Callback::from(move |_| log::debug!("Signature: {:?}", signature))}>{"Dump signature"}</button>
        </>
    }
}
