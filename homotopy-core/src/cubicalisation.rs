use homotopy_common::graph::{Edge, Node};

use crate::{
    common::{DimensionError, Direction, SliceIndex},
    diagram::Diagram,
    graph::SliceGraph,
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Bias {
    Left,
    Right,
}

// Cubicalisation

impl Diagram {
    pub fn cubicalise(self, _biases: &[Bias]) -> Result<CubicalGraph, DimensionError> {
        todo!()
    }
}

// Cubical graphs

type Orientation = usize;

#[derive(Clone, Debug)]
pub struct CubicalGraph;

pub struct Square {
    pub top: Edge,
    pub left: Edge,
    pub right: Edge,
    pub bottom: Edge,
    pub orientation: [Orientation; 2],
}

pub struct Cube {
    pub top_front: Edge,
    pub left_front: Edge,
    pub right_front: Edge,
    pub bottom_front: Edge,
    pub top_back: Edge,
    pub left_back: Edge,
    pub right_back: Edge,
    pub bottom_back: Edge,
    pub top_left: Edge,
    pub top_right: Edge,
    pub bottom_left: Edge,
    pub bottom_right: Edge,
    pub orientation: [Orientation; 3],
}

impl CubicalGraph {
    pub fn new(_diagram: Diagram) -> Self {
        todo!()
    }

    pub fn inner(&self) -> &SliceGraph {
        todo!()
    }

    pub fn size(&self, _direction: usize) -> usize {
        todo!()
    }

    pub fn dimension(&self) -> usize {
        todo!()
    }

    pub fn label(&self, _node: Node) -> &[SliceIndex] {
        todo!()
    }

    pub fn get_direction(&self, _edge: Edge) -> Direction {
        todo!()
    }

    /// Returns all squares in the graph.
    pub fn squares(&self) -> Vec<Square> {
        todo!()
    }

    /// Returns all cubes in the graph.
    pub fn cubes(&self) -> Vec<Cube> {
        todo!()
    }
}
