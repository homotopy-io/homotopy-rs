use indoc::indoc;
use settings::AppSettings;
use sidebar::Sidebar;
use signature_stylesheet::SignatureStylesheet;
use wasm_bindgen::closure::Closure;
use workspace::WorkspaceView;
use yew::prelude::*;

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
}

pub struct App {
    state: model::State,
    panzoom: PanZoom,
    signature_stylesheet: SignatureStylesheet,
    toaster: Toaster,
    _settings: AppSettings,
    before_unload: Option<Closure<dyn FnMut(web_sys::BeforeUnloadEvent)>>,
}

impl Component for App {
    type Message = Message;
    type Properties = Props;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        let state = model::State::default();
        // Install the signature stylesheet
        let mut signature_stylesheet = SignatureStylesheet::new("generator");
        signature_stylesheet.update(state.with_proof(|p| p.signature().clone()));
        signature_stylesheet.mount();

        let mut app = Self {
            state,
            panzoom: PanZoom::new(),
            signature_stylesheet,
            toaster: Toaster::new(),
            _settings: AppSettings::connect(Callback::noop()),
            before_unload: None,
        };
        app.install_unload_hook();
        app
    }

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
                <div id="about" class="modal">
                    <div class="modal-dialog">
                        <div class="modal-content">
                            <header>
                                <h2>{"About"}</h2>
                                <a href="#" class="modal-close">
                                    <Icon name="close" size={IconSize::Icon18} />
                                </a>
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
                            <pre>{indoc!("
                                BSD 3-Clause License

                                Copyright (c) 2021, github.com/homotopy-io/homotopy-rs contributors
                                All rights reserved.

                                Redistribution and use in source and binary forms, with or without
                                modification, are permitted provided that the following conditions are met:

                                1. Redistributions of source code must retain the above copyright notice, this
                                  list of conditions and the following disclaimer.

                                2. Redistributions in binary form must reproduce the above copyright notice,
                                  this list of conditions and the following disclaimer in the documentation
                                  and/or other materials provided with the distribution.

                                3. Neither the name of the copyright holder nor the names of its
                                  contributors may be used to endorse or promote products derived from
                                  this software without specific prior written permission.

                                THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS \"AS IS\"
                                AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
                                IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
                                DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
                                FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
                                DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
                                SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
                                CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
                                OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
                                OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
                            ")}</pre>
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
