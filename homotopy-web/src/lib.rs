#![recursion_limit = "1024"]

use wasm_bindgen::prelude::*;

mod app;
mod components;
// Model has to be public for tests to work
pub mod model;

fn panic_handler(info: &std::panic::PanicInfo<'_>) {
    model::display_panic_message();
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::hook(info);
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
#[allow(clippy::unnecessary_wraps)]
pub fn main_js() -> Result<(), JsValue> {
    // Set panic hook.
    // We must use the yew provided method,
    // otherwise yew will override whatever we
    // set with its own handler at start_app!
    yew::set_custom_panic_hook(Box::new(panic_handler));

    // check if we are the main/UI thread
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<app::App>();
    Ok(())
}
