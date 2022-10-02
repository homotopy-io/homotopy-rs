use closure::closure;
use wasm_bindgen::prelude::*;
use web_sys::{File, HtmlInputElement};
use yew::prelude::*;

use crate::{
    app::tex::TexSpan,
    model::{
        self,
        proof::{self, Metadata, MetadataEdit},
    },
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

#[derive(Debug, Default)]
pub struct ProjectView {
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
        let test = |_e: MouseEvent| {
            crate::model::create_download_closure("test", "json", &[0])
                .call0(&JsValue::UNDEFINED)
                .unwrap();
        };

        html! {
            <>
                <button onclick={export}>{"Export"}</button>
                <button onclick={test}>{"Test"}</button>
                <label for="import" class="button">
                    {"Import"}
                </label>
                <input type="file" accept="application/msgpack,.hom,.json" class="visually-hidden" id="import" onchange={import}/>
                <div class="metadata__details">
                    <TexSpan
                        class="metadata__title"
                        placeholder="Title"
                        is_editable_as_textarea={true}
                        raw_tex={ctx.props().metadata.title.clone().unwrap_or_default()}
                        on_focus_out={ctx.link().callback(move |title: String| {
                            Msg::EditMetadata(MetadataEdit::Title(title))
                        })}
                        on_key_down={ctx.link().callback(move |e: KeyboardEvent| {
                            e.stop_propagation();
                            Msg::Noop
                        })}
                    />
                    <TexSpan
                        class="metadata__author"
                        placeholder="Author(s)"
                        is_editable_as_textarea={true}
                        raw_tex={ctx.props().metadata.author.clone().unwrap_or_default()}
                        on_focus_out={ctx.link().callback(move |author: String| {
                            Msg::EditMetadata(MetadataEdit::Author(author))
                        })}
                        on_key_down={ctx.link().callback(move |e: KeyboardEvent| {
                            e.stop_propagation();
                            Msg::Noop
                        })}
                    />
                    <TexSpan
                        class="metadata__abstract"
                        placeholder="Abstract"
                        is_editable_as_textarea={true}
                        raw_tex={ctx.props().metadata.abstr.clone().unwrap_or_default()}
                        on_focus_out={ctx.link().callback(move |abstr: String| {
                            Msg::EditMetadata(MetadataEdit::Abstract(abstr))
                        })}
                        on_key_down={ctx.link().callback(move |e: KeyboardEvent| {
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
                        dispatch.emit(model::Action::Proof(model::proof::Action::ImportProof(res.expect("failed to read file").into())));
                    }),
                );
                self.reader = Some(task);
                false
            }
            Msg::EditMetadata(edit) => {
                // In order to avoid generating multiple history events for a single rename, we
                // don't dispatch renames until the user is done editing.
                let changed = match &edit {
                    MetadataEdit::Title(title) => {
                        *title != ctx.props().metadata.title.clone().unwrap_or_default()
                    }
                    MetadataEdit::Author(author) => {
                        *author != ctx.props().metadata.author.clone().unwrap_or_default()
                    }
                    MetadataEdit::Abstract(abstr) => {
                        *abstr != ctx.props().metadata.abstr.clone().unwrap_or_default()
                    }
                };
                if changed {
                    dispatch.emit(model::Action::Proof(proof::Action::EditMetadata(edit)));
                }
                false
            }
            Msg::Noop => false,
        }
    }
}
