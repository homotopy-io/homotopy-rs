use std::collections::HashMap;

use homotopy_common::{graph::Node, idx::IdxVec};
use itertools::Itertools;
use petgraph::{graph::NodeIndex, visit::EdgeRef};

use crate::{
    common::{DimensionError, SingularHeight},
    graph::{explode_graph, SliceGraph, TopologicalSort},
    DiagramN, RewriteN, SliceIndex,
};

// TODO(@calintat): Clean this up.
#[derive(Clone, Debug)]
pub struct Layout {
    pub x_coords: Vec<f32>,
    pub y_coords: HashMap<SliceIndex, Vec<f32>>,
    pub z_coords: HashMap<(SliceIndex, SliceIndex), Vec<f32>>,
    pub w_coords: HashMap<(SliceIndex, SliceIndex, SliceIndex), Vec<f32>>,
}

impl Layout {
    pub fn new(diagram: &DiagramN, depth: usize) -> Result<Self, DimensionError> {
        if depth > diagram.dimension() {
            return Err(DimensionError);
        }
        let mut layout = Self {
            x_coords: vec![],
            y_coords: HashMap::new(),
            z_coords: HashMap::new(),
            w_coords: HashMap::new(),
        };

        let mut graph = SliceGraph::new();
        graph.add_node((vec![], diagram.clone().into()));

        for _ in 0..depth {
            let node_to_constraints = calculate_constraints(&graph)?;
            let colimit = take_colimit(&graph, &node_to_constraints);
            let positions = calculate_layout(&node_to_constraints, &colimit);

            for (n, coords) in positions {
                let path = &graph[n].0;
                layout.add_coordinates(path, coords);
            }

            graph = explode_graph(&graph)?.0;
        }

        Ok(layout)
    }

    fn add_coordinates(&mut self, path: &[SliceIndex], coords: Vec<f32>) {
        match path.len() {
            0 => {
                self.x_coords = coords;
            }
            1 => {
                self.y_coords.insert(path[0], coords);
            }
            2 => {
                self.z_coords.insert((path[0], path[1]), coords);
            }
            3 => {
                self.w_coords.insert((path[0], path[1], path[2]), coords);
            }
            _ => {
                panic!("Only 4D coordinates are supported!");
            }
        }
    }

    pub fn get(&self, path: &[SliceIndex]) -> Vec<f32> {
        let mut coord = vec![];
        if !path.is_empty() {
            coord.push(self.x_coords[path[0]]);
        }
        if path.len() > 1 {
            coord.push(self.y_coords[&path[0]][path[1]]);
        }
        if path.len() > 2 {
            coord.push(self.z_coords[&(path[0], path[1])][path[2]]);
        }
        if path.len() > 3 {
            coord.push(self.w_coords[&(path[0], path[1], path[2])][path[3]]);
        }
        coord.reverse();
        coord
    }

    pub fn get2<I, J>(&self, x: I, y: J) -> (f32, f32)
    where
        I: Into<SliceIndex>,
        J: Into<SliceIndex>,
    {
        let (y, x) = (x.into(), y.into());
        (self.y_coords[&x][y], self.x_coords[x])
    }
}

pub type Name = (Node, SingularHeight);
pub type ConstraintSet = petgraph::Graph<Name, ()>;

fn concat(a: &ConstraintSet, b: &ConstraintSet) -> ConstraintSet {
    let mut union = ConstraintSet::new();

    // Maps the nodes of a (resp. b) to the nodes of the union.
    let mut a_nodes: Vec<NodeIndex> = Vec::new();
    let mut b_nodes: Vec<NodeIndex> = Vec::new();

    // Copy of a
    for n in a.node_indices() {
        a_nodes.push(union.add_node(a[n]));
    }
    for ed in a.edge_references() {
        let s = ed.source();
        let t = ed.target();
        union.add_edge(a_nodes[s.index()], a_nodes[t.index()], ());
    }

    // Copy of b
    for n in b.node_indices() {
        b_nodes.push(union.add_node(b[n]));
    }
    for ed in b.edge_references() {
        let s = ed.source();
        let t = ed.target();
        union.add_edge(b_nodes[s.index()], b_nodes[t.index()], ());
    }

    // Edges from a to b
    for s in a.externals(petgraph::EdgeDirection::Outgoing) {
        for t in b.externals(petgraph::EdgeDirection::Incoming) {
            union.add_edge(a_nodes[s.index()], b_nodes[t.index()], ());
        }
    }

    union
}

