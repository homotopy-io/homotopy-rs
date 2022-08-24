use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::model::{
    proof::{Metadata, MetadataEdit},
    Action,
};

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub dispatch: Callback<Action>,
    #[prop_or_default]
    pub metadata: Metadata,
}
pub struct ProjectView {}

impl Component for ProjectView {
    type Message = crate::model::proof::Action;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let export = ctx.props().dispatch.reform(|_| Action::ExportProof);
        // TODO (@anastaia-jc) broken:
        // let dispatch = &ctx.props().dispatch;
        // let reader_task = use_mut_ref(|| None);
        // let import = Callback::from(closure!(clone dispatch, |evt: Event| {
        //     let input: HtmlInputElement = evt.target_unchecked_into();
        //     if let Some(filelist) = input.files() {
        //         let file = filelist.get(0).unwrap();
        //         let task = gloo::file::callbacks::read_as_bytes(&file.into(), closure!(clone dispatch, |res| {
        //             dispatch.emit(Action::ImportProof(res.expect("failed to read file").into()));
        //         }));
        //         *reader_task.borrow_mut() = Some(task);
        //     }
        // }));
        html! {
            <>
                <button onclick={export}>{"Export"}</button>
                <label for="import" class="button">
                    {"Import"}
                </label>
                // <input type="file" accept="application/msgpack,.hom" class="visually-hidden" id="import" onchange={import}/>
                <input
                    type="text"
                    class="metadata_title"
                    name="title"
                    value= {
                        let title = ctx.props().metadata.title.clone();
                        match title {
                            Some(value) => value,
                            None => "Title".to_owned(),
                    }}
                    oninput={ctx.link().callback(move |e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        crate::model::proof::Action::EditMetadata(MetadataEdit::Title(input.value()))
                    })}
                    onkeyup={ctx.link().callback(move |e: KeyboardEvent| {
                        e.stop_propagation();
                        crate::model::proof::Action::Nothing
                    })}
                />

                <input
                    type="text"
                    class="metadata_author"
                    name="Author"
                    value = {
                        let author = ctx.props().metadata.author.clone();
                        match author {
                            Some(value) => value,
                            None => "Author".to_owned(),
                    }}
                    oninput={ctx.link().callback(move |e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        crate::model::proof::Action::EditMetadata(MetadataEdit::Author(input.value()))
                    })}
                    onkeyup={ctx.link().callback(move |e: KeyboardEvent| {
                        e.stop_propagation();
                        crate::model::proof::Action::Nothing
                    })}
                />

                <input
                    type="textarea"
                    class="metadata_abstract"
                    name="Abstract"
                    value = {
                        let abstract_ = ctx.props().metadata.abstract_.clone();
                        match abstract_ {
                            Some(value) => value,
                            None => "Abstract".to_owned(),
                    }}
                    oninput={ctx.link().callback(move |e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        crate::model::proof::Action::EditMetadata(MetadataEdit::Abstract(input.value()))
                    })}
                    onkeyup={ctx.link().callback(move |e: KeyboardEvent| {
                        e.stop_propagation();
                        crate::model::proof::Action::Nothing
                    })}
                />
            </>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        if let crate::model::proof::Action::EditMetadata(edit) = msg {
            ctx.props()
                .dispatch
                .emit(Action::Proof(crate::model::proof::Action::EditMetadata(
                    edit,
                )));
        }
        true
    }
}
