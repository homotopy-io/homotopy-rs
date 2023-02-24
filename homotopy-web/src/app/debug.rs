use closure::closure;
use homotopy_model::proof;
use web_sys::{File, HtmlInputElement};
use yew::prelude::*;

use crate::model::{proof::Workspace, Action, Proof};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    ImportActions(File),
    Noop,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub proof: Proof,
    pub dispatch: Callback<Action>,
}

#[derive(Debug, Default)]
pub struct DebugView {
    reader: Option<gloo::file::callbacks::FileReader>,
}

impl Component for DebugView {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let import = ctx.link().callback(|e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(filelist) = input.files() {
                Msg::ImportActions(filelist.get(0).unwrap())
            } else {
                Msg::Noop
            }
        });
        let diagram = ctx
            .props()
            .proof
            .workspace
            .as_ref()
            .map(Workspace::visible_diagram);
        let signature = ctx.props().proof.signature.clone();
        html! {
            <>
                <div>
                    <button onclick={ctx.props().dispatch.reform(move |_| proof::Action::Suspend(proof::SuspensionKind::Abelian).into())}>{"Abelianize"}</button>
                    <button onclick={ctx.props().dispatch.reform(move |_| proof::Action::Suspend(proof::SuspensionKind::Standard).into())}>{"Suspend"}</button>
                    <button onclick={ctx.props().dispatch.reform(move |_| proof::Action::Suspend(proof::SuspensionKind::Reduced).into())}>{"Suspend Reduced"}</button>
                </div>
                <div>
                    <button onclick={Callback::from(move |_| web_sys::console::dir_2(&"Workspace diagram:".into(), &serde_wasm_bindgen::to_value(&diagram).unwrap()))}>{"Dump workspace diagram"}</button>
                </div>
                <div>
                    <button onclick={Callback::from(move |_| web_sys::console::dir_2(&"Signature:".into(), &serde_wasm_bindgen::to_value(&signature).unwrap()))}>{"Dump signature"}</button>
                </div>
                <div>
                    <button onclick={ctx.props().dispatch.reform(move |_| Action::ExportActions)}>{"Export actions"}</button>
                    <label for="import" class="button">{"Import actions"}</label>
                    <input type="file" accept=".json,.txt" class="visually-hidden" id="import" onchange={import}/>
                </div>
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let dispatch = &ctx.props().dispatch;
        match msg {
            Msg::ImportActions(file) => {
                let task = gloo::file::callbacks::read_as_bytes(
                    &file.into(),
                    closure!(clone dispatch, |res| {
                        dispatch.emit(Action::ImportActions(res.expect("failed to read file").into()));
                    }),
                );
                self.reader = Some(task);
                false
            }
            Msg::Noop => false,
        }
    }
}
