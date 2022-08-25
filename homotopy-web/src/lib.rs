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
)]
#![allow( // pedantic is too annoying
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::default_trait_access,
    clippy::doc_markdown,
    clippy::expect_used,
    clippy::match_same_arms,
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
    clippy::unused_unit,
    clippy::shadow_unrelated,
    clippy::use_self, // https://github.com/rust-lang/rust-clippy/issues/6902
    clippy::too_many_arguments,
)]
#![recursion_limit = "1024"]

use wasm_bindgen::prelude::*;

mod app;
mod components;
// Model has to be public for tests to work
pub mod model;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
#[allow(clippy::unnecessary_wraps)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    // check if we are the main/UI thread
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<app::App>();
    Ok(())
}
