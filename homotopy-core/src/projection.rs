//! Diagrams of higher dimensions can be projected into 2 dimensions to be presented in a user
//! interface. This module contains diagram analyses which that calculate various aspects about the
//! 2-dimensional projection of a diagram.
//!
//! In order to avoid potentially costly recomputations and accidental quadratic complexity when a
//! diagram is traversed again for every point, the analyses are performed for the entire diagram
//! at once and the results are cached for efficient random-access retrieval.
use std::cmp::Ordering;

use homotopy_common::{
    hash::{FastHashMap, FastHasher},
    idx::IdxVec,
};
use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    visit::{EdgeRef, IntoNodeReferences, Topo, Walker},
    EdgeDirection,
};

use crate::{
    common::{DimensionError, Generator, SliceIndex},
    graph::{Explodable, SliceGraph},
    layout::Layout,
    Diagram, Direction, Height, Rewrite,
};

type Coordinate<const N: usize> = [SliceIndex; N];

#[derive(Copy, Clone, Debug)]
pub enum Homotopy {
    Complex,
    Duality,
    Identity,
    HalfBraid,
    FullBraid,
}

/// Diagram analysis that determines the generator displayed at any point in the 2-dimensional
/// projection of a diagram, as well as information about homotopies.
#[derive(Clone, Debug)]
pub struct Projection<const N: usize> {
    generators: IdxVec<NodeIndex, Generator>,
    front_generators: IdxVec<NodeIndex, Generator>,
    homotopies: IdxVec<NodeIndex, Option<Homotopy>>,
    coord_to_node: FastHashMap<Coordinate<N>, NodeIndex>,
}

impl<const N: usize> Projection<N> {
    pub fn new(
        diagram: &Diagram,
        layout: &Layout<N>,
        depths: &Depths<N>,
    ) -> Result<Self, DimensionError> {
        use Height::Singular;
        use SliceIndex::Interior;

        if diagram.dimension() < N {
            return Err(DimensionError);
        }

        // Construct the exploded graph.
        let mut graph: SliceGraph<Vec<SliceIndex>, _> =
            SliceGraph::singleton(vec![], diagram.clone());
        for i in 0..N {
            graph = graph
                .explode(
                    |_, key, si| {
                        let mut v = key.clone();
                        v.push(si);
                        Some(v)
                    },
                    |_, _, r| (i == 0).then_some(r.direction()),
                    |_, key, r| (i > 0 && r.is_atomic()).then_some(*key),
                )?
                .output;
        }
        let graph: SliceGraph<Coordinate<N>, _> = graph.map(
            |_, (indices, diagram)| (indices.clone().try_into().unwrap(), diagram.clone()),
            |_, e| e.clone(),
        );

        let mut generators = IdxVec::with_capacity(graph.node_count());
        let mut homotopies = IdxVec::with_capacity(graph.node_count());
        let mut front_generators = IdxVec::with_capacity(graph.node_count());
        let mut coord_to_node =
            FastHashMap::with_capacity_and_hasher(graph.node_count(), FastHasher::default());

        for n in graph.node_indices() {
            let coord = graph[n].0;

            let g = graph[n].1.max_generator();
            let front_g = match &graph[n].1 {
                Diagram::Diagram0(g) => *g,
                Diagram::DiagramN(d) => match depths.node_depth(coord) {
                    None => diagram.max_generator(),
                    Some(i) => d.slice(Height::Singular(i)).unwrap().max_generator(),
                },
            };

            let h = || -> Option<Homotopy> {
                if coord.iter().any(|x| !matches!(x, Interior(Singular(_)))) {
                    return None;
                }

                // Collect information about the incoming and outgoing strands.
                let mut inputs = 0;
                let mut input_depths = vec![];
                let mut input_rewrites = vec![];
                let mut input_coords = vec![];
                let mut outputs = 0;
                let mut output_depths = vec![];
                let mut output_rewrites = vec![];
                let mut output_coords = vec![];
                for e in graph.edges_directed(n, EdgeDirection::Incoming) {
                    let rewrite: &Rewrite = &e.weight().1;
                    let source_coord = graph[e.source()].0;
                    let depth = depths.edge_depth(source_coord, coord);

                    match e.weight().0 {
                        Direction::Forward => {
                            inputs += 1;
                            input_depths.push(depth);
                            input_rewrites.push(rewrite);
                            input_coords.push(source_coord);
                        }
                        Direction::Backward => {
                            outputs += 1;
                            output_depths.push(depth);
                            output_rewrites.push(rewrite);
                            output_coords.push(source_coord);
                        }
                    }
                }

                if inputs == 0 || outputs == 0 {
                    Some(Homotopy::Duality)
                } else if inputs == 1 && outputs == 1 {
                    if input_rewrites[0].is_identity() && output_rewrites[0].is_identity() {
                        Some(Homotopy::Identity)
                    } else {
                        Some(Homotopy::Complex)
                    }
                } else {
                    // Find the depth of the front layer.
                    let &min_depth = input_depths.iter().chain(&output_depths).min().unwrap();

                    // Collect the incoming and outgoing strands of the front layer.
                    let front_inputs = input_depths
                        .iter()
                        .enumerate()
                        .filter_map(
                            |(i, &depth)| {
                                if depth == min_depth {
                                    Some(i)
                                } else {
                                    None
                                }
                            },
                        )
                        .collect::<Vec<_>>();
                    let front_outputs = output_depths
                        .iter()
                        .enumerate()
                        .filter_map(
                            |(i, &depth)| {
                                if depth == min_depth {
                                    Some(i)
                                } else {
                                    None
                                }
                            },
                        )
                        .collect::<Vec<_>>();

                    if front_inputs.len() != 1 || front_outputs.len() != 1 {
                        return Some(Homotopy::Complex);
                    }

                    // If the front layer has one incoming strand and one outgoing strand, it's either a half braid or a full braid.
                    // To determine which, we check if the incoming strand crosses over the outgoing strand using the layout.
                    let &i = front_inputs.first().unwrap();
                    let &j = front_outputs.first().unwrap();
                    let position_node = layout.get(coord)[0];
                    let position_input_wire = layout.get(input_coords[i])[0];
                    let position_output_wire = layout.get(output_coords[j])[0];

                    if matches!(
                        (
                            position_node.partial_cmp(&position_input_wire),
                            position_node.partial_cmp(&position_output_wire)
                        ),
                        (Some(Ordering::Less), Some(Ordering::Greater))
                            | (Some(Ordering::Greater), Some(Ordering::Less))
                    ) {
                        Some(Homotopy::FullBraid)
                    } else {
                        Some(Homotopy::HalfBraid)
                    }
                }
            }();

            generators.push(g);
            front_generators.push(front_g);
            homotopies.push(h);
            coord_to_node.insert(coord, n);
        }

        Ok(Self {
            generators,
            front_generators,
            homotopies,
            coord_to_node,
        })
    }

