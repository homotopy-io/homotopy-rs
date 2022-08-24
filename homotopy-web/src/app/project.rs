use closure::closure;
use web_sys::{File, HtmlInputElement};
use yew::prelude::*;

use crate::model::{
    self,
    proof::{self, Metadata, MetadataEdit},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    ImportProof(File),
    EditMetadata(MetadataEdit),
    Noop,
}

#[derive(Debug, Properties, Clone, PartialEq)]
pub struct Props {
    pub dispatch: Callback<model::Action>,
    #[prop_or_default]
    pub metadata: Metadata,
}

pub struct ProjectView {
    reader: Option<gloo::file::callbacks::FileReader>,
}

impl Component for ProjectView {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { reader: None }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let export = ctx.props().dispatch.reform(|_| model::Action::ExportProof);
        let import = ctx.link().callback(|e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(filelist) = input.files() {
                Msg::ImportProof(filelist.get(0).unwrap())
            } else {
                Msg::Noop
            }
        });
        html! {
            <>
                <button onclick={export}>{"Export"}</button>
                <label for="import" class="button">
                    {"Import"}
                </label>
                <input type="file" accept="application/msgpack,.hom" class="visually-hidden" id="import" onchange={import}/>
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
                        Msg::EditMetadata(MetadataEdit::Title(input.value()))
                    })}
                    onkeyup={ctx.link().callback(move |e: KeyboardEvent| {
                        e.stop_propagation();
                        Msg::Noop
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
                        Msg::EditMetadata(MetadataEdit::Author(input.value()))
                    })}
                    onkeyup={ctx.link().callback(move |e: KeyboardEvent| {
                        e.stop_propagation();
                        Msg::Noop
                    })}
                />

                <input
                    type="textarea"
                    class="metadata_abstract"
                    name="Abstract"
                    value = {
                        let abstr = ctx.props().metadata.abstr.clone();
                        match abstr {
                            Some(value) => value,
                            None => "Abstract".to_owned(),
                    }}
                    oninput={ctx.link().callback(move |e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Msg::EditMetadata(MetadataEdit::Abstract(input.value()))
                    })}
                    onkeyup={ctx.link().callback(move |e: KeyboardEvent| {
                        e.stop_propagation();
                        Msg::Noop
                    })}
                />
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let dispatch = &ctx.props().dispatch;
        match msg {
            Msg::ImportProof(file) => {
                let task = gloo::file::callbacks::read_as_bytes(
                    &file.into(),
                    closure!(clone dispatch, |res| {
                        dispatch.emit(model::Action::ImportProof(res.expect("failed to read file").into()));
                    }),
                );
                self.reader = Some(task);
                false
            }
            Msg::EditMetadata(edit) => {
                dispatch.emit(model::Action::Proof(proof::Action::EditMetadata(edit)));
                true
            }
            Msg::Noop => false,
        }
    }
}
