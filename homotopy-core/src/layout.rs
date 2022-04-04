use std::collections::HashMap;

use homotopy_common::{hash::FastHashMap, idx::IdxVec};
use itertools::Itertools;
use petgraph::{graph::NodeIndex, graphmap::DiGraphMap, visit::EdgeRef, EdgeDirection};

use crate::{
    common::{DimensionError, SingularHeight},
    graph::{Explodable, SliceGraph},
    Boundary, DiagramN, Height, RewriteN, SliceIndex,
};

pub type Layout2D = Layout<2>;

#[derive(Clone, Debug)]
pub struct Layout<const N: usize> {
    pub positions: FastHashMap<[SliceIndex; N], [f32; N]>,
}

impl<const N: usize> Layout<N> {
    pub fn new(diagram: &DiagramN) -> Result<Self, DimensionError> {
        if diagram.dimension() < N {
            return Err(DimensionError);
        }

        let mut graph =
            SliceGraph::singleton(([Boundary::Source.into(); N], [0.0; N]), diagram.clone());

        for i in 0..N {
            let positions = layout(&graph, i, |key| &key.0[..i])?;
            graph = graph
                .explode(
                    |n, key, si| {
                        let mut key = *key;
                        key.0[i] = si;
                        key.1[i] = positions[n][si];
                        Some(key)
                    },
                    |_, _, _| Some(()),
                    |_, _, r| r.is_atomic().then(|| ()),
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

pub type Point = (NodeIndex, SingularHeight);

pub type ConstraintSet = DiGraphMap<Point, ()>;

fn extrema(cs: &ConstraintSet, dir: EdgeDirection) -> impl Iterator<Item = Point> + '_ {
    cs.nodes()
        .filter(move |p| cs.edges_directed(*p, dir).next().is_none())
}

fn concat(lhs: &ConstraintSet, rhs: &ConstraintSet) -> ConstraintSet {
    let mut union = ConstraintSet::new();

    // Copy of `lhs`.
    for n in lhs.nodes() {
        union.add_node(n);
    }
    for (a, b, _) in lhs.all_edges() {
        union.add_edge(a, b, ());
    }

    // Copy of `rhs`.
    for n in rhs.nodes() {
        union.add_node(n);
    }
    for (a, b, _) in rhs.all_edges() {
        union.add_edge(a, b, ());
    }

    // Edges from `lhs` to `rhs`.
    for a in extrema(lhs, EdgeDirection::Outgoing) {
        for b in extrema(rhs, EdgeDirection::Incoming) {
            union.add_edge(a, b, ());
        }
    }

    union
}

fn colimit(constraints: &[ConstraintSet]) -> ConstraintSet {
    let mut colimit = ConstraintSet::new();

    for constraint in constraints {
        for n in constraint.nodes() {
            colimit.add_node(n);
        }
        for (a, b, _) in constraint.all_edges() {
            colimit.add_edge(a, b, ());
        }
    }

    colimit
}

fn layout<V, E, F>(
    graph: &SliceGraph<V, E>,
    dim: usize,
    coord_map: F,
) -> Result<IdxVec<NodeIndex, Vec<f32>>, DimensionError>
where
    F: Fn(&V) -> &[SliceIndex],
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
            for e in graph.edges_directed(n, EdgeDirection::Incoming) {
                let s = e.source();
                let rewrite: &RewriteN = (&e.weight().1).try_into()?;

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
                colimit(&preimages)
            };

            node_to_constraints[n].push(constraint);
        }
    }

    // Colimit
    let maximal_constraints = node_to_constraints
        .iter()
        .filter_map(|(_n, constraints)| {
            constraints
                .clone()
                .into_iter()
                .reduce(|a, b| concat(&a, &b))
        })
        .collect_vec();
    let colimit = colimit(&maximal_constraints);

    let (width, positions) = if dim <= 1 {
        solve_2d(&node_to_constraints, &colimit)
    } else {
        solve_3d(graph, coord_map, &node_to_constraints, &colimit)
    };

    // Calculate final layout by taking averages.
    let mut layout = IdxVec::new();
    for constraints in node_to_constraints.values() {
        let singular_positions = constraints
            .iter()
            .map(|cs| {
                let min = cs
                    .nodes()
                    .map(|n| positions[&n].0)
                    .min_by(|x, y| x.partial_cmp(y).unwrap())
                    .unwrap();
                let max = cs
                    .nodes()
                    .map(|n| positions[&n].1)
                    .max_by(|x, y| x.partial_cmp(y).unwrap())
                    .unwrap();
                (min, max)
            })
            .collect_vec();
        layout.push(compute_averages(width, singular_positions));
    }

    Ok(layout)
}

