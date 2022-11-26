use std::cell::RefCell;

use boundary::BoundaryPreview;
use gloo_timers::{callback::Timeout, future::TimeoutFuture};
use homotopy_model::{proof::SerializedData, serialize};
use rexie::{ObjectStore, Rexie, Store, Transaction, TransactionMode};
use settings::{AppSettings, AppSettingsKey};
use sidebar::Sidebar;
use signature_stylesheet::SignatureStylesheet;
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;
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
mod keybindings;
mod project;
mod settings;
mod sidebar;
mod signature;
mod signature_stylesheet;
mod tex;
mod workspace;

pub enum Message {
    Autosave,
    BlockingDispatch(model::Action),
    Dispatch(model::Action),
    DatabaseReady,
    LoadAutosave(Option<SerializedData>),
}

thread_local! {
    pub static INDEXEDDB: RefCell<Option<Rexie>>  = RefCell::new(None);
}

pub struct App {
    state: model::State,
    autosave: Option<Timeout>,
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
        // Initialize IndexedDB for autosaving
        ctx.link().send_future(async {
            let rexie = Rexie::builder("autosave")
                .add_object_store(ObjectStore::new("saves"))
                .build()
                .await
                .expect("failed to initialize IndexedDB");
            log::info!("IndexedDB ready");
            INDEXEDDB.with(|db| db.replace(Some(rexie)));
            Message::DatabaseReady
        });

        Self {
            state,
            autosave: Default::default(),
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
            Message::Autosave => {
                let data = serialize::serialize(
                    self.state.proof().signature.clone(),
                    self.state.proof().workspace.clone(),
                    self.state.proof().metadata.clone(),
                );
                spawn_local(async move {
                    INDEXEDDB.with(|db| {
                        if let Some((transaction, saves)) =
                            get_saves(db, TransactionMode::ReadWrite)
                        {
                            let encoded = serde_wasm_bindgen::to_value(data.as_slice()).unwrap();
                            spawn_local(async move {
                                saves.put(&encoded, Some(&"latest".into())).await.unwrap();
                                transaction.done().await.unwrap();
                            });
                        } else {
                            log::warn!("Attempted autosave but IndexedDB not ready");
                        }
                    });
                });
                false
            }
            Message::BlockingDispatch(action) => {
                self.autosave.take().map(Timeout::cancel);
                self.loading = true;

                ctx.link().send_future(async move {
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

                    let link = ctx.link().clone();
                    self.autosave = Some(Timeout::new(30000, move || {
                        link.send_message(Message::Autosave);
                    }));
                } else if let Err(error) = result {
                    log::error!("Error occured: {}", error);
                    self.toaster.toast(Toast::error(error.to_string()));
                }

                true
            }
            Message::DatabaseReady => {
                // load autosave
                let link = ctx.link().clone();
                INDEXEDDB.with(|db| {
                    if let Some((_, saves)) = get_saves(db, TransactionMode::ReadOnly) {
                        link.send_future(async move {
                            if let Ok(autosave) = saves.get(&"latest".into()).await {
                                log::info!("Loading autosave…");
                                let proof = serde_wasm_bindgen::from_value(autosave).ok();
                                Message::LoadAutosave(proof)
                            } else {
                                Message::LoadAutosave(None)
                            }
                        });
                    } else {
                        log::error!("IndexedDB not ready!");
                    }
                });
                true
            }
            Message::LoadAutosave(proof) => {
                let Some(data) = proof else { return false };
                if self
                    .state
                    .update(model::Action::Proof(model::proof::Action::ImportProof(
                        data,
                    )))
                    .is_ok()
                {
                    self.signature_stylesheet
                        .update(self.state.proof().signature.clone());
                    self.toaster
                        .toast(Toast::success("Successfully loaded autosave"));
                    true
                } else {
                    log::error!("Failed to load autosave");
                    false
                }
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
                        <div class="modal-content">
                            <header>
                                <h2>{"Unexpected Crash!"}</h2>
                            </header>
                            <p>
                                {"It appears you have found an unexpected bug in our tool. Many apologies for the poor experience."}
                            </p>
                            <p>
                                {"We would be extremely grateful if you could report this issue."}
                            </p>
                            <p>
                                {"The process is rather straightforward: the button below will download a file containing some debugging information for us, you can attach it in a new issue in our "}
                                <a href="https://github.com/homotopy-io/homotopy-rs/issues">{"GitHub tracker"}</a>
                                {", alongside a brief description of what your were doing."}
                            </p>
                            <p>
                                {"We'll fix the problem in no time!"}
                            </p>
                            <button onclick={move |_| {crate::panic::export_dump(false).unwrap();}}>{"Download action logs"}</button>
                        </div>
                    </div>
                </div>
                <div id="about" class="modal">
                    <div class="modal-dialog">
                        <a href="#invisible-button">
                            // Empty div to create an invisible button
                            <div class="modal-close"></div>
                        </a>
                        <div class="modal-content">
                            <header>
                                <h2>{"About"}</h2>
                            </header>
                            <p>
                                <a href="https://ncatlab.org/nlab/show/homotopy.io">{"homotopy.io"}</a>
                                {": the proof assistant for finitely-presented globular n-categories."}
                            </p>
                            <p>{"Written by "}
                                <a href="https://github.com/doctorn">{"Nathan Corbyn"}</a>
                                {", "}
                                <a href="https://github.com/zrho">{"Lukas Heidemann"}</a>
                                {", "}
                                <a href="https://github.com/NickHu">{"Nick Hu"}</a>
                                {", "}
                                <a href="https://github.com/calintat">{"Calin Tataru"}</a>
                                {", and "}
                                <a href="https://github.com/jamievicary">{"Jamie Vicary"}</a>
                                {"."}
                            </p>
                            <h3>{"License"}</h3>
                            <p>{"homotopy.io source code is published under the terms of the BSD 3-Clause License."}</p>
                            <pre>{include_str!("../../LICENSE")}</pre>
                            {"homotopy.io documentation is licensed under a "}
                            <a rel="license" href="http://creativecommons.org/licenses/by/4.0/">
                                {"Creative Commons Attribution 4.0 International License"}
                            </a>{"."}
                            <br />
                            <a rel="license" href="http://creativecommons.org/licenses/by/4.0/">
                                <img alt="Creative Commons License" style="border-width:0" src="by.svg" />
                            </a>
                        </div>
                    </div>
                </div>
                <span class="version">
                    {format!("Version: {}", option_env!("GIT_DESCRIBE").unwrap_or(env!("CARGO_PKG_VERSION")))}
                </span>
            </main>
        }
    }
}

fn get_saves(db: &RefCell<Option<Rexie>>, mode: TransactionMode) -> Option<(Transaction, Store)> {
    let db = db.borrow();
    let rexie = db.as_ref()?;
    let transaction = rexie.transaction(&["saves"], mode).ok()?;
    let store = transaction.store("saves").ok()?;
    Some((transaction, store))
}
