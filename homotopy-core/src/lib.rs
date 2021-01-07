pub mod attach;
pub mod common;
pub mod complex;
pub mod contraction;
pub mod diagram;
pub mod expansion;
pub mod factorization;
pub mod normalization;
pub mod projection;
pub mod rewrite;
pub mod typecheck;

pub use common::{Boundary, Direction, Generator, Height, SliceIndex};
pub use contraction::Bias;
pub use diagram::{Diagram, DiagramN};
pub use rewrite::{Cospan, Rewrite, Rewrite0, RewriteN};
