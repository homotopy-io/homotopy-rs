use boundary::BoundaryPreview;
use settings::{AppSettings, AppSettingsKey, AppSettingsMsg};
use sidebar::Sidebar;
use signature_stylesheet::SignatureStylesheet;
use wasm_bindgen::{closure::Closure, JsCast};
use workspace::WorkspaceView;
use yew::prelude::*;

use self::diagram_gl::GlViewControl;
use crate::{
    components::{
        icon::{Icon, IconSize},
        modal::Modal,
        panzoom::PanZoom,
        toast::{toast, Toast, ToasterComponent},
    },
    model,
};

mod account;
mod attach;
mod boundary;
#[cfg(any(debug_assertions, feature = "show_debug_panel"))]
mod debug;
mod diagram_gl;
mod diagram_svg;
mod image_export;
mod info;
mod keybindings;
mod project;
mod settings;
mod sidebar;
mod signature;
mod signature_stylesheet;
mod tex;
mod workspace;

pub enum Message {
    BlockingDispatch(model::Action),
    #[allow(dead_code)]
    Dispatch(model::Action),
}

pub struct App {
    state: model::State,
    loading: bool,
    signature_stylesheet: SignatureStylesheet,
    before_unload: Option<Closure<dyn FnMut(web_sys::BeforeUnloadEvent)>>,
}

impl Component for App {
    type Message = Message;
    type Properties = ();

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        let state = model::State::default();
        // Install the signature stylesheet
        let signature_stylesheet = SignatureStylesheet::new();
        signature_stylesheet.mount();

        Self {
            state,
            loading: false,
            signature_stylesheet,
            before_unload: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::BlockingDispatch(action) | Message::Dispatch(action) => {
                tracing::info!("Received action: {:?}", action);

                // Intercept 'MakeOriented' actions to show warning.
                if let model::Action::Proof(model::proof::Action::EditSignature(
                    model::proof::SignatureEdit::Edit(
                        _,
                        model::proof::SignatureItemEdit::MakeOriented(true),
                    ),
                )) = &action
                {
                    toast(Toast::warn("Oriented generators are experimental"));
                }

                // Determine if the action needs to reset the panzoom
                // but do not reset it until we have performed the action.
                let resets_panzoom = if let model::Action::Proof(action) = &action {
                    self.state.proof().resets_panzoom(action)
                } else {
                    false
                };

                let performance = web_sys::window().unwrap().performance().unwrap();
                performance.mark("startStateUpdate").unwrap();
                let result = self.state.update(action);
                performance.mark("stopStateUpdate").unwrap();
                performance
                    .measure_with_start_mark_and_end_mark(
                        "stateUpdate",
                        "startStateUpdate",
                        "stopStateUpdate",
                    )
                    .unwrap();
                tracing::info!(
                    "State update took {}ms.",
                    js_sys::Reflect::get(
                        &performance
                            .get_entries_by_name_with_entry_type("stateUpdate", "measure")
                            .get(0),
                        &wasm_bindgen::JsValue::from_str("duration")
                    )
                    .unwrap()
                    .as_f64()
                    .unwrap()
                );

                performance.clear_marks();
                performance.clear_measures();

                homotopy_core::collect_garbage();

                self.loading = false;

                if let Ok(true) = result {
                    if resets_panzoom {
                        PanZoom::reset();
                        GlViewControl::reset();
                    }

                    if self.before_unload.is_none() {
                        self.install_unload_hook();
                    }

                    self.signature_stylesheet
                        .update(self.state.proof().signature.clone());
                } else if let Err(error) = result {
                    tracing::error!("Error occured: {}", error);
                    toast(Toast::error(error.to_string()));
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        Self::render(ctx, &self.state, self.loading)
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.signature_stylesheet.unmount();
    }
}

impl App {
    fn install_unload_hook(&mut self) {
        let before_unload = Closure::wrap(Box::new(move |event: web_sys::BeforeUnloadEvent| {
            event.set_return_value("Are you sure you want to leave? Unsaved changes will be lost!");
        }) as Box<dyn FnMut(_)>);

        web_sys::window()
            .unwrap()
            .set_onbeforeunload(Some(before_unload.as_ref().unchecked_ref()));

        self.before_unload = Some(before_unload);
    }

    #[allow(clippy::let_underscore_untyped)]
    fn render(ctx: &Context<Self>, state: &model::State, loading: bool) -> Html {
        let proof = state.proof();
        let dispatch = ctx.link().callback(Message::BlockingDispatch);

        let workspace = html! {
            <WorkspaceView
                workspace={proof.workspace.clone()}
                signature={proof.signature.clone()}
                metadata={proof.metadata.clone()}
                dispatch={dispatch.clone()}
                attachment_highlight={state.attachment_highlight.clone()}
                slice_highlight={state.slice_highlight}
            />
        };

        let boundary_preview = match &proof.boundary {
            Some(b) => html! {
                <BoundaryPreview
                    boundary={b.clone()}
                    dispatch={dispatch.reform(model::Action::Proof)}
                    signature={proof.signature.clone()}
                />
            },
            None => Default::default(),
        };

        let spinner = if loading {
            html! { <div class="cover-spin"></div> }
        } else {
            html! {}
        };

        html! {
            <main class="app">
                {spinner}
                <Sidebar
                    dispatch={dispatch}
                    proof={proof.clone()}
                    options={state.options.clone()}
                />
                <div class="toaster">
                    <ToasterComponent timeout={3000} />
                </div>
                <div class="boundary__and__workspace">
                    {boundary_preview}
                    {workspace}
                </div>
                <Modal id="panic" header="Unexpected crash" persistent=true>
                    {info::get_panic_message()}
                </Modal>
                <Modal id="about" header="About">
                    {info::get_about_message()}
                </Modal>
                <Modal id="help" header="Help">
                    {info::get_help_message()}
                </Modal>
                <span class="version">
                    {format!("Version: {}", option_env!("GIT_DESCRIBE").unwrap_or(env!("CARGO_PKG_VERSION")))}
                </span>
            </main>
        }
    }
}
