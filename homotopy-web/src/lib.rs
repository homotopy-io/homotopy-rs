#![recursion_limit = "1024"]

use tracing::metadata::Metadata;
use tracing_subscriber::{
    filter::{FilterFn, LevelFilter},
    fmt::{format::Pretty, time::UtcTime},
    prelude::__tracing_subscriber_SubscriberExt,
    util::SubscriberInitExt,
};
use tracing_web::{performance_layer, MakeConsoleWriter};
use wasm_bindgen::prelude::*;

mod app;
mod components;
mod panic;
// Model has to be public for tests to work
pub mod model;

fn tracing_filter(meta: &Metadata<'_>) -> bool {
    meta.target().contains("homotopy")
}

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
    // setup tracing
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_writer(MakeConsoleWriter);
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(LevelFilter::DEBUG)
        .with(FilterFn::new(tracing_filter))
        .with(fmt_layer)
        .with(perf_layer)
        .init();

    yew::Renderer::<app::App>::new().render();
    Ok(())
}
