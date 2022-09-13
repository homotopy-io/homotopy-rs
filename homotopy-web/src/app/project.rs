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
    EditMetadata(MetadataEdit, bool), // (edit, should_dispatch)
    Noop,
}

#[derive(Debug, Properties, Clone, PartialEq)]
pub struct Props {
    pub dispatch: Callback<model::Action>,
    #[prop_or_default]
    pub metadata: Metadata,
}

#[derive(Debug, Default)]
pub struct ProjectView {
    title: String,
    author: String,
    abstr: String,
    reader: Option<gloo::file::callbacks::FileReader>,
}

impl Component for ProjectView {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
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
                <input type="file" accept="application/msgpack,.hom,.json" class="visually-hidden" id="import" onchange={import}/>
                <div class="metadata__details">
                    <textarea
                        class="metadata__title"
                        name="title"
                        placeholder="Title"
                        value= {ctx.props().metadata.title.clone().unwrap_or_default()}
                        oninput={ctx.link().callback(move |e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::EditMetadata(MetadataEdit::Title(input.value()), false)
                        })}
                        onfocusout={ctx.link().callback(move |e: FocusEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::EditMetadata(MetadataEdit::Title(input.value()), true)
                        })}
                        onkeyup={ctx.link().callback(move |e: KeyboardEvent| {
                            e.stop_propagation();
                            let input: HtmlInputElement = e.target_unchecked_into();
                            if e.key().to_ascii_lowercase() == "enter" {
                                input.blur().unwrap();
                                return Msg::EditMetadata(MetadataEdit::Title(input.value()), true);
                            }
                            Msg::Noop
                        })}
                    />

                    <textarea
                        class="metadata__author"
                        name="Author"
                        placeholder="Author(s)"
                        spellcheck="false"
                        value = {ctx.props().metadata.author.clone().unwrap_or_default()}
                        oninput={ctx.link().callback(move |e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::EditMetadata(MetadataEdit::Author(input.value()), false)
                        })}
                        onfocusout={ctx.link().callback(move |e: FocusEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::EditMetadata(MetadataEdit::Author(input.value()), true)
                        })}
                        onkeyup={ctx.link().callback(move |e: KeyboardEvent| {
                            e.stop_propagation();
                            let input: HtmlInputElement = e.target_unchecked_into();
                            if e.key().to_ascii_lowercase() == "enter" {
                                input.blur().unwrap();
                                return Msg::EditMetadata(MetadataEdit::Author(input.value()), true);
                            }
                            Msg::Noop
                        })}
                    />

                    <textarea
                        class="metadata__abstract"
                        name="Abstract"
                        placeholder="Abstract"
                        value = {ctx.props().metadata.abstr.clone().unwrap_or_default()}
                        oninput={ctx.link().callback(move |e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::EditMetadata(MetadataEdit::Abstract(input.value()), false)
                        })}
                        onfocusout={ctx.link().callback(move |e: FocusEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::EditMetadata(MetadataEdit::Abstract(input.value()), true)
                        })}
                        onkeyup={ctx.link().callback(move |e: KeyboardEvent| {
                            e.stop_propagation();
                            Msg::Noop
                        })}
                    />
                </div>
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
            Msg::EditMetadata(edit, should_dispatch) => {
                // In order to avoid generating multiple history events for a single rename, we
                // don't dispatch renames until the user is done editing.
                let changed = matches!(&edit, MetadataEdit::Title(ref title) if title != &self.title)
                    || matches!(&edit, MetadataEdit::Author(ref author) if author != &self.author)
                    || matches!(&edit, MetadataEdit::Abstract(ref abstr) if abstr != &self.abstr);

                if should_dispatch && changed {
                    dispatch.emit(model::Action::Proof(proof::Action::EditMetadata(
                        edit.clone(),
                    )));
                    match edit {
                        MetadataEdit::Title(title) => self.title = title,
                        MetadataEdit::Author(author) => self.author = author,
                        MetadataEdit::Abstract(abstr) => self.abstr = abstr,
                    }
                }
                should_dispatch
            }
            Msg::Noop => false,
        }
    }
}
