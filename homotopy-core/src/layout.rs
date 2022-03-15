use std::collections::HashMap;

use homotopy_common::{hash::FastHashMap, idx::IdxVec};
use itertools::Itertools;
use petgraph::{graph::NodeIndex, visit::EdgeRef, EdgeDirection, Graph};

use crate::{
    common::{DimensionError, SingularHeight},
    graph::{Explodable, SliceGraph},
    Boundary, DiagramN, Height, RewriteN, SliceIndex,
};

pub type Layout2D = Layout<2>;

#[derive(Clone, Debug)]
pub struct Layout<const N: usize> {
    positions: FastHashMap<[SliceIndex; N], [f32; N]>,
}

impl<const N: usize> Layout<N> {
    pub fn new(diagram: &DiagramN) -> Result<Self, DimensionError> {
        if diagram.dimension() < N {
            return Err(DimensionError);
        }

        let mut graph =
            SliceGraph::singleton(([Boundary::Source.into(); N], [0.0; N]), diagram.clone());

        for i in 0..N {
            let positions = layout(&graph, i, |v| &v.0, |e| *e)?;
            graph = graph
                .explode(
                    |n, key, si| {
                        let mut key = *key;
                        key.0[i] = si;
                        key.1[i] = positions[n][si];
                        Some(key)
                    },
                    |_, _, _| Some(i),
                    |_, j, r| r.is_atomic().then(|| *j),
                )?
                .output;
        }

        let positions = graph
            .into_nodes_edges()
            .0
            .into_iter()
            .map(|node| node.weight.0)
            .collect();

        Ok(Self { positions })
    }

    pub fn get(&self, path: [SliceIndex; N]) -> [f32; N] {
        let mut position = self.positions[&path];
        position.reverse();
        position
    }
}

pub type Name = (NodeIndex, SingularHeight);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NameEdge {
    Full,
    Partial(Extremum),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Extremum {
    Min,
    Max,
}

pub type ConstraintSet = Graph<Name, NameEdge>;

fn extrema(cs: &ConstraintSet, ext: Extremum) -> impl Iterator<Item = NodeIndex> + '_ {
    let dir = match ext {
        Extremum::Min => EdgeDirection::Incoming,
        Extremum::Max => EdgeDirection::Outgoing,
    };
    cs.node_indices().filter(move |&n| {
        !cs.edges_directed(n, dir)
            .any(|e| *e.weight() == NameEdge::Full)
    })
}

fn concat(a: &ConstraintSet, b: &ConstraintSet) -> ConstraintSet {
    let mut union = ConstraintSet::new();

    // Maps the nodes of a (resp. b) to the nodes of the union.
    let mut a_nodes: IdxVec<NodeIndex, NodeIndex> = IdxVec::new();
    let mut b_nodes: IdxVec<NodeIndex, NodeIndex> = IdxVec::new();

    // Copy of a
    for &name in a.node_weights() {
        a_nodes.push(union.add_node(name));
    }
    for e in a.edge_references() {
        let s = e.source();
        let t = e.target();
        union.add_edge(a_nodes[s], a_nodes[t], *e.weight());
    }

    // Copy of b
    for &name in b.node_weights() {
        b_nodes.push(union.add_node(name));
    }
    for e in b.edge_references() {
        let s = e.source();
        let t = e.target();
        union.add_edge(b_nodes[s], b_nodes[t], *e.weight());
    }

    // Edges from a to b
    for s in extrema(a, Extremum::Max) {
        for t in extrema(b, Extremum::Min) {
            union.add_edge(a_nodes[s], b_nodes[t], NameEdge::Full);
        }
    }

    union
}

