use wasm_bindgen::JsValue;
use yew::prelude::*;

use crate::model::{dump_actions, Proof};

#[derive(Properties, Clone, PartialEq, Eq)]
pub struct Props {
    pub proof: Proof,
}

#[function_component(DebugView)]
pub fn debug_view(props: &Props) -> Html {
    let diagram = props
        .proof
        .workspace()
        .cloned()
        .map(|ws| ws.visible_diagram());
    let signature = props.proof.signature().clone();
    html! {
        <>
            <button onclick={Callback::from(move |_| web_sys::console::dir_2(&"Workspace diagram:".into(), &JsValue::from_serde(&diagram).unwrap()))}>{"Dump workspace diagram"}</button>
            <br />
            <button onclick={Callback::from(move |_| web_sys::console::dir_2(&"Signature:".into(), &JsValue::from_serde(&signature).unwrap()))}>{"Dump signature"}</button>
            <br />
            <button onclick={Callback::from(move |_| web_sys::console::dir_2(&"Actions:".into(), &dump_actions()))}>{"Dump actions"}</button>
        </>
    }
}
