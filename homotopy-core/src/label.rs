use std::collections::BTreeSet;

use crate::{common::BoundaryPath, Generator, Height};

pub(crate) type Coord = Vec<Height>;

pub(crate) type Label = Option<(Generator, BoundaryPath, BTreeSet<Coord>)>;
