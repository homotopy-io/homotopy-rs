use homotopy_core::{
    common::{Direction, SingularHeight, SliceIndex},
    contraction::Bias,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Contract {
    pub bias: Option<Bias>,
    pub location: Vec<SliceIndex>,
    pub height: SingularHeight,
    pub direction: Direction,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Expand {
    pub location: Vec<SliceIndex>,
    pub direction: Direction,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Homotopy {
    Contract(Contract),
    Expand(Expand),
}
