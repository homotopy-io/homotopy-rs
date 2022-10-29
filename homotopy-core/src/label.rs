use homotopy_common::hash::FastHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    collapse::collapse,
    common::BoundaryPath,
    scaffold::{Explodable, StableScaffold},
    signature::Signature,
    Diagram, Height, SliceIndex,
};

type Coord = Vec<Height>;

pub type Label = Option<(usize, BoundaryPath, Coord)>;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Neighbourhood(FastHashMap<BoundaryPath, BoundaryEquivalences>);

impl Neighbourhood {
    pub(crate) fn insert(&mut self, b: BoundaryPath, base: &Diagram, signature: &impl Signature) {
        self.0
            .entry(b)
            .or_insert_with(|| BoundaryEquivalences::new(base, signature));
    }

    /// Checks whether two points in the given boundary are identified.
    pub fn equiv(&self, b: BoundaryPath, x: &Coord, y: &Coord) -> bool {
        self.0[&b].equiv(x, y)
    }

    /// Returns the canonical representative of the given point.
    pub fn find(&self, b: BoundaryPath, x: &Coord) -> Coord {
        self.0[&b].find(x)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct BoundaryEquivalences {
    nodes: Vec<Coord>,
    union_find: Vec<usize>,
}

impl BoundaryEquivalences {
    fn new(base: &Diagram, signature: &impl Signature) -> Self {
        let mut scaffold: StableScaffold<Coord> = StableScaffold::default();
        scaffold.add_node(base.clone().into());
        for _ in 0..base.dimension() {
            scaffold = scaffold
                .explode_simple(
                    |_, key, si| match si {
                        SliceIndex::Boundary(_) => None,
                        SliceIndex::Interior(h) => Some([key.as_slice(), &[h]].concat()),
                    },
                    |_, _, _| Some(()),
                    |_, _, _| Some(()),
                )
                .unwrap();
        }
        let nodes = scaffold.node_weights().map(|n| n.key.clone()).collect();
        let union_find = collapse(&mut scaffold, signature)
            .into_labeling()
            .into_iter()
            .map(petgraph::prelude::NodeIndex::index)
            .collect();
        Self { nodes, union_find }
    }

    fn equiv(&self, x: &Coord, y: &Coord) -> bool {
        let i = self.nodes.iter().position(|coord| coord == x).unwrap();
        let j = self.nodes.iter().position(|coord| coord == y).unwrap();
        self.union_find[i] == self.union_find[j]
    }

    fn find(&self, x: &Coord) -> Coord {
        let i = self.nodes.iter().position(|coord| coord == x).unwrap();
        let j = self.union_find[i];
        self.nodes[j].clone()
    }
}
