#![recursion_limit = "1024"]

use wasm_bindgen::prelude::*;

mod app;
mod components;
mod panic;
// Model has to be public for tests to work
pub mod model;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
#[allow(clippy::unnecessary_wraps)]
pub fn main_js() -> Result<(), JsValue> {
    // Set panic hook.
    // We must use the yew provided method,
    // otherwise yew will override whatever we
    // set with its own handler at start_app!
    yew::set_custom_panic_hook(Box::new(panic::panic_handler));

    // check if we are the main/UI thread
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<app::App>::new().render();
    Ok(())
}