fn colimit(constraints: &[ConstraintSet], partial: bool) -> ConstraintSet {
    let mut colimit = ConstraintSet::new();
    let mut name_to_node = HashMap::<Name, NodeIndex>::new();

    for constraint in constraints {
        for &name in constraint.node_weights() {
            name_to_node
                .entry(name)
                .or_insert_with(|| colimit.add_node(name));
        }
        for e in constraint.edge_references() {
            let s = e.source();
            let t = e.target();
            let name_s = constraint[s];
            let name_t = constraint[t];
            colimit.add_edge(name_to_node[&name_s], name_to_node[&name_t], *e.weight());
        }
    }

    if partial {
        for (i, constraint1) in constraints.iter().enumerate() {
            for (j, constraint2) in constraints.iter().enumerate() {
                if i != j {
                    for ext in [Extremum::Min, Extremum::Max] {
                        for s in extrema(constraint1, ext) {
                            let name_s = constraint1[s];
                            for t in extrema(constraint2, ext) {
                                let name_t = constraint2[t];
                                colimit.add_edge(
                                    name_to_node[&name_s],
                                    name_to_node[&name_t],
                                    NameEdge::Partial(ext),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    colimit
}

fn layout<V, E, F, G>(
    graph: &SliceGraph<V, E>,
    index: usize,
    coord_map: F,
    orientation_map: G,
) -> Result<IdxVec<NodeIndex, Vec<f32>>, DimensionError>
where
    F: Fn(&V) -> &[SliceIndex],
    G: Fn(&E) -> usize,
{
    // Sort nodes by stratum.
    let nodes = graph.node_indices().sorted_by_cached_key(|&n| {
        coord_map(&graph[n].0)
            .iter()
            .map(|&si| match si {
                SliceIndex::Boundary(_) => -1,
                SliceIndex::Interior(Height::Regular(_)) => 0,
                SliceIndex::Interior(Height::Singular(_)) => 1,
            })
            .sum::<isize>()
    });

    // Injectification
    let mut node_to_constraints: IdxVec<NodeIndex, Vec<ConstraintSet>> =
        IdxVec::splat(vec![], graph.node_count());
    for n in nodes {
        let diagram: &DiagramN = (&graph[n].1).try_into()?;

        for target_index in 0..diagram.size() {
            // Collect preimages.
            let mut preimages = vec![];
            let mut orientations = vec![];
            for e in graph.edges_directed(n, EdgeDirection::Incoming) {
                let s = e.source();
                let rewrite: &RewriteN = (&e.weight().1).try_into()?;

                let preimage = rewrite
                    .singular_preimage(target_index)
                    .map(|source_index| node_to_constraints[s][source_index].clone())
                    .reduce(|a, b| concat(&a, &b));

                if let Some(preimage) = preimage {
                    preimages.push(preimage);
                    orientations.push(orientation_map(&e.weight().0));
                }
            }

            let constraint = if preimages.is_empty() {
                // If there are no preimages, we insert a singleton constraint.
                let mut singleton = ConstraintSet::new();
                singleton.add_node((n, target_index));
                singleton
            } else {
                // Otherwise, we take a colimit of the preimages.
                colimit(
                    &preimages,
                    index > 0 && orientations == vec![index - 1, index - 1],
                )
            };

            node_to_constraints[n].push(constraint);
        }
    }

    // Colimit
    let maximal_constraints = node_to_constraints
        .iter()
        .filter_map(|(_n, constraints)| {
            // let maximal = graph[n].0.iter().all(|index| match index {
            //     SliceIndex::Interior(Height::Singular(_)) => true,
            //     _ => false,
            // });
            constraints
                .clone()
                .into_iter()
                .reduce(|a, b| concat(&a, &b))
        })
        .collect_vec();
    let colimit = colimit(&maximal_constraints, false);

    // For each point in the colimit, calculate its min and max positions.
    let mut width = 0;
    let mut name_to_min_position: HashMap<Name, usize> = HashMap::new();
    let mut name_to_max_position: HashMap<Name, usize> = HashMap::new();

    let mut full_colimit = colimit.clone();
    full_colimit.retain_edges(|graph, e| graph[e] == NameEdge::Full);

    // Left alignment
    let mut left_colimit = colimit.clone();
    left_colimit.retain_edges(|graph, e| graph[e] != NameEdge::Partial(Extremum::Max));
    for scc in petgraph::algo::kosaraju_scc(&left_colimit).iter().rev() {
        let pos = scc
            .iter()
            .flat_map(|&n| full_colimit.neighbors_directed(n, EdgeDirection::Incoming))
            .filter_map(|s| name_to_min_position.get(&colimit[s]))
            .max()
            .map_or(0, |&a| a + 1);
        for &n in scc {
            name_to_min_position.insert(colimit[n], pos);
        }
        width = std::cmp::max(width, pos + 1);
    }

    // Right alignment
    let mut right_colimit = colimit.clone();
    right_colimit.retain_edges(|graph, e| graph[e] != NameEdge::Partial(Extremum::Min));
    for scc in petgraph::algo::kosaraju_scc(&right_colimit) {
        let pos = scc
            .iter()
            .flat_map(|&n| full_colimit.neighbors_directed(n, EdgeDirection::Outgoing))
            .filter_map(|t| name_to_max_position.get(&colimit[t]))
            .max()
            .map_or(0, |&a| a + 1);
        for n in scc {
            name_to_max_position.insert(colimit[n], pos);
        }
        width = std::cmp::max(width, pos + 1);
    }

    // Calculate final layout by taking averages.
    let mut layout = IdxVec::new();
    for constraints in node_to_constraints.values() {
        let singular_positions = constraints
            .iter()
            .map(|cs| {
                let min = cs
                    .node_weights()
                    .map(|name| name_to_min_position[name])
                    .min()
                    .unwrap();
                let max = cs
                    .node_weights()
                    .map(|name| width - name_to_max_position[name] - 1)
                    .max()
                    .unwrap();
                (min, max)
            })
            .collect_vec();
        layout.push(compute_averages(width, singular_positions));
    }

    Ok(layout)
}

// Takes a list of minimum and maximum positions for every singular slice and computes the final positions.
fn compute_averages(width: usize, singular_positions: Vec<(usize, usize)>) -> Vec<f32> {
    let mut positions = vec![0.0];

    let mut start = 1.0;
    for (a, b) in singular_positions {
        let a = (2 * a + 2) as f32;
        let b = (2 * b + 2) as f32;
        positions.push((start + a - 1.0) / 2.0);
        positions.push((a + b) / 2.0);
        start = b + 1.0;
    }
    positions.push((start + 2.0 * width as f32 + 1.0) / 2.0);
    positions.push(2.0 * width as f32 + 2.0);
    positions
}
