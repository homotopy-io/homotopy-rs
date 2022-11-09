use std::collections::BTreeSet;

use crate::{collapse::OneMany, common::BoundaryPath, Generator, Height};

pub(crate) type Coord = Vec<Height>;
pub(crate) type Coords = OneMany<Coord, BTreeSet<Coord>>;

pub(crate) type Label = Option<(Generator, BoundaryPath, BTreeSet<Coord>)>;
