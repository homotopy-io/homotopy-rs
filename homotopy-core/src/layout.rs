use std::ops::{Deref, DerefMut};

use good_lp::{
    default_solver, variable, Constraint, Expression, ProblemVariables, Solution, SolverModel,
    Variable,
};
use homotopy_common::{
    hash::{FastHashMap, FastHashSet},
    idx::IdxVec,
};
use itertools::Itertools;
use petgraph::{graph::NodeIndex, visit::EdgeRef, EdgeDirection, Graph};

use crate::{
    common::{DimensionError, SingularHeight},
    graph::{Explodable, SliceGraph},
    Boundary, Diagram, DiagramN, Direction, Height, RewriteN, SliceIndex,
};

#[derive(Clone, Debug)]
pub struct Layout<const N: usize> {
    pub positions: FastHashMap<[SliceIndex; N], [f32; N]>,
}

impl<const N: usize> Layout<N> {
    pub fn new(diagram: &Diagram) -> Result<Self, DimensionError> {
        if diagram.dimension() < N {
            return Err(DimensionError);
        }

        let mut graph =
            SliceGraph::singleton(([Boundary::Source.into(); N], [0.0; N]), diagram.clone());

        for i in 0..N {
            let positions = layout(&graph, i, |key| &key.0[..i], |key| *key)?;
            graph = graph
                .explode(
                    |n, key, si| {
                        let mut key = *key;
                        key.0[i] = si;
                        key.1[i] = positions[n][si];
                        Some(key)
                    },
                    |_, _, r| Some((i, r.direction())),
                    |_, key, r| (!r.is_flange()).then_some(*key),
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
        self.get_checked(path).unwrap()
    }

    // TODO(@calintat): Remove this once #712 is fixed.
    pub fn get_checked(&self, path: [SliceIndex; N]) -> Option<[f32; N]> {
        let mut position = *self.positions.get(&path)?;
        position.reverse();
        Some(position)
    }
}

pub type Point = (NodeIndex, SingularHeight);

#[derive(Clone, Debug, Default)]
pub struct ConstraintSet {
    graph: Graph<Point, ()>,
    ins: FastHashSet<NodeIndex>,
    outs: FastHashSet<NodeIndex>,
    orientation: Option<usize>,
}

impl Deref for ConstraintSet {
    type Target = Graph<Point, ()>;

    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}

impl DerefMut for ConstraintSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}

fn concat(lhs: &ConstraintSet, rhs: &ConstraintSet) -> ConstraintSet {
    let mut union = ConstraintSet::default();

    // Copy of `lhs`.
    let mut lhs_nodes: IdxVec<NodeIndex, NodeIndex> = IdxVec::default();
    for n in lhs.node_weights() {
        lhs_nodes.push(union.add_node(*n));
    }
    for e in lhs.edge_references() {
        union.add_edge(lhs_nodes[e.source()], lhs_nodes[e.target()], ());
    }

    // Copy of `rhs`.
    let mut rhs_nodes: IdxVec<NodeIndex, NodeIndex> = IdxVec::default();
    for n in rhs.node_weights() {
        rhs_nodes.push(union.add_node(*n));
    }
    for e in rhs.edge_references() {
        union.add_edge(rhs_nodes[e.source()], rhs_nodes[e.target()], ());
    }

    // Edges from `lhs` to `rhs`.
    for a in lhs.externals(EdgeDirection::Outgoing) {
        for b in rhs.externals(EdgeDirection::Incoming) {
            union.add_edge(lhs_nodes[a], rhs_nodes[b], ());
        }
    }

    union
}

fn colimit(constraints: &[ConstraintSet]) -> (ConstraintSet, FastHashMap<Point, NodeIndex>) {
    let mut colimit = ConstraintSet::default();
    let mut point_to_node = FastHashMap::<Point, NodeIndex>::default();

    for constraint in constraints {
        for p in constraint.node_weights() {
            point_to_node
                .entry(*p)
                .or_insert_with(|| colimit.add_node(*p));
        }
        for e in constraint.edge_references() {
            let s = constraint[e.source()];
            let t = constraint[e.target()];
            colimit.add_edge(point_to_node[&s], point_to_node[&t], ());
        }
    }

    (colimit, point_to_node)
}

fn layout<V, E, F, G>(
    graph: &SliceGraph<V, E>,
    dim: usize,
    coord_map: F,
    direction_map: G,
) -> Result<IdxVec<NodeIndex, Vec<f32>>, DimensionError>
where
    F: Fn(&V) -> &[SliceIndex],
    G: Fn(&E) -> (usize, Direction),
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
            let mut directions = vec![];
            for e in graph.edges_directed(n, EdgeDirection::Incoming) {
                let s = e.source();
                let rewrite: &RewriteN = (&e.weight().1).try_into()?;

                let preimage = rewrite
                    .singular_preimage(target_index)
                    .map(|source_index| node_to_constraints[s][source_index].clone())
                    .reduce(|a, b| concat(&a, &b));

                if let Some(preimage) = preimage {
                    preimages.push(preimage);
                    directions.push(direction_map(&e.weight().0));
                }
            }