fn solve_2d(
    node_to_constraints: &IdxVec<NodeIndex, Vec<ConstraintSet>>,
    colimit: &ConstraintSet,
) -> (f32, HashMap<Point, (f32, f32)>) {
    let mut problem = minilp::Problem::new(minilp::OptimizationDirection::Minimize);

    // Variables
    let mut variables = HashMap::new();
    for n in colimit.nodes() {
        variables.insert(n, problem.add_var(0.0, (0.0, f64::INFINITY)));
    }

    // Distance constraints.
    for (a, b, _) in colimit.all_edges() {
        let x = variables[&a];
        let y = variables[&b];
        problem.add_constraint(&[(x, -1.0), (y, 1.0)], minilp::ComparisonOp::Ge, 1.0);
    }

    // Fair averaging constraints (inc. straight wires).
    for css in node_to_constraints.values() {
        for cs in css {
            let mut paths: Vec<Vec<Point>> = vec![];
            for min in extrema(cs, EdgeDirection::Incoming) {
                for max in extrema(cs, EdgeDirection::Outgoing) {
                    if min == max {
                        paths.push(vec![min]);
                    } else if petgraph::algo::has_path_connecting(cs, min, max, None) {
                        paths.push(vec![min, max]);
                    }
                }
            }
            if paths.len() > 1 {
                let c = problem.add_var(0.0, (0.0, f64::INFINITY));
                for path in paths {
                    problem.add_constraint(
                        std::iter::once((c, path.len() as f64))
                            .chain(path.iter().map(|p| (variables[p], -1.0))),
                        minilp::ComparisonOp::Eq,
                        0.0,
                    );
                }
            }
        }
    }

    let solution = problem.solve().unwrap();
    let positions: HashMap<Point, (f32, f32)> = variables
        .into_iter()
        .map(|(p, x)| (p, (solution[x] as f32, solution[x] as f32)))
        .collect();

    let width = colimit
        .nodes()
        .map(|n| positions[&n].0 as f32 + 1.0)
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap_or_default();

    (width, positions)
}

fn solve_3d<V, E, F>(
    graph: &SliceGraph<V, E>,
    coord_map: F,
    node_to_constraints: &IdxVec<NodeIndex, Vec<ConstraintSet>>,
    colimit: &ConstraintSet,
) -> (f32, HashMap<Point, (f32, f32)>)
where
    F: Fn(&V) -> &[SliceIndex],
{
    // For each point in the colimit, calculate its min and max positions.
    let mut width = 0;
    let mut min_positions: HashMap<Point, usize> = HashMap::new();
    let mut max_positions: HashMap<Point, usize> = HashMap::new();

    // Straight wires.
    let mut left_colimit = colimit.clone();
    let mut right_colimit = colimit.clone();
    for n in graph.node_indices() {
        let coord = coord_map(&graph[n].0);
        let dim = coord.len();
        if dim > 0
            && coord[..dim - 1]
                .iter()
                .all(|si| matches!(si, SliceIndex::Interior(Height::Regular(_))))
            && matches!(coord[dim - 1], SliceIndex::Interior(Height::Singular(_)))
        {
            for constraints in &node_to_constraints[n] {
                for a in extrema(constraints, EdgeDirection::Incoming) {
                    for b in extrema(constraints, EdgeDirection::Incoming) {
                        if a != b {
                            left_colimit.add_edge(a, b, ());
                        }
                    }
                }
                for a in extrema(constraints, EdgeDirection::Outgoing) {
                    for b in extrema(constraints, EdgeDirection::Outgoing) {
                        if a != b {
                            right_colimit.add_edge(a, b, ());
                        }
                    }
                }
            }
        }
    }

    // Left alignment
    for scc in petgraph::algo::kosaraju_scc(&left_colimit).iter().rev() {
        let pos = scc
            .iter()
            .flat_map(|&n| colimit.neighbors_directed(n, EdgeDirection::Incoming))
            .filter_map(|s| min_positions.get(&s))
            .max()
            .map_or(0, |&a| a + 1);
        for &n in scc {
            min_positions.insert(n, pos);
        }
        width = std::cmp::max(width, pos + 1);
    }

    // Right alignment
    for scc in petgraph::algo::kosaraju_scc(&right_colimit) {
        let pos = scc
            .iter()
            .flat_map(|&n| colimit.neighbors_directed(n, EdgeDirection::Outgoing))
            .filter_map(|t| max_positions.get(&t))
            .max()
            .map_or(0, |&a| a + 1);
        for n in scc {
            max_positions.insert(n, pos);
        }
        width = std::cmp::max(width, pos + 1);
    }

    (
        width as f32,
        colimit
            .nodes()
            .map(|n| {
                (
                    n,
                    (
                        min_positions[&n] as f32,
                        (width - max_positions[&n] - 1) as f32,
                    ),
                )
            })
            .collect(),
    )
}

// Takes a list of minimum and maximum positions for every singular slice and computes the final positions.
fn compute_averages(width: f32, singular_positions: Vec<(f32, f32)>) -> Vec<f32> {
    let mut positions = vec![0.0];

    let mut start = 1.0;
    for (a, b) in singular_positions {
        let a = 2.0 * a + 2.0;
        let b = 2.0 * b + 2.0;
        positions.push((start + a - 1.0) / 2.0);
        positions.push((a + b) / 2.0);
        start = b + 1.0;
    }
    positions.push((start + 2.0 * width + 1.0) / 2.0);
    positions.push(2.0 * width + 2.0);
    positions
}
