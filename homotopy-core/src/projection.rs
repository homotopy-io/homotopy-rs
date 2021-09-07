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

use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::{EdgeRef, IntoNodeReferences, Topo, Walker},
    EdgeDirection,
};
use serde::Serialize;

use crate::{
    common::{Boundary, Generator, SliceIndex},
    diagram::DiagramN,
    graph::GraphBuilder,
    Diagram, Rewrite,
};

/// Diagram analysis that determines the generator displayed at any point in the 2-dimensional
/// projection of a diagram. Currently this is the first maximum-dimensional generator, but will
/// change to incorporate information about homotopies.
#[derive(Debug, Clone, Serialize)]
pub struct Generators(Vec<Vec<Generator>>);

impl Generators {
    pub fn new(diagram: &DiagramN) -> Self {
        if diagram.dimension() < 2 {
            // TODO: Make this into an error
            panic!();
        }

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
    graph: DiGraph<([SliceIndex; 2], Diagram), Rewrite>,
    coord_to_node: HashMap<[SliceIndex; 2], NodeIndex<u32>>,
    edge_depths: Vec<Option<usize>>,
    node_depths: Vec<Option<usize>>,
}

impl Depths {
    pub fn new(diagram: &DiagramN) -> Self {
        if diagram.dimension() < 2 {
            panic!();
        }

        let graph = GraphBuilder::new((), diagram.clone().into())
            .explode(|y, ()| y)
            .unwrap()
            .explode(|x, y| [y, x])
            .unwrap()
            .build();

        let coord_to_node = graph.node_references().map(|n| (n.1 .0, n.0)).collect();

        let mut edge_depths: Vec<Option<usize>> = [None].repeat(graph.edge_count());
        let mut node_depths: Vec<Option<usize>> = [None].repeat(graph.node_count());

        for node in Topo::new(&graph).iter(&graph) {
            let mut node_depth = None;

            for edge in graph.edges_directed(node, EdgeDirection::Incoming) {
                if let Rewrite::RewriteN(r) = edge.weight() {
                    let edge_depth =
                        node_depths[edge.source().index()].map(|d| r.singular_image(d));
                    edge_depths[edge.id().index()] = edge_depth;

                    let target_depth = r.targets().first().copied();
                    node_depth = min_defined(min_defined(node_depth, edge_depth), target_depth);
                };
            }

            node_depths[node.index()] = node_depth;
        }

        Self {
            graph,
            coord_to_node,
            edge_depths,
            node_depths,
        }
    }

    pub fn node_depth(&self, coord: [SliceIndex; 2]) -> Option<usize> {
        let node = *self.coord_to_node.get(&coord)?;
        self.node_depths[node.index()]
    }

    pub fn edge_depth(&self, from: [SliceIndex; 2], to: [SliceIndex; 2]) -> Option<usize> {
        let from = *self.coord_to_node.get(&from)?;
        let to = *self.coord_to_node.get(&to)?;
        let edge = self.graph.edges_connecting(from, to).next()?;
        self.edge_depths[edge.id().index()]
    }

    pub fn edges_above(&self, depth: usize, to: [SliceIndex; 2]) -> Vec<[SliceIndex; 2]> {
        let to = match self.coord_to_node.get(&to) {
            Some(to) => *to,
            None => return vec![],
        };

        self.graph
            .edges_directed(to, EdgeDirection::Incoming)
            .filter_map(|e| match self.edge_depths[e.id().index()] {
                Some(d) if d < depth => self.graph.node_weight(e.source()).map(|e| e.0),
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
