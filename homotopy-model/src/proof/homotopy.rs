use homotopy_core::{
    common::{Direction, Height, SingularHeight, SliceIndex},
    contraction::Bias,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Contract {
    pub height: SingularHeight,
    pub direction: Direction,
    pub step: usize,
    pub bias: Option<Bias>,
    pub location: Vec<SliceIndex>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Expand {
    pub point: [Height; 2],
    pub direction: Direction,
    pub location: Vec<SliceIndex>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Homotopy {
    Contract(Contract),
    Expand(Expand),
}
