#![warn(
    clippy::pedantic,
    // clippy::cargo, // turn this on before release
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::exit,
    clippy::expect_used,
    clippy::get_unwrap,
    clippy::let_underscore_must_use,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::multiple_inherent_impl,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::rc_buffer,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::semicolon_if_nothing_returned,
    clippy::str_to_string,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::unneeded_field_pattern,
    clippy::unreachable,
    clippy::use_debug,
    clippy::use_self,
)]
#![allow( // pedantic is too annoying
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::default_trait_access,
    clippy::doc_markdown,
    clippy::expect_used,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::needless_continue,
    clippy::range_plus_one,
    clippy::redundant_else,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::shadow_unrelated,
    clippy::match_same_arms,
)]
#![cfg_attr(feature = "parallel", feature(once_cell))]
#![recursion_limit = "1024"]

use wasm_bindgen::prelude::*;
#[cfg(feature = "wasm-bindgen-rayon")]
pub use wasm_bindgen_rayon::init_thread_pool;
#[cfg(not(feature = "wasm-bindgen-rayon"))]
#[wasm_bindgen(js_name = initThreadPool)]
pub fn init_thread_pool(_nthreads: usize) {
    // empty stub
}
#[cfg(feature = "parallel")]
use yew_agent::Threaded;

mod app;
mod components;
#[cfg(feature = "parallel")]
mod worker;

pub mod model;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
#[allow(clippy::unnecessary_wraps)]
pub fn main_js() -> Result<(), JsValue> {
    use js_sys::{global, Reflect};
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    // check if we are the main/UI thread
    if Reflect::has(&global(), &JsValue::from_str("window")).unwrap() {
        homotopy_core::util::rayon::MAIN_THREAD.with(|b| b.set(true));
        wasm_logger::init(wasm_logger::Config::default());
        yew::start_app::<app::App>();
    } else {
        #[cfg(feature = "parallel")]
        worker::Worker::register();
    }
    Ok(())
}
