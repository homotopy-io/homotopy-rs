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
    clippy::expect_used,
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::needless_continue,
    clippy::range_plus_one,
    clippy::redundant_else,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::multiple_inherent_impl,
    clippy::shadow_unrelated,
    clippy::match_on_vec_items,
    clippy::type_complexity,
    clippy::unnecessary_wraps
)]
#![feature(once_cell)]

pub use common::{Boundary, Direction, Generator, Height, SliceIndex};
pub use contraction::Bias;
pub use diagram::{Diagram, DiagramN};
pub use rewrite::{Cospan, Rewrite, Rewrite0, RewriteN};

pub mod attach;
pub mod common;
pub mod complex;
pub mod contraction;
pub mod diagram;
pub mod examples;
pub mod expansion;
pub mod factorization;
pub mod graph;
pub mod layout;
pub mod mesh;
pub mod monotone;
pub mod normalization;
pub mod projection;
pub mod rewrite;
pub mod serialize;
pub mod signature;
pub mod typecheck;
pub mod util;

pub fn collect_garbage() {
    DiagramN::collect_garbage();
    RewriteN::collect_garbage();
    rewrite::Cone::collect_garbage();
}
