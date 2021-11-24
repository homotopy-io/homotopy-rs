//! Diagrams of higher dimensions can be projected into 2 dimensions to be presented in a user
//! interface. This module contains diagram analyses which that calculate various aspects about the
//! 2-dimensional projection of a diagram.
//!
//! In order to avoid potentially costly recomputations and accidental quadratic complexity when a
//! diagram is traversed again for every point, the analyses are performed for the entire diagram
//! at once and the results are cached for efficient random-access retrieval.
use std::{
    collections::HashMap,
    convert::{Into, TryFrom},
};

use homotopy_common::{
    graph::{Edge, Node},
    idx::IdxVec,
};
use serde::Serialize;

use crate::{
    common::{Boundary, DimensionError, Generator, SliceIndex},
    diagram::DiagramN,
    graph::{GraphBuilder, SliceGraph, TopologicalSort},
    Rewrite,
};

/// Diagram analysis that determines the generator displayed at any point in the 2-dimensional
/// projection of a diagram. Currently this is the first maximum-dimensional generator, but will
/// change to incorporate information about homotopies.
#[derive(Debug, Clone, Serialize)]
pub struct Generators(Vec<Vec<Generator>>);

impl Generators {
    pub fn new(diagram: &DiagramN) -> Self {
        assert!(diagram.dimension() >= 2, "TODO: Make this into an error");

        // TODO: Projection
        Self(
            diagram
                .slices()
                .map(|slice| {
                    DiagramN::try_from(slice)
                        .unwrap()
                        .slices()
                        .map(|p| p.max_generator())
                        .collect()
                })
                .collect(),
        )
    }

    pub fn get(&self, x: SliceIndex, y: SliceIndex) -> Option<Generator> {
        let slice = match y {
            SliceIndex::Boundary(Boundary::Source) => self.0.first()?,
            SliceIndex::Boundary(Boundary::Target) => self.0.last()?,
            SliceIndex::Interior(height) => self.0.get(height.to_int())?,
        };

        match x {
            SliceIndex::Boundary(Boundary::Source) => slice.first().copied(),
            SliceIndex::Boundary(Boundary::Target) => slice.last().copied(),
            SliceIndex::Interior(height) => slice.get(height.to_int()).copied(),
        }
    }
}

/// Diagram analysis that finds the depth of cells in the 2-dimensional projection of a diagram.
#[derive(Debug, Clone)]
pub struct Depths {
    graph: SliceGraph,
    node_depths: IdxVec<Node, Option<usize>>,
    edge_depths: IdxVec<Edge, Option<usize>>,
    coord_to_node: HashMap<[SliceIndex; 2], Node>,
}

impl Depths {
    pub fn new(diagram: &DiagramN) -> Result<Self, DimensionError> {
        let graph = GraphBuilder::build(diagram.clone().into(), 2)?;

        let mut node_depths = IdxVec::splat(None, graph.node_count());
        let mut edge_depths = IdxVec::splat(None, graph.edge_count());

        for node in TopologicalSort::new(&graph) {
            for edge in graph.incoming_edges(node) {
                if let Rewrite::RewriteN(r) = &graph[edge] {
                    edge_depths[edge] =
                        node_depths[graph.source(edge)].map(|d| r.singular_image(d));

                    let target_depth = r.targets().first().copied();
                    node_depths[node] = min_defined(
                        target_depth,
                        min_defined(node_depths[node], edge_depths[edge]),
                    );
                }
            }
        }

        let coord_to_node = graph
            .nodes()
            .map(|(n, nd)| ([nd.0[0], nd.0[1]], n))
            .collect();

        Ok(Self {
            graph,
            node_depths,
            edge_depths,
            coord_to_node,
        })
    }

    pub fn node_depth(&self, coord: [SliceIndex; 2]) -> Option<usize> {
        let &n = self.coord_to_node.get(&coord)?;
        self.node_depths[n]
    }

    pub fn edge_depth(&self, from: [SliceIndex; 2], to: [SliceIndex; 2]) -> Option<usize> {
        let &from = self.coord_to_node.get(&from)?;
        let &to = self.coord_to_node.get(&to)?;
        let e = self
            .graph
            .outgoing_edges(from)
            .find(|&e| self.graph.target(e) == to)?;
        self.edge_depths[e]
    }

    pub fn edges_above(&self, depth: usize, to: [SliceIndex; 2]) -> Vec<[SliceIndex; 2]> {
        let to = match self.coord_to_node.get(&to) {
            Some(to) => *to,
            None => return vec![],
        };

        self.graph
            .incoming_edges(to)
            .filter_map(|e| match self.edge_depths[e] {
                Some(d) if d < depth => self
                    .graph
                    .node_weight(self.graph.source(e))
                    .map(|(coord, _)| [coord[0], coord[1]]),
                _ => None,
            })
            .collect()
    }
}

fn min_defined<T>(a: Option<T>, b: Option<T>) -> Option<T>
where
    T: Ord,
{
    match (a, b) {
        (None, None) => None,
        (None, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
    }
}
