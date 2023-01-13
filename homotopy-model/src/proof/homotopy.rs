use homotopy_core::{
    common::{Direction, Height, SingularHeight, SliceIndex},
    contraction::Bias,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Contract {
    pub bias: Option<Bias>,
    pub location: Vec<SliceIndex>,
    pub height: SingularHeight,
    pub direction: Direction,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Expand {
    pub location: Vec<SliceIndex>,
    pub point: [Height; 2],
    pub direction: Direction,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Homotopy {
    Contract(Contract),
    Expand(Expand),
}
