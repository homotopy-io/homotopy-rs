use std::{
    cmp::Ordering,
    convert::{Into, TryInto},
    hash::Hash,
};

use homotopy_common::{declare_idx, hash::FastHashMap, idx::IdxVec};
use itertools::Itertools;
use petgraph::{
    adj::UnweightedList,
    algo::{
        condensation, toposort,
        tred::{dag_to_toposorted_adjacency_list, dag_transitive_reduction_closure},
    },
    graph::{DefaultIx, DiGraph, IndexType, NodeIndex},
    graphmap::DiGraphMap,
    visit::{EdgeRef, IntoNodeReferences},
    EdgeDirection::{Incoming, Outgoing},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    attach::attach,
    collapse::{unify, Cartesian, Collapsible},
    common::{Boundary, BoundaryPath, DimensionError, Height, Orientation, SingularHeight},
    diagram::{Diagram, Diagram0, DiagramN},
    expansion::expand_propagate,
    rewrite::{Cone, Cospan, Rewrite, Rewrite0, RewriteN},
    scaffold::{
        Explodable, ExplosionOutput, ExternalRewrite, InternalRewrite, Scaffold, ScaffoldEdge,
        ScaffoldNode,
    },
    signature::Signature,
    typecheck::{typecheck_cospan, TypeError},
    Direction, SliceIndex,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Bias {
    Higher,
    Same,
    Lower,
}

impl Bias {
    #[must_use]
    pub fn flip(self) -> Self {
        match self {
            Self::Higher => Self::Lower,
            Self::Same => Self::Same,
            Self::Lower => Self::Higher,
        }
    }
}

#[derive(Debug, Error)]
pub enum ContractionError {
    #[error("contraction failed: label inconsistency")]
    LabelInconsistency,

    #[error("contraction failed: max dimensional generator not connected to all others")]
    NonConnectedMaxDimensionGenerator,

    #[error("contraction failed: max dimensional generator not unique")]
    NonUniqueMaxDimensionGenerator,

    #[error("contraction failed: orientation error")]
    Orientation,

    #[error("contraction invalid")]
    Invalid,

    #[error("contraction ambiguous")]
    Ambiguous,

    #[error("contraction is ill-typed")]
    IllTyped(#[from] TypeError),

    #[error(transparent)]
    Dimension(#[from] DimensionError),

    #[error("location is outside of the diagram")]
    OutOfBounds,
}

struct ContractExpand {
    contract: RewriteN,
    expand: RewriteN,
}

impl DiagramN {
    #[allow(clippy::too_many_arguments)]
    pub fn contract(
        &self,
        boundary_path: BoundaryPath,
        interior_path: &mut [Height],
        height: SingularHeight,
        direction: Direction,
        step: usize,
        bias: Option<Bias>,
        signature: &impl Signature,
    ) -> Result<Self, ContractionError> {
        attach(self, boundary_path, |slice| {
            let slice = slice.try_into()?;
            let ContractExpand { contract, expand } =
                contract_in_path(&slice, interior_path, height, direction, step, bias, true)?;

            let cospan = Cospan {
                forward: contract.into(),
                backward: expand.into(),
            };

            typecheck_cospan(slice.into(), cospan.clone(), signature)?;

            let cospan = match boundary_path.boundary() {
                Boundary::Source => cospan.flip(),
                Boundary::Target => cospan,
            };

            Ok(vec![cospan])
        })
    }
}

fn contract_base(
    diagram: &DiagramN,
    height: SingularHeight,
    direction: Direction,
    step: usize,
    bias: Option<Bias>,
    cone_wise_smooth: bool,
) -> Result<ContractExpand, ContractionError> {
    use Height::{Regular, Singular};

    assert!(
        step == 1 || bias.is_none(),
        "Biased multicontraction is not supported"
    );

    if height >= diagram.size()
        || (height < step && direction == Direction::Backward)
        || (height + step >= diagram.size() && direction == Direction::Forward)
    {
        return Err(ContractionError::OutOfBounds);
    }

    let (i, bias) = match direction {
        Direction::Forward => (height, bias),
        Direction::Backward => (height - step, bias.map(Bias::flip)),
    };

    let cospans = &diagram.cospans()[i..=i + step];

    let mut graph = DiGraph::new();
    let nodes: IdxVec<Height, NodeIndex> = diagram
        .slices()
        .skip(2 * i)
        .take(2 * step + 3)
        .enumerate()
        .map(|(i, slice)| {
            let height = Height::from(i);
            let bias = match (step, height) {
                (1, Singular(0)) => bias.map(Bias::flip),
                (1, Singular(1)) => bias,
                _ => None,
            };
            graph.add_node(ScaffoldNode {
                key: ContractNode {
                    bias,
                    coordinate: vec![height],
                },
                diagram: slice,
            })
        })
        .collect();
    for (i, cs) in cospans.iter().enumerate() {
        graph.add_edge(
            nodes[Regular(i)],
            nodes[Singular(i)],
            cs.forward.clone().into(),
        );
        graph.add_edge(
            nodes[Regular(i + 1)],
            nodes[Singular(i)],
            cs.backward.clone().into(),
        );
    }

    let result = colimit(&graph)?;

    let cospan = Cospan {
        forward: result.legs[nodes[Regular(0)]].clone(),
        backward: result.legs[nodes[Regular(step + 1)]].clone(),
    };

    let contract = RewriteN::new(
        diagram.dimension(),
        vec![Cone::new(
            i,
            cospans.to_vec(),
            cospan.clone(),
            (0..=step + 1)
                .map(|i| result.legs[nodes[Regular(i)]].clone())
                .collect(),
            (0..=step)
                .map(|i| result.legs[nodes[Singular(i)]].clone())
                .collect(),
        )],
    );

    let expand = {
        if cospan.is_redundant() {
            // Coarse smoothing
            // A cospan is smoothable if the forward and backward rewrites are identical and redundant.
            RewriteN::new(
                diagram.dimension(),
                vec![Cone::new_unit(i, cospan.clone(), cospan.forward)],
            )
        } else {
            match (cone_wise_smooth, result.colimit) {
                (true, Diagram::DiagramN(colimit)) => {
                    // Cone-wise smoothing
                    // A pair of cones over the same target height is smoothable if they are identical (modulo different indices) and redundant.
                    let forward: &RewriteN = (&cospan.forward).try_into().unwrap();
                    let backward: &RewriteN = (&cospan.backward).try_into().unwrap();

                    let mut s_cones = vec![];
                    let mut f_cones = vec![];
                    let mut b_cones = vec![];
                    for height in 0..colimit.size() {
                        match (
                            forward.cone_over_target(height).left(),
                            backward.cone_over_target(height).left(),
                        ) {
                            (None, None) => {}
                            (None, Some(b_cone)) => b_cones.push(b_cone.clone()),
                            (Some(f_cone), None) => f_cones.push(f_cone.clone()),
                            (Some(f_cone), Some(b_cone)) => {
                                if f_cone.internal == b_cone.internal && f_cone.is_redundant() {
                                    s_cones.push(Cone {
                                        index: height,
                                        internal: f_cone.internal.clone(),
                                    });
                                } else {
                                    f_cones.push(f_cone.clone());
                                    b_cones.push(b_cone.clone());
                                }
                            }
                        }
                    }

                    let smooth = RewriteN::new(colimit.dimension(), s_cones).into();
                    let smooth_cospan = Cospan {
                        forward: RewriteN::new(colimit.dimension(), f_cones).into(),
                        backward: RewriteN::new(colimit.dimension(), b_cones).into(),
                    };

                    let cone = if smooth_cospan.is_identity() {
                        // Decrease diagram height by 1.
                        Cone::new_unit(i, cospan, smooth)
                    } else {
                        // Keep diagram height the same.
                        Cone::new(
                            i,
                            vec![smooth_cospan],
                            cospan.clone(),
                            vec![cospan.forward, cospan.backward],
                            vec![smooth],
                        )
                    };

                    RewriteN::new(diagram.dimension(), vec![cone])
                }
                _ => RewriteN::identity(diagram.dimension()),
            }
        }
    };

    Ok(ContractExpand { contract, expand })
}

fn contract_in_path(
    diagram: &DiagramN,
    path: &mut [Height],
    height: SingularHeight,
    direction: Direction,
    step: usize,
    bias: Option<Bias>,
    cone_wise_smooth: bool,
) -> Result<ContractExpand, ContractionError> {
    use Height::{Regular, Singular};

    match path.split_first_mut() {
        None => contract_base(diagram, height, direction, step, bias, cone_wise_smooth),
        Some((first, rest)) => {
            let slice: DiagramN = diagram
                .slice(*first)
                .ok_or(ContractionError::OutOfBounds)?
                .try_into()?;
            let ContractExpand {
                contract: contract_base,
                expand: expand_base,
            } = contract_in_path(&slice, rest, height, direction, step, bias, false)?;
            match *first {
                Regular(i) => {
                    *first = Singular(i);
                    let contract = RewriteN::new(
                        diagram.dimension(),
                        vec![Cone::new_unit(
                            i,
                            Cospan {
                                forward: contract_base.clone().into(),
                                backward: contract_base.clone().into(),
                            },
                            contract_base.into(),
                        )],
                    );
                    let expand = expand_propagate(
                        &diagram.clone().rewrite_forward(&contract).unwrap(),
                        first,
                        expand_base.into(),
                        false,
                        false,
                    )
                    .map_err(|_err| ContractionError::Invalid)?;
                    Ok(ContractExpand { contract, expand })
                }
                Singular(i) => {
                    let source_cospan = &diagram.cospans()[i];
                    let slice: Diagram = slice.into();
                    let contract_base = contract_base.into();
                    let (forward, backward) = {
                        // compose by colimit
                        let mut graph = DiGraph::new();
                        let r_p = graph.add_node(ScaffoldNode {
                            key: ContractNode {
                                bias: None,
                                coordinate: vec![Height::Regular(0), Height::Regular(i)],
                            },
                            diagram: slice
                                .clone()
                                .rewrite_backward(&source_cospan.forward)
                                .unwrap(),
                        });
                        let s = graph.add_node(ScaffoldNode {
                            key: ContractNode {
                                bias: None,
                                coordinate: vec![Height::Regular(0), Height::Singular(i)],
                            },
                            diagram: slice.clone(),
                        });
                        graph.add_edge(r_p, s, source_cospan.forward.clone().into());
                        let r_n = graph.add_node(ScaffoldNode {
                            key: ContractNode {
                                bias: None,
                                coordinate: vec![Height::Regular(0), Height::Regular(i + 1)],
                            },
                            diagram: slice
                                .clone()
                                .rewrite_backward(&source_cospan.backward)
                                .unwrap(),
                        });
                        graph.add_edge(r_n, s, source_cospan.backward.clone().into());
                        let c = graph.add_node(ScaffoldNode {
                            key: ContractNode {
                                bias: None,
                                coordinate: vec![Height::Singular(0), Height::Singular(i)],
                            },
                            diagram: slice.rewrite_forward(&contract_base).unwrap(),
                        });
                        graph.add_edge(s, c, contract_base.clone().into());
                        let cocone = colimit(&graph)?;
                        (cocone.legs[r_p].clone(), cocone.legs[r_n].clone())
                    };
                    let contract = RewriteN::new(
                        diagram.dimension(),
                        vec![Cone::new(
                            i,
                            vec![source_cospan.clone()],
                            Cospan {
                                forward: forward.clone(),
                                backward: backward.clone(),
                            },
                            vec![forward, backward],
                            vec![contract_base],
                        )],
                    );
                    let expand = expand_propagate(
                        &diagram.clone().rewrite_forward(&contract).unwrap(),
                        first,
                        expand_base.into(),
                        true,
                        false,
                    )
                    .map_err(|_err| ContractionError::Invalid)?;
                    Ok(ContractExpand { contract, expand })
                }
            }
        }
    }
}

#[derive(Debug)]
struct Cocone<Ix = DefaultIx>
where
    Ix: IndexType,
{
    colimit: Diagram,
    legs: IdxVec<NodeIndex<Ix>, Rewrite>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct ContractNode {
    bias: Option<Bias>,
    coordinate: Vec<Height>,
}

impl Cartesian<Height> for ContractNode {
    fn coordinate(&self) -> &[Height] {
        self.coordinate.as_slice()
    }
}

type ContractGraph<Ix> = Scaffold<ContractNode, (), Ix>;

fn colimit<Ix: IndexType>(graph: &ContractGraph<Ix>) -> Result<Cocone<Ix>, ContractionError> {
    let dimension = graph
        .node_weights()
        .next()
        .ok_or(ContractionError::Invalid)?
        .diagram
        .dimension();

    for ScaffoldNode { diagram, .. } in graph.node_weights() {
        assert_eq!(diagram.dimension(), dimension);
    }

    for ScaffoldEdge { rewrite, .. } in graph.edge_weights() {
        assert_eq!(rewrite.dimension(), dimension);
    }

    if dimension == 0 {
        colimit_base(graph)
    } else {
        colimit_recursive(graph)
    }
}

fn colimit_base<Ix: IndexType>(graph: &ContractGraph<Ix>) -> Result<Cocone<Ix>, ContractionError> {
    let (mut stable, mut union_find) = graph.collapse();

    // unify all nodes of maximal dimension
    let (max_dim_index, max_dim_generator) = stable
        .node_references()
        .map(|(i, ScaffoldNode { diagram, .. })| {
            let d: Diagram0 = diagram.try_into().unwrap();
            (i, d.generator)
        })
        .max_by_key(|(_, g)| g.dimension)
        .ok_or(ContractionError::NonUniqueMaxDimensionGenerator)?;

    let codimension = graph[max_dim_index]
        .key
        .coordinate
        .len()
        .saturating_sub(max_dim_generator.dimension);

    // Collect the orientations of the maximum-dimensional generator by subslice.
    let mut orientations = FastHashMap::<&[Height], Vec<Orientation>>::default();

    let mut max_dims: Vec<_> = Default::default();
    for (
        i,
        ScaffoldNode {
            key: ContractNode { coordinate, .. },
            diagram,
        },
    ) in graph.node_references()
    {
        let d: Diagram0 = diagram.try_into().unwrap();
        if d.generator.dimension == max_dim_generator.dimension {
            if d.generator != max_dim_generator {
                // found distinct elements of maximal dimension
                return Err(ContractionError::NonUniqueMaxDimensionGenerator);
            }
            if stable.contains_node(i) {
                max_dims.push(i);
            };

            orientations
                .entry(&coordinate[..codimension])
                .or_default()
                .push(d.orientation);
        }
    }

    let orientation = {
        // Check that the orientations in each subslice cancel out.
        let slice_orientations = orientations
            .into_values()
            .map(|orientations| {
                let counts = orientations.into_iter().counts();
                let pos = counts
                    .get(&Orientation::Positive)
                    .copied()
                    .unwrap_or_default();
                let neg = counts
                    .get(&Orientation::Negative)
                    .copied()
                    .unwrap_or_default();

                match pos.cmp(&neg) {
                    Ordering::Less => (neg == pos + 1).then_some(Orientation::Negative),
                    Ordering::Equal => Some(Orientation::Zero),
                    Ordering::Greater => (pos == neg + 1).then_some(Orientation::Positive),
                }
            })
            .collect::<Option<Vec<_>>>()
            .ok_or(ContractionError::Orientation)?;

        // Check that all subslices yield the same orientation.
        for x in &slice_orientations[1..] {
            if *x != slice_orientations[0] {
                return Err(ContractionError::Orientation);
            }
        }
        slice_orientations[0]
    };

    for i in max_dims {
        unify(
            &mut stable,
            i,
            max_dim_index,
            &mut union_find,
            |_rn| (),
            |_re| (),
        );
    }

    let colimit = Diagram0::new(max_dim_generator, orientation);

    // construct colimit legs
    let legs = {
        let mut legs = IdxVec::with_capacity(graph.node_count());
        for (n, ScaffoldNode { diagram, .. }) in graph.node_references() {
            let d: Diagram0 = diagram.try_into().unwrap();
            let r = {
                let (p, q) = (union_find.find_mut(n), union_find.find_mut(max_dim_index));
                if p == q {
                    debug_assert_eq!(d.generator, colimit.generator);
                    Rewrite0::new(d, colimit, None)
                } else {
                    // Collect the labels of all edges between p and q.
                    let mut labels = stable.edges_connecting(p, q).map(|e| {
                        <&Rewrite0>::try_from(&e.weight().rewrite)
                            .expect("non 0-rewrite passed to colimit_base")
                            .label()
                    });
                    let label = labels
                        .next()
                        .ok_or(ContractionError::NonConnectedMaxDimensionGenerator)?;
                    if labels.any(|l| l != label) {
                        return Err(ContractionError::LabelInconsistency);
                    }
                    Rewrite0::new(d, colimit, label.cloned())
                }
            };
            legs.push(r.into());
        }
        legs
    };

    let cocone = Cocone {
        colimit: colimit.into(),
        legs,
    };
    Ok(cocone)
}

fn colimit_recursive<Ix: IndexType>(
    graph: &ContractGraph<Ix>,
) -> Result<Cocone<Ix>, ContractionError> {
    // Input: graph of n-diagrams and n-rewrites

    // marker for edges in Δ
    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum DeltaSlice {
        // within singular heights from the same diagram
        Internal(SingularHeight, Direction),
        // between singular heights of different diagrams
        SingularSlice,
    }

    // in the exploded graph, each singular node is tagged with its parent's NodeIndex, and height
    //                        each singular slice is tagged with its parent's EdgeIndex
    struct ExplodedNode<Ix> {
        parent: NodeIndex<Ix>,
        height: Height,
        coordinate: Vec<Height>,
    }
    declare_idx! { struct ExplodedIx = DefaultIx; }
    let ExplosionOutput {
        scaffold: exploded,
        node_to_nodes: node_to_slices,
        ..
    }: ExplosionOutput<_, Scaffold<_, _, ExplodedIx>> = graph
        .explode(
            |parent, ContractNode { coordinate, .. }, si| match si {
                SliceIndex::Boundary(_) => None,
                SliceIndex::Interior(height) => {
                    let mut coordinate = coordinate.clone();
                    coordinate.push(height);
                    Some(ExplodedNode {
                        parent,
                        height,
                        coordinate,
                    })
                }
            },
            |_parent, _bias, internal| match internal {
                InternalRewrite::Boundary(_) => None,
                InternalRewrite::Interior(i, dir) => Some(Some(DeltaSlice::Internal(i, dir))),
            },
            |_parent, (), external| match external {
                ExternalRewrite::SingularSlice { .. } | ExternalRewrite::Sparse(_) => {
                    Some(Some(DeltaSlice::SingularSlice))
                }
                _ => Some(None),
            },
        )
        .map_err(|_err| ContractionError::Invalid)?;

    // Find colimit in Δ (determines the order of subproblem solutions as singular heights in the
    // constructed colimit)
    //
    // Δ is a subgraph of the exploded graph, comprising of information in the projection of
    // rewrites to monotone functions between singular levels, containing the singular heights of
    // nodes which themselves are singular in the unexploded graph. Each successive singular height
    // originating from the same node is connected by a uni-directional edge, and nodes in Δ which
    // are connected by a span (sliced from a span at the unexploded level) are connected by
    // bidirectional edges. This allows one to compute the colimit in Δ by strongly-connected
    // components.

    // each node of delta is keyed by the NodeIndex of exploded from where it originates
    let mut delta: DiGraphMap<NodeIndex<ExplodedIx>, ()> = Default::default();

    // construct each object of the Δ diagram
    // these should be the singular heights of the n-diagrams from the input which themselves
    // originate from singular heights (which can be determined by ensuring adjacent edges are all
    // incoming)
    for singular in graph.externals(Outgoing) {
        if node_to_slices[singular].len() == 3 {
            // only one singular level
            // R -> S <- R
            delta.add_node(node_to_slices[singular][1]);
        } else {
            // more than one singular level
            // R -> S <- ... -> S <- R
            for (&s, &snext) in node_to_slices[singular]
                .iter()
                .filter(|&i| matches!(exploded[*i].key.height, Height::Singular(_)))
                .tuple_windows::<(_, _)>()
            {
                // uni-directional edges between singular heights originating from the same diagram
                delta.add_edge(s, snext, ());
            }
        }
    }

    // construct each morphism of the Δ diagram
    for r in exploded
        .edge_references()
        .filter(|e| matches!(e.weight().key, Some(DeltaSlice::SingularSlice)))
    {
        for s in exploded.edges_directed(r.source(), Outgoing).filter(|e| {
            e.id() > r.id() && matches!(e.weight().key, Some(DeltaSlice::SingularSlice))
        }) {
            // for all slice spans between singular levels
            if delta.contains_node(r.target()) && delta.contains_node(s.target()) {
                // bidirectional edge
                delta.add_edge(r.target(), s.target(), ());
                delta.add_edge(s.target(), r.target(), ());
            }
        }
    }

    // find the colimit of the Δ diagram by computing the quotient graph under strongly-connected
    // components and linearizing
    declare_idx! { struct QuotientIx = DefaultIx; }
    let quotient: DiGraph<_, _, QuotientIx> = condensation(delta.into_graph(), true);

    // linearize the quotient graph
    //  * each node in the quotient graph is a singular height in the colimit
    //  * the monotone function on singular heights is determined by the inclusion of Δ into the
    //    quotient graph
    let scc_to_priority: IdxVec<NodeIndex<QuotientIx>, (usize, Option<Bias>)> = {
        let mut scc_to_priority: IdxVec<NodeIndex<QuotientIx>, (usize, Option<Bias>)> =
            IdxVec::splat(Default::default(), quotient.node_count());
        for (i, scc) in quotient.node_references().rev() {
            let priority = quotient
                .neighbors_directed(i, Incoming)
                .map(|prev| scc_to_priority[prev].0 + 1) // defined because SCCs are already topologically sorted
                .max()
                .unwrap_or_default();
            let bias = scc
                .iter()
                .map(|&n| graph[exploded[n].key.parent].key.bias)
                .all_equal_value()
                .ok()
                .flatten();
            scc_to_priority[i] = (priority, bias);
        }
        scc_to_priority
    };
    // linear_components is the inverse image of the singular monotone
    let linear_components: Vec<_> = {
        scc_to_priority
            .values()
            .sorted_unstable()
            .tuple_windows()
            .all(|((p, x), (q, y))| !(p == q && (x.is_none() || y.is_none())))
            .then(|| {
                let mut components: Vec<_> = quotient.node_references().collect();
                components.sort_by_key(|(i, _)| scc_to_priority[*i]);
                components
                    .into_iter()
                    .group_by(|(i, _)| scc_to_priority[*i])
                    .into_iter()
                    .map(|(_, sccs)| {
                        sccs.map(|(_, scc)| scc.clone())
                            .collect::<Vec<_>>()
                            .concat()
                    })
                    .collect()
            })
            .ok_or(ContractionError::Ambiguous)
    }?;

    // determine the dual monotone on regular heights
    // regular_monotone[..][j] is the jth regular monotone from the colimit
    let regular_monotone: Vec<Vec<_>> = {
        let mut regular_monotone: Vec<Vec<NodeIndex<ExplodedIx>>> =
            Vec::with_capacity(linear_components.len() + 1);
        let mut parent_by_height: Vec<NodeIndex<Ix>> = Default::default();
        // invariant: ∀ m ∈ regular_monotone. m.len() == parent_by_height.len()
        regular_monotone.push(
            // all targeting Regular(0)
            exploded
                .node_references()
                .filter(
                    |(
                        _,
                        ScaffoldNode {
                            key: ExplodedNode { parent, height, .. },
                            ..
                        },
                    )| {
                        graph.externals(Outgoing).contains(parent) // comes from singular height (i.e. in Δ)
                        && height == &Height::Regular(0)
                    },
                )
                .map(
                    |(
                        i,
                        ScaffoldNode {
                            key: ExplodedNode { parent, .. },
                            ..
                        },
                    )| {
                        parent_by_height.push(*parent);
                        i
                    },
                )
                .collect(),
        );
        for scc in &linear_components {
            // get the right-most boundary of this scc
            regular_monotone.push({
                let mut right = regular_monotone.last().unwrap().clone();
                for (p, next) in scc
                    .iter()
                    .group_by(|&i| exploded[*i].key.parent) // group by parent
                    .into_iter()
                    .map(|(p, group)| {
                        (
                            p,
                            group
                                .max() // get right-most
                                .map(
                                    |i| NodeIndex::<ExplodedIx>::new(i.index() + 1), // next regular level,
                                )
                                .expect("scc empty group in Δ"),
                        )
                    })
                {
                    right[parent_by_height.iter().position(|&x| x == p).unwrap()] = next;
                }
                right
            });
        }
        regular_monotone
    };

    // solve recursive subproblems
    let (topo, revmap): (
        UnweightedList<NodeIndex<ExplodedIx>>,
        Vec<NodeIndex<ExplodedIx>>,
    ) = dag_to_toposorted_adjacency_list(&exploded, &toposort(&exploded, None).unwrap());
    let (_, closure) = dag_transitive_reduction_closure(&topo);
    declare_idx! { struct RestrictionIx = DefaultIx; }
    #[allow(clippy::type_complexity)]
    let cocones: Vec<(
        NodeIndex<RestrictionIx>,
        Cocone<RestrictionIx>,
        NodeIndex<RestrictionIx>,
        IdxVec<NodeIndex<RestrictionIx>, NodeIndex<ExplodedIx>>,
    )> = linear_components
        .into_iter()
        .zip(regular_monotone.windows(2))
        .map(|(scc, adjacent_regulars)| -> Result<_, ContractionError> {
            // construct subproblem for each SCC
            // the subproblem for each SCC is the subgraph of the exploded graph containing the SCC
            // and its adjacent regulars closed under reverse-reachability
            let mut restriction_to_exploded = IdxVec::new();
            let restriction: Scaffold<ContractNode, _, RestrictionIx> = exploded.filter_map(
                |i,
                 ScaffoldNode {
                     key:
                         ExplodedNode {
                             parent, coordinate, ..
                         },
                     diagram,
                 }| {
                    scc.iter()
                        .chain(&adjacent_regulars[0])
                        .chain(&adjacent_regulars[1])
                        .any(|&c| {
                            i == c || closure.contains_edge(revmap[i.index()], revmap[c.index()])
                        })
                        .then(|| {
                            restriction_to_exploded.push(i);
                            ScaffoldNode {
                                key: ContractNode {
                                    bias: graph[*parent].key.bias,
                                    coordinate: coordinate.clone(),
                                },
                                diagram: diagram.clone(),
                            }
                        })
                },
                |_, ScaffoldEdge { key, rewrite }| {
                    Some(ScaffoldEdge {
                        key,
                        rewrite: rewrite.clone(),
                    })
                },
            );
            // note: every SCC spans every input diagram, and all sources (resp. targets) of
            // subdiagrams within an SCC are equal by globularity

            let max_ix = restriction
                .externals(Outgoing)
                .max_by_key(|&ix| restriction[ix].diagram.max_generator().generator.dimension)
                .expect("recursive colimit subproblem has no max dimensional subdiagram");
            // TODO(@calintat): Clean this up!
            let source_ix = {
                if let Some(mut cur) = restriction
                    .edges_directed(max_ix, Incoming)
                    .find(|e| {
                        matches!(
                            e.weight().key,
                            Some(DeltaSlice::Internal(_, Direction::Forward))
                        )
                    })
                    .map(|e| e.source())
                {
                    while let Some(prev) = restriction
                        .edges_directed(cur, Outgoing)
                        .find(|e| {
                            matches!(
                                e.weight().key,
                                Some(DeltaSlice::Internal(_, Direction::Backward))
                            )
                        })
                        .map(|e| e.target())
                    {
                        cur = restriction
                            .edges_directed(prev, Incoming)
                            .find(|e| {
                                matches!(
                                    e.weight().key,
                                    Some(DeltaSlice::Internal(_, Direction::Forward))
                                )
                            })
                            .unwrap()
                            .source();
                    }
                    cur
                } else {
                    max_ix
                }
            };
            let target_ix = {
                if let Some(mut cur) = restriction
                    .edges_directed(max_ix, Incoming)
                    .find(|e| {
                        matches!(
                            e.weight().key,
                            Some(DeltaSlice::Internal(_, Direction::Backward))
                        )
                    })
                    .map(|e| e.source())
                {
                    while let Some(prev) = restriction
                        .edges_directed(cur, Outgoing)
                        .find(|e| {
                            matches!(
                                e.weight().key,
                                Some(DeltaSlice::Internal(_, Direction::Forward))
                            )
                        })
                        .map(|e| e.target())
                    {
                        cur = restriction
                            .edges_directed(prev, Incoming)
                            .find(|e| {
                                matches!(
                                    e.weight().key,
                                    Some(DeltaSlice::Internal(_, Direction::Backward))
                                )
                            })
                            .unwrap()
                            .source();
                    }
                    cur
                } else {
                    max_ix
                }
            };
            // throw away extra information used to compute source and target
            let restriction = restriction.filter_map(
                |_,
                 ScaffoldNode {
                     key: ContractNode { bias, coordinate },
                     diagram,
                 }| {
                    ScaffoldNode {
                        key: ContractNode {
                            bias: bias.filter(|bias| *bias == Bias::Same),
                            coordinate: coordinate.clone(),
                        },
                        diagram: diagram.clone(),
                    }
                    .into()
                },
                |_, ScaffoldEdge { rewrite, .. }| Some(rewrite.clone().into()),
            );
            let cocone: Cocone<RestrictionIx> = colimit(&restriction)?;
            Ok((source_ix, cocone, target_ix, restriction_to_exploded))
        })
        .fold_ok(vec![], |mut acc, x| {
            acc.push(x);
            acc
        })?;

    // assemble solutions
    let (s, first, _, _) = cocones.first().ok_or(ContractionError::Invalid)?;
    let colimit: DiagramN = if let Ok(terminal) = graph.externals(Outgoing).exactly_one() {
        DiagramN::try_from(graph[terminal].diagram.clone()).unwrap()
    } else {
        DiagramN::new(
            first
                .colimit
                .clone()
                .rewrite_backward(&first.legs[*s])
                .map_err(|_err| ContractionError::Invalid)?,
            cocones
                .iter()
                .map(|(source, cocone, target, _)| Cospan {
                    forward: cocone.legs[*source].clone(),
                    backward: cocone.legs[*target].clone(),
                })
                .collect(),
        )
    };

    let dimension = colimit.dimension();
    let (regular_slices_by_height, singular_slices_by_height) = {
        // build (regular_slices, singular_slices) for each node in graph
        let mut regular_slices_by_height: IdxVec<NodeIndex<Ix>, Vec<Vec<Rewrite>>> =
            IdxVec::splat(Vec::with_capacity(cocones.len()), graph.node_count());
        let mut singular_slices_by_height: IdxVec<NodeIndex<Ix>, Vec<Vec<Rewrite>>> =
            IdxVec::splat(Vec::with_capacity(cocones.len()), graph.node_count());
        for (_, cocone, _, restriction_to_exploded) in cocones {
            for (graph_ix, slices) in &cocone.legs.iter().group_by(|(restriction_ix, _)| {
                // parent node in graph
                exploded[restriction_to_exploded[*restriction_ix]]
                    .key
                    .parent
            }) {
                // each rewrite that will go into legs[graph_ix] from cocone
                let mut cone_regular_slices: Vec<Rewrite> = Default::default();
                let mut cone_singular_slices: Vec<Rewrite> = Default::default();
                for (restriction_ix, slice) in slices {
                    match exploded[restriction_to_exploded[restriction_ix]].key.height {
                        Height::Regular(_) => cone_regular_slices.push(slice.clone()),
                        Height::Singular(_) => cone_singular_slices.push(slice.clone()),
                    }
                }
                regular_slices_by_height[graph_ix].push(cone_regular_slices);
                singular_slices_by_height[graph_ix].push(cone_singular_slices);
            }
        }
        (regular_slices_by_height, singular_slices_by_height)
    };
    let legs = regular_slices_by_height
        .into_raw()
        .into_iter()
        .zip(singular_slices_by_height.into_raw())
        .enumerate()
        .map(|(n, (regular_slices, singular_slices))| {
            RewriteN::from_slices(
                dimension,
                <&DiagramN>::try_from(&graph[NodeIndex::new(n)].diagram)
                    .unwrap()
                    .cospans(),
                colimit.cospans(),
                regular_slices,
                singular_slices,
            )
            .into()
        })
        .collect();

    Ok(Cocone {
        colimit: colimit.into(),
        legs,
    })
}

impl Cospan {
    pub fn is_redundant(&self) -> bool {
        self.forward == self.backward && self.forward.is_redundant()
    }
}

impl Rewrite {
    fn is_redundant(&self) -> bool {
        match self {
            Rewrite::Rewrite0(r) => r
                .target()
                .map_or(true, |t| t.orientation == Orientation::Zero),
            Rewrite::RewriteN(r) => r.cones().iter().all(Cone::is_redundant),
        }
    }
}

impl Cone {
    fn is_redundant(&self) -> bool {
        self.singular_slices()
            .iter()
            .chain(self.regular_slices().iter())
            .all(Rewrite::is_redundant)
    }
}
