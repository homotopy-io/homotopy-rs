//! Diagrams of higher dimensions can be projected into 2 dimensions to be presented in a user
//! interface. This module contains diagram analyses which that calculate various aspects about the
//! 2-dimensional projection of a diagram.
//!
//! In order to avoid potentially costly recomputations and accidental quadratic complexity when a
//! diagram is traversed again for every point, the analyses are performed for the entire diagram
//! at once and the results are cached for efficient random-access retrieval.
use crate::common::*;
use crate::diagram::DiagramN;
use crate::graph::GraphBuilder;
use crate::rewrite::RewriteN;
use petgraph::{
    graph::{DiGraph, NodeIndex},
    EdgeDirection,
};
use serde::Serialize;
use std::{collections::HashMap, convert::*};

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
        Generators(
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
            SliceIndex::Boundary(Boundary::Source) => slice.first().cloned(),
            SliceIndex::Boundary(Boundary::Target) => slice.last().cloned(),
            SliceIndex::Interior(height) => slice.get(height.to_int()).cloned(),
        }
    }
}

/// Diagram analysis that finds the depth of cells in the 2-dimensional projection of a diagram.
#[derive(Debug, Clone)]
pub struct Depths {
    graph: DiGraph<[SliceIndex; 2], Option<usize>>,
    coord_to_node: HashMap<[SliceIndex; 2], NodeIndex<u32>>,
}

impl Depths {
    pub fn new(diagram: &DiagramN) -> Self {
        if diagram.dimension() < 2 {
            panic!();
        }

        let graph_builder = GraphBuilder::new((), diagram.clone().into())
            .explode(|y, ()| y)
            .unwrap()
            .explode(|x, y| [y, x])
            .unwrap();

        let mut graph = DiGraph::<[SliceIndex; 2], Option<usize>>::with_capacity(
            graph_builder.nodes.len(),
            graph_builder.edges.len(),
        );

        let mut coord_to_node: HashMap<[SliceIndex; 2], NodeIndex<u32>> = HashMap::new();

        for (coords, _) in &graph_builder.nodes {
            coord_to_node.insert(*coords, graph.add_node(*coords));
        }

        for (source, target, rewrite) in &graph_builder.edges {
            let depth = <&RewriteN>::try_from(rewrite)
                .ok()
                .map(|r| r.targets().last().cloned())
                .flatten();
            graph.add_edge((*source as u32).into(), (*target as u32).into(), depth);
        }

        Depths {
            graph,
            coord_to_node,
        }
    }

    pub fn node_depth(&self, coord: [SliceIndex; 2]) -> Option<usize> {
        let node = *self.coord_to_node.get(&coord)?;
        self.graph
            .edges_directed(node, EdgeDirection::Incoming)
            .map(|e| *e.weight())
            .fold(None, min_defined)
    }

    pub fn edge_depth(&self, from: [SliceIndex; 2], to: [SliceIndex; 2]) -> Option<usize> {
        let from = *self.coord_to_node.get(&from)?;
        let to = *self.coord_to_node.get(&to)?;
        let edge = self.graph.edges_connecting(from, to).next()?;
        *edge.weight()
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
