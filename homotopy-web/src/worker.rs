use serde::{Deserialize, Serialize};
use yew_agent::{Agent, AgentLink, Public};

use crate::{app::STATE, model};

#[derive(Serialize, Deserialize)]
pub enum Request {
    Dispatch(model::Action),
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    Finished(Result<FinishResponse, String>),
}

#[derive(Serialize, Deserialize)]
pub struct FinishResponse {
    pub(crate) time: f64,
    pub(crate) reset_panzoom: bool,
}

pub struct Worker {
    link: AgentLink<Self>,
}

impl Agent for Worker {
    type Input = Request;
    type Message = ();
    type Output = Response;
    type Reach = Public<Self>;

    fn create(link: AgentLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, id: yew_agent::HandlerId) {
        match msg {
            Request::Dispatch(action) => {
                // hack for https://github.com/rustwasm/wasm-bindgen/issues/1752
                let performance: web_sys::Performance = wasm_bindgen::JsCast::unchecked_into(
                    js_sys::Reflect::get(
                        &js_sys::global(),
                        &wasm_bindgen::JsValue::from_str("performance"),
                    )
                    .expect("failed to get performance from global object"),
                );

                let time_start = performance.now();
                let mut state = STATE.lock().unwrap();
                let reset_panzoom = if let model::Action::Proof(ref action) = action {
                    state.with_proof(|p| p.resets_panzoom(action))
                } else {
                    false
                };
                let result = state.update(action);
                let time_end = performance.now();
                drop(state);
                self.link.respond(
                    id,
                    Response::Finished(
                        result
                            .map(|()| FinishResponse {
                                time: time_end - time_start,
                                reset_panzoom,
                            })
                            .map_err(|e| e.to_string()),
                    ),
                );
                homotopy_core::collect_garbage();
            }
        }
    }

    fn name_of_resource() -> &'static str {
        "homotopy_web-parallel.js"
    }

    fn is_module() -> bool {
        true
    }
}