            let constraint = if preimages.is_empty() {
                // If there are no preimages, we insert a singleton constraint.
                let mut singleton = ConstraintSet::default();
                singleton.add_node((n, target_index));
                singleton
            } else {
                // Otherwise, we take a colimit of the preimages.
                let (mut colimit, point_to_node) = colimit(&preimages);

                let j = directions.iter().map(|p| p.0).min().unwrap();
                colimit.orientation = Some(j);

                for (preimage, &(i, dir)) in std::iter::zip(preimages, &directions) {
                    if i == j {
                        match dir {
                            Direction::Forward => colimit
                                .ins
                                .extend(preimage.node_weights().map(|p| point_to_node[p])),
                            Direction::Backward => colimit
                                .outs
                                .extend(preimage.node_weights().map(|p| point_to_node[p])),
                        }
                    }
                }

                colimit
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
    let colimit = colimit(&maximal_constraints).0;

    // Condense the colimit.
    let condensed_colimit = petgraph::algo::condensation(colimit.graph, true);

    let (width, positions) = solve(dim, &node_to_constraints, &condensed_colimit);

    // Calculate final layout by taking averages.
    let mut layout = IdxVec::new();
    for constraints in node_to_constraints.values() {
        let singular_positions = constraints
            .iter()
            .map(|cs| {
                let min = cs
                    .node_weights()
                    .map(|n| positions[n])
                    .min_by(|x, y| x.partial_cmp(y).unwrap())
                    .unwrap();
                let max = cs
                    .node_weights()
                    .map(|n| positions[n])
                    .max_by(|x, y| x.partial_cmp(y).unwrap())
                    .unwrap();
                (min, max)
            })
            .collect_vec();
        layout.push(compute_averages(width, singular_positions));
    }

    Ok(layout)
}

fn solve(
    dim: usize,
    node_to_constraints: &IdxVec<NodeIndex, Vec<ConstraintSet>>,
    colimit: &Graph<Vec<Point>, ()>,
) -> (f32, FastHashMap<Point, f32>) {
    let mut problem = ProblemVariables::new();

    // Variables
    let mut variables: IdxVec<NodeIndex, Variable> = IdxVec::default();
    let mut point_to_variable: FastHashMap<Point, Variable> = FastHashMap::default();
    for ps in colimit.node_weights() {
        let v = problem.add(variable().min(0.0));
        variables.push(v);
        point_to_variable.extend(ps.iter().copied().zip(std::iter::repeat(v)));
    }

    // Distance constraints.
    let mut objective: Vec<Expression> = Default::default();
    let mut constraints: Vec<Constraint> = Default::default();
    for e in colimit.edge_references() {
        let x = variables[e.source()];
        let y = variables[e.target()];
        let d = problem.add(variable().min(1.0));
        objective.push(Expression::from(d));
        constraints.push((d + x - y).eq(0.0));
    }

    // Fair averaging constraints (inc. straight wires).
    for css in node_to_constraints.values() {
        for cs in css {
            if let Some(orientation) = cs.orientation {
                let ins = cs
                    .ins
                    .iter()
                    .filter_map(|n| {
                        let external = cs
                            .edges_directed(*n, EdgeDirection::Incoming)
                            .next()
                            .is_none()
                            || cs
                                .edges_directed(*n, EdgeDirection::Outgoing)
                                .next()
                                .is_none();
                        external.then(|| point_to_variable[&cs[*n]])
                    })
                    .collect_vec();

                let outs = cs
                    .outs
                    .iter()
                    .filter_map(|n| {
                        let external = cs
                            .edges_directed(*n, EdgeDirection::Incoming)
                            .next()
                            .is_none()
                            || cs
                                .edges_directed(*n, EdgeDirection::Outgoing)
                                .next()
                                .is_none();
                        external.then(|| point_to_variable[&cs[*n]])
                    })
                    .collect_vec();

                if ins.is_empty() || outs.is_empty() {
                    continue;
                }

                let n: f64 = u32::try_from(ins.len()).unwrap().into();
                let m: f64 = u32::try_from(outs.len()).unwrap().into();

                if orientation == dim - 1 {
                    // Strict constraint: avg(ins) = avg(outs)
                    let c = problem.add(variable().min(0.0));
                    constraints.push(
                        std::iter::once(c * n)
                            .chain(ins.iter().map(|&i| -1.0 * i))
                            .sum::<Expression>()
                            .eq(0.0),
                    );
                    constraints.push(
                        std::iter::once(c * m)
                            .chain(outs.iter().map(|&o| -1.0 * o))
                            .sum::<Expression>()
                            .eq(0.0),
                    );
                } else {
                    // Weak constraints: |avg(ins) - avg(outs)| <= c.
                    let c = problem.add(variable().min(0.0));
                    objective.push(Expression::from(c));
                    constraints.push(
                        std::iter::once(c * (n * m))
                            .chain(ins.iter().map(|&i| i * m))
                            .chain(outs.iter().map(|&o| o * (-n)))
                            .sum::<Expression>()
                            .geq(0.0),
                    );
                    constraints.push(
                        std::iter::once(c * (n * m))
                            .chain(ins.iter().map(|&i| i * (-m)))
                            .chain(outs.iter().map(|&o| o * n))
                            .sum::<Expression>()
                            .geq(0.0),
                    );
                }
            }
        }
    }

    let mut model = problem
        .minimise(objective.into_iter().sum::<Expression>())
        .using(default_solver);
    for c in constraints {
        model.add_constraint(c);
    }
    let solution = model.solve().unwrap();

    let mut width = 0.0;
    let mut positions: FastHashMap<Point, f32> = FastHashMap::default();

    for n in colimit.node_indices() {
        let v = variables[n];
        let position = solution.value(v) as f32;

        for p in &colimit[n] {
            positions.insert(*p, position);
        }

        width = std::cmp::max_by(width, position + 1.0, |x, y| x.partial_cmp(y).unwrap());
    }

    (width, positions)
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
