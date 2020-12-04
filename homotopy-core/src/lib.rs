pub mod attach;
pub mod common;
pub mod contraction;
pub mod diagram;
pub mod projection;
pub mod complex;
pub mod normalization;
pub mod rewrite;
pub mod typecheck;
pub mod expansion;

pub use common::{Boundary, Generator, Height, SliceIndex};
pub use diagram::{Diagram, DiagramN};
pub use rewrite::{Cospan, Rewrite, Rewrite0, RewriteN};
