use boundary::BoundaryPreview;
use gloo_timers::future::TimeoutFuture;
use settings::{AppSettings, AppSettingsKey};
use sidebar::Sidebar;
use signature_stylesheet::SignatureStylesheet;
use wasm_bindgen::{closure::Closure, JsCast};
use workspace::WorkspaceView;
use yew::prelude::*;

use self::diagram_gl::GlViewControl;
use crate::{
    components::{
        icon::{Icon, IconSize},
        panzoom::PanZoom,
        settings::Settings,
        toast::{Toast, Toaster, ToasterComponent},
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
    Dispatch(model::Action),
}

pub struct App {
    state: model::State,
    loading: bool,
    panzoom: PanZoom,
    orbit_control: GlViewControl,
    signature_stylesheet: SignatureStylesheet,
    toaster: Toaster,
    _settings: AppSettings,
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
            panzoom: PanZoom::new(),
            orbit_control: GlViewControl::new(),
            signature_stylesheet,
            toaster: Toaster::new(),
            _settings: AppSettings::connect(Callback::noop()),
            before_unload: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::BlockingDispatch(action) => {
                self.loading = true;

                ctx.link().send_future(async move {
                    //TODO: remove this too
                    std::panic::set_hook(Box::new(crate::panic::panic_handler));

                    TimeoutFuture::new(0).await; // TODO: remove this awful hack
                    Message::Dispatch(action)
                });
                true
            }
            Message::Dispatch(action) => {
                log::info!("Received action: {:?}", action);

                // Intercept 'MakeOriented' actions to show warning.
                if let model::Action::Proof(model::proof::Action::EditSignature(
                    model::proof::SignatureEdit::Edit(
                        _,
                        model::proof::SignatureItemEdit::MakeOriented(true),
                    ),
                )) = &action
                {
                    self.toaster
                        .toast(Toast::warn("Oriented generators are experimental"));
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
                log::info!(
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
                        self.panzoom.reset();
                        self.orbit_control.reset();
                    }

                    if self.before_unload.is_none() {
                        self.install_unload_hook();
                    }

                    self.signature_stylesheet
                        .update(self.state.proof().signature.clone());
                } else if let Err(error) = result {
                    log::error!("Error occured: {}", error);
                    self.toaster.toast(Toast::error(error.to_string()));
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

    fn render(ctx: &Context<Self>, state: &model::State, loading: bool) -> Html {
        let proof = state.proof();
        let dispatch = ctx.link().callback(Message::BlockingDispatch);

        let workspace = html! {
            <WorkspaceView
                workspace={proof.workspace.clone()}
                signature={proof.signature.clone()}
                metadata={proof.metadata.clone()}
                dispatch={dispatch.clone()}
                attach={state.attach.clone()}
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
                    attach={state.attach.clone()}
                />
                <div class="toaster">
                    <ToasterComponent timeout={3000} />
                </div>
                <div class="boundary__and__workspace">
                    {boundary_preview}
                    {workspace}
                </div>
                <div id="panic" class="modal" style="display:none;position:absolute;">
                    <div class="modal-dialog">
                        <a href="#invisible-button">
                            // Empty div to create an invisible button
                            <div class="modal-close"></div>
                        </a>
                        {info::get_panic_message()}
                    </div>
                </div>
                <div id="about" class="modal">
                    <div class="modal-dialog">
                        <a href="#">
                            // Empty div to create an invisible button
                            <div class="modal-close"></div>
                        </a>
                        {info::get_about_message()}
                    </div>
                </div>
                <div id="help" class="modal">
                    <div class="modal-dialog">
                        <a href="#">
                            // Empty div to create an invisible button
                            <div class="modal-close"></div>
                        </a>
                        {info::get_help_message()}
                    </div>
                </div>
                <span class="version">
                    {format!("Version: {}", option_env!("GIT_DESCRIBE").unwrap_or(env!("CARGO_PKG_VERSION")))}
                </span>
            </main>
        }
    }
}