fn colimit<I>(constraints: I) -> ConstraintSet
where
    I: IntoIterator<Item = ConstraintSet>,
{
    let mut colimit = ConstraintSet::new();
    let mut name_to_node = HashMap::<Name, NodeIndex>::new();

    for constraint in constraints {
        for name in constraint.node_weights() {
            name_to_node
                .entry(*name)
                .or_insert_with(|| colimit.add_node(*name));
        }
        for ed in constraint.edge_references() {
            let s = ed.source();
            let t = ed.target();
            let name_s = constraint[s];
            let name_t = constraint[t];
            colimit.update_edge(name_to_node[&name_s], name_to_node[&name_t], ());
        }
    }

    colimit
}

fn calculate_constraints(
    graph: &SliceGraph,
) -> Result<IdxVec<Node, Vec<ConstraintSet>>, DimensionError> {
    let mut node_to_constraints: IdxVec<Node, Vec<ConstraintSet>> =
        IdxVec::from_iter(vec![vec![]; graph.node_count()]);

    for n in TopologicalSort::new(graph) {
        let diagram: &DiagramN = (&graph[n].1).try_into()?;

        for target_index in 0..diagram.size() {
            // Collect preimages.
            let mut preimages = vec![];
            for e in graph.incoming_edges(n) {
                let s = graph.source(e);
                let rewrite: &RewriteN = (&graph[e]).try_into()?;

                let preimage = rewrite
                    .singular_preimage(target_index)
                    .map(|source_index| node_to_constraints[s][source_index].clone())
                    .reduce(|a, b| concat(&a, &b));

                if let Some(preimage) = preimage {
                    preimages.push(preimage);
                }
            }

            let constraint = if preimages.is_empty() {
                // If there are no preimages, we insert a singleton constraint.
                let mut singleton = ConstraintSet::new();
                singleton.add_node((n, target_index));
                singleton
            } else {
                // Otherwise, we take a colimit of the preimages.
                colimit(preimages)
            };

            node_to_constraints[n].push(constraint);
        }
    }

    Ok(node_to_constraints)
}

fn take_colimit(
    _graph: &SliceGraph,
    node_to_constraints: &IdxVec<Node, Vec<ConstraintSet>>,
) -> ConstraintSet {
    let maximal_constraints = node_to_constraints.iter().filter_map(|(_n, constraints)| {
        // let maximal = graph[n].0.iter().all(|index| match index {
        //     SliceIndex::Interior(Height::Singular(_)) => true,
        //     _ => false,
        // });
        constraints
            .clone()
            .into_iter()
            .reduce(|a, b| concat(&a, &b))
    });

    colimit(maximal_constraints)
}

fn calculate_layout(
    node_to_constraints: &IdxVec<Node, Vec<ConstraintSet>>,
    colimit: &ConstraintSet,
) -> IdxVec<Node, Vec<f32>> {
    // For each point in the colimit, calculate its min and max positions.
    let sccs = petgraph::algo::kosaraju_scc(colimit);
    let mut width = 0;
    let mut name_to_min_position: HashMap<Name, usize> = HashMap::new();
    let mut name_to_max_position: HashMap<Name, usize> = HashMap::new();

    for scc in sccs.iter().rev() {
        let pos = scc
            .iter()
            .flat_map(|&n| colimit.neighbors_directed(n, petgraph::EdgeDirection::Incoming))
            .filter_map(|s| name_to_min_position.get(&colimit[s]))
            .max()
            .map_or(0, |&a| a + 1);
        for &n in scc {
            name_to_min_position.insert(colimit[n], pos);
        }
        width = std::cmp::max(width, pos + 1);
    }
    for scc in &sccs {
        let pos = scc
            .iter()
            .flat_map(|&n| colimit.neighbors_directed(n, petgraph::EdgeDirection::Outgoing))
            .filter_map(|t| name_to_max_position.get(&colimit[t]))
            .min()
            .map_or(width - 1, |&a| a - 1);
        for &n in scc {
            name_to_max_position.insert(colimit[n], pos);
        }
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
                    .map(|name| name_to_max_position[name])
                    .max()
                    .unwrap();
                (min, max)
            })
            .collect_vec();
        layout.push(compute_averages(width, singular_positions));
    }

    layout
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