    pub fn generator(&self, p: Coordinate<N>) -> Generator {
        self.generators[self.coord_to_node[&p]]
    }

    pub fn front_generator(&self, p: Coordinate<N>) -> Generator {
        self.front_generators[self.coord_to_node[&p]]
    }

    pub fn homotopy(&self, p: Coordinate<N>) -> Option<Homotopy> {
        self.homotopies[self.coord_to_node[&p]]
    }
}

/// Diagram analysis that finds the depth of cells in the 2-dimensional projection of a diagram.
#[derive(Debug, Clone)]
pub struct Depths<const N: usize> {
    graph: SliceGraph<Coordinate<N>>,
    node_depths: IdxVec<NodeIndex, Option<usize>>,
    edge_depths: IdxVec<EdgeIndex, Option<usize>>,
    coord_to_node: FastHashMap<Coordinate<N>, NodeIndex>,
}

impl<const N: usize> Depths<N> {
    pub fn new(diagram: &Diagram) -> Result<Self, DimensionError> {
        let mut graph: SliceGraph<Vec<SliceIndex>> = SliceGraph::singleton(vec![], diagram.clone());
        for _ in 0..N {
            graph = graph
                .explode(
                    |_, key, si| {
                        let mut v = key.clone();
                        v.push(si);
                        Some(v)
                    },
                    |_, _, _| Some(()),
                    |_, _, r| r.is_atomic().then_some(()),
                )?
                .output;
        }
        let graph: SliceGraph<Coordinate<N>, _> = graph.map(
            |_, (indices, diagram)| (indices.clone().try_into().unwrap(), diagram.clone()),
            |_, e| e.clone(),
        );

        let mut node_depths = IdxVec::splat(None, graph.node_count());
        let mut edge_depths = IdxVec::splat(None, graph.edge_count());

        let coord_to_node = graph
            .node_references()
            .map(|(n, (coord, _))| (*coord, n))
            .collect();

        for node in Topo::new(&graph).iter(&graph) {
            for edge in graph.edges_directed(node, EdgeDirection::Incoming) {
                if let ((), Rewrite::RewriteN(r)) = edge.weight() {
                    edge_depths[edge.id()] =
                        node_depths[edge.source()].map(|d| r.singular_image(d));

                    let target_depth = r.targets().first().copied();
                    node_depths[node] = min_defined(
                        min_defined(node_depths[node], edge_depths[edge.id()]),
                        target_depth,
                    );
                }
            }
        }

        Ok(Self {
            graph,
            node_depths,
            edge_depths,
            coord_to_node,
        })
    }

    pub fn node_depth(&self, coord: Coordinate<N>) -> Option<usize> {
        let &n = self.coord_to_node.get(&coord)?;
        self.node_depths[n]
    }

    pub fn edge_depth(&self, from: Coordinate<N>, to: Coordinate<N>) -> Option<usize> {
        let &from = self.coord_to_node.get(&from)?;
        let &to = self.coord_to_node.get(&to)?;
        let e = self
            .graph
            .edges_directed(from, EdgeDirection::Outgoing)
            .find(|&e| e.target() == to)?;
        self.edge_depths[e.id()]
    }

    pub fn edges_above(&self, depth: usize, to: Coordinate<N>) -> Vec<Coordinate<N>> {
        let to = match self.coord_to_node.get(&to) {
            Some(to) => *to,
            None => return vec![],
        };

        self.graph
            .edges_directed(to, EdgeDirection::Incoming)
            .filter_map(|e| match self.edge_depths[e.id()] {
                Some(d) if d < depth => self.graph.node_weight(e.source()).map(|(coord, _)| *coord),
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
