#[cfg(feature = "parallel")]
use std::{
    lazy::SyncLazy,
    sync::{
        Mutex,
        TryLockError::{Poisoned, WouldBlock},
    },
};

use settings::AppSettings;
use sidebar::Sidebar;
use signature_stylesheet::SignatureStylesheet;
use wasm_bindgen::closure::Closure;
use workspace::WorkspaceView;
use yew::prelude::*;
#[cfg(feature = "parallel")]
use yew_agent::{Bridge, Bridged};

#[cfg(feature = "parallel")]
use crate::worker::{Request, Response, Worker};
use crate::{
    components::{
        icon::{Icon, IconSize},
        panzoom::PanZoom,
        settings::Settings,
        toast::{Toast, Toaster, ToasterComponent},
    },
    model,
};

mod attach;
mod diagram_gl;
mod diagram_svg;
mod project;
mod settings;
mod sidebar;
mod signature;
mod signature_stylesheet;
mod workspace;

#[derive(Default, Clone, Debug, PartialEq, Properties)]
pub struct Props {}

pub enum Message {
    Dispatch(model::Action),
    #[cfg(feature = "parallel")]
    WorkerMessage(Response),
}

#[cfg(feature = "parallel")]
pub(crate) static STATE: SyncLazy<Mutex<model::State>> =
    SyncLazy::new(|| Mutex::new(model::State::default()));

pub struct App {
    #[cfg(not(feature = "parallel"))]
    state: model::State,
    panzoom: PanZoom,
    signature_stylesheet: SignatureStylesheet,
    toaster: Toaster,
    _settings: AppSettings,
    before_unload: Option<Closure<dyn FnMut(web_sys::BeforeUnloadEvent)>>,
    #[cfg(feature = "parallel")]
    worker: Box<dyn Bridge<Worker>>,
}

impl Component for App {
    type Message = Message;
    type Properties = Props;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(feature = "parallel")]
        // Spawn background worker
        let worker = Worker::bridge(ctx.link().callback(Self::Message::WorkerMessage));
        #[cfg(not(feature = "parallel"))]
        let state = model::State::default();
        // Install the signature stylesheet
        let mut signature_stylesheet = SignatureStylesheet::new("generator");
        signature_stylesheet.update(
            #[cfg(feature = "parallel")]
            STATE
                .try_lock()
                .unwrap()
                .with_proof(|p| p.signature().clone()),
            #[cfg(not(feature = "parallel"))]
            state.with_proof(|p| p.signature().clone()),
        );
        signature_stylesheet.mount();

        let mut app = Self {
            #[cfg(not(feature = "parallel"))]
            state,
            panzoom: PanZoom::new(),
            signature_stylesheet,
            toaster: Toaster::new(),
            _settings: AppSettings::connect(Callback::noop()),
            before_unload: None,
            #[cfg(feature = "parallel")]
            worker,
        };
        app.install_unload_hook();
        app
    }

    #[cfg(feature = "parallel")]
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Dispatch(action) => {
                log::info!("Received action: {:?}", action);
                self.worker.send(Request::Dispatch(action));
                false
            }
            Message::WorkerMessage(Response::Finished(Err(error))) => {
                self.toaster.toast(Toast::error(format!("{}", error)));
                log::error!("Error occured: {}", error);
                false
            }
            Message::WorkerMessage(Response::Finished(Ok(params))) => {
                log::info!("State update took {}ms.", params.time);
                if params.reset_panzoom {
                    self.panzoom.reset()
                }
                let _ = STATE.try_lock().map(|state| {
                    self.signature_stylesheet
                        .update(state.with_proof(|p| p.signature().clone()))
                });
                true
            }
        }
    }

    #[cfg(not(feature = "parallel"))]
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Dispatch(action) => {
                log::info!("Received action: {:?}", action);
                if let model::Action::Proof(ref action) = action {
                    if self.state.with_proof(|p| p.resets_panzoom(action)) {
                        self.panzoom.reset();
                    }
                }

                let time_start = web_sys::window().unwrap().performance().unwrap().now();
                let result = self.state.update(action);
                let time_stop = web_sys::window().unwrap().performance().unwrap().now();
                log::info!("State update took {}ms.", time_stop - time_start);

                homotopy_core::collect_garbage();

                if let Err(error) = result {
                    self.toaster.toast(Toast::error(format!("{}", error)));
                    log::error!("Error occured: {}", error);
                }

                self.signature_stylesheet
                    .update(self.state.with_proof(|p| p.signature().clone()));

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        #[cfg(feature = "parallel")]
        match STATE.try_lock() {
            Ok(state) => Self::render(ctx, &state),
            Err(WouldBlock) => {
                //    do nothing; try again after worker processes next action;
                //    guaranteed to be able to acquire the lock without blocking
                //    after the last worker action
                html! {<div class="loader">{"Loading…"}</div>}
            }
            Err(Poisoned(_)) => {
                todo!("handle panic in background thread")
            }
        }
        #[cfg(not(feature = "parallel"))]
        Self::render(ctx, &self.state)
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.signature_stylesheet.unmount();
    }
}

impl App {
    fn install_unload_hook(&mut self) {
        use wasm_bindgen::JsCast;

        let before_unload = Closure::wrap(Box::new(move |event: web_sys::BeforeUnloadEvent| {
            event.set_return_value("Are you sure you want to leave? Unsaved changes will be lost!");
        }) as Box<dyn FnMut(_)>);

        web_sys::window()
            .unwrap()
            .set_onbeforeunload(Some(before_unload.as_ref().unchecked_ref()));

        self.before_unload = Some(before_unload);
    }

    fn render(ctx: &Context<Self>, state: &model::State) -> Html {
        let proof = state.with_proof(Clone::clone);
        let dispatch = ctx.link().callback(Message::Dispatch);
        let signature = proof.signature();

        let workspace = match proof.workspace() {
            Some(workspace) => {
                html! {
                    <WorkspaceView
                        workspace={workspace.clone()}
                        signature={signature.clone()}
                        dispatch={dispatch.reform(model::Action::Proof)}
                    />
                }
            }
            None => {
                // TODO: Show onboarding info if workspace and signature is empty
                html! {
                    <content class="workspace workspace--empty">
                    </content>
                }
            }
        };

        html! {
            <main class="app">
                <Sidebar
                    dispatch={dispatch}
                    proof={proof}
                />
                <ToasterComponent timeout={3000} />
                {workspace}
                <span class="version">
                    {format!("Version: {}", option_env!("GIT_DESCRIBE").unwrap_or(env!("CARGO_PKG_VERSION")))}
                </span>
            </main>
        }
    }
}
