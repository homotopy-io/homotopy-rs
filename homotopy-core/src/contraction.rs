use std::{
    convert::{Into, TryInto},
    hash::Hash,
};

use homotopy_common::{declare_idx, idx::IdxVec};
use itertools::Itertools;
use petgraph::{
    adj::UnweightedList,
    algo::{
        condensation, tarjan_scc, toposort,
        tred::{dag_to_toposorted_adjacency_list, dag_transitive_reduction_closure},
    },
    graph::{DefaultIx, DiGraph, NodeIndex},
    graphmap::DiGraphMap,
    visit::{EdgeRef, IntoNodeReferences},
    EdgeDirection::{Incoming, Outgoing},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    attach::{attach, BoundaryPath},
    common::{Boundary, Height, SingularHeight},
    diagram::{Diagram, DiagramN},
    graph::{Explodable, ExplosionOutput, ExternalRewrite, InternalRewrite},
    normalization,
    rewrite::{Cone, Cospan, Rewrite, Rewrite0, RewriteN},
    signature::Signature,
    typecheck::{typecheck_cospan, TypeError},
    Direction, Generator, SliceIndex,
};

type BiasValue = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Bias {
    Higher,
    Lower,
}

impl Bias {
    #[must_use]
    pub fn flip(self) -> Self {
        match self {
            Self::Higher => Self::Lower,
            Self::Lower => Self::Higher,
        }
    }
}

#[derive(Debug, Error)]
pub enum ContractionError {
    #[error("contraction invalid")]
    Invalid,
    #[error("contraction ambiguous")]
    Ambiguous,
    #[error("contraction fails to typecheck: {0}")]
    IllTyped(#[from] TypeError),
}

impl DiagramN {
    pub fn contract<S>(
        &self,
        boundary_path: BoundaryPath,
        interior_path: &[Height],
        height: SingularHeight,
        bias: Option<Bias>,
        signature: &S,
    ) -> Result<Self, ContractionError>
    where
        S: Signature,
    {
        if boundary_path.1 >= self.dimension() {
            return Err(ContractionError::Invalid);
        }

        attach(self, boundary_path, |slice| {
            let slice = slice.try_into().map_err(|_d| ContractionError::Invalid)?;
            let contract = contract_in_path(&slice, interior_path, height, bias)?;
            let singular = slice.clone().rewrite_forward(&contract).unwrap();
            // TODO: normalization
            // let normalize = normalization::normalize_singular(&singular.into());
            let normalize = Rewrite::identity(contract.dimension());

            let cospan = match boundary_path.boundary() {
                Boundary::Source => Cospan {
                    forward: normalize,
                    backward: contract.into(),
                },
                Boundary::Target => Cospan {
                    forward: contract.into(),
                    backward: normalize,
                },
            };

            // TODO: typechecking
            // typecheck_cospan(
            //     slice.into(),
            //     cospan.clone(),
            //     boundary_path.boundary(),
            //     signature,
            // )?;

            Ok(vec![cospan])
        })
    }
}

fn contract_base(
    diagram: &DiagramN,
    height: SingularHeight,
    bias: Option<Bias>,
) -> Result<RewriteN, ContractionError> {
    use Height::{Regular, Singular};
    let slices: Vec<_> = diagram.slices().collect();
    let cospans = diagram.cospans();

    let cospan0 = cospans.get(height).ok_or(ContractionError::Invalid)?;
    let cospan1 = cospans.get(height + 1).ok_or(ContractionError::Invalid)?;

    let regular0: &Diagram = slices
        .get(usize::from(Regular(height)))
        .ok_or(ContractionError::Invalid)?;
    let singular0: &Diagram = slices
        .get(usize::from(Singular(height)))
        .ok_or(ContractionError::Invalid)?;
    let regular1: &Diagram = slices
        .get(usize::from(Regular(height + 1)))
        .ok_or(ContractionError::Invalid)?;
    let singular1: &Diagram = slices
        .get(usize::from(Singular(height + 1)))
        .ok_or(ContractionError::Invalid)?;
    let regular2: &Diagram = slices
        .get(usize::from(Regular(height + 2)))
        .ok_or(ContractionError::Invalid)?;

    let (bias0, bias1) = match bias {
        None => (0, 0),
        Some(Bias::Higher) => (1, 0),
        Some(Bias::Lower) => (0, 1),
    };

    let mut graph = DiGraph::new();
    let r0 = graph.add_node((regular0.clone(), Default::default()));
    let s0 = graph.add_node((singular0.clone(), bias0));
    let r1 = graph.add_node((regular1.clone(), Default::default()));
    let s1 = graph.add_node((singular1.clone(), bias1));
    let r2 = graph.add_node((regular2.clone(), Default::default()));
    graph.add_edge(r0, s0, cospan0.forward.clone());
    graph.add_edge(r1, s0, cospan0.backward.clone());
    graph.add_edge(r1, s1, cospan1.forward.clone());
    graph.add_edge(r2, s1, cospan1.backward.clone());
    let result = collapse(&graph)?;
    let mut regular_slices = vec![];
    let mut singular_slices = vec![];
    for (i, r) in result.legs.into_iter() {
        if i.index() % 2 == 0 {
            regular_slices.push(r);
        } else {
            singular_slices.push(r);
        }
    }

    let rewrite = RewriteN::new(
        diagram.dimension(),
        vec![Cone::new(
            height,
            vec![cospan0.clone(), cospan1.clone()],
            Cospan {
                forward: regular_slices[0].clone(),
                backward: regular_slices[2].clone(),
            },
            regular_slices,
            singular_slices,
        )],
    );

    Ok(rewrite)
}

fn contract_in_path(
    diagram: &DiagramN,
    path: &[Height],
    height: SingularHeight,
    bias: Option<Bias>,
) -> Result<RewriteN, ContractionError> {
    match path.split_first() {
        None => contract_base(diagram, height, bias),
        Some((step, rest)) => {
            let slice: DiagramN = diagram
                .slice(*step)
                .ok_or(ContractionError::Invalid)?
                .try_into()
                .ok()
                .ok_or(ContractionError::Invalid)?;
            let rewrite = contract_in_path(&slice, rest, height, bias)?;
            match step {
                Height::Regular(i) => Ok(RewriteN::new(
                    diagram.dimension(),
                    vec![Cone::new(
                        *i,
                        vec![],
                        Cospan {
                            forward: rewrite.clone().into(),
                            backward: rewrite.into(),
                        },
                        todo!(),
                        vec![],
                    )],
                )),
                Height::Singular(i) => {
                    let source_cospan = &diagram.cospans()[*i];
                    let rewrite = rewrite.into();
                    Ok(RewriteN::new(
                        diagram.dimension(),
                        vec![Cone::new(
                            *i,
                            vec![source_cospan.clone()],
                            Cospan {
                                forward: source_cospan.forward.compose(&rewrite).unwrap(),
                                backward: source_cospan.backward.compose(&rewrite).unwrap(),
                            },
                            todo!(),
                            vec![rewrite],
                        )],
                    ))
                }
            }
        }
    }
}

declare_idx! { struct RestrictionIx = DefaultIx; }
#[derive(Debug)]
struct Cocone {
    colimit: Diagram,
    legs: IdxVec<NodeIndex<RestrictionIx>, Rewrite>,
}

fn collapse(graph: &DiGraph<(Diagram, BiasValue), Rewrite>) -> Result<Cocone, ContractionError> {
    let dimension = graph
        .node_weights()
        .next()
        .ok_or(ContractionError::Invalid)?
        .0
        .dimension();

    for (diagram, _bias) in graph.node_weights() {
        assert_eq!(diagram.dimension(), dimension);
    }

    for rewrite in graph.edge_weights() {
        assert_eq!(rewrite.dimension(), dimension);
    }

    if dimension == 0 {
        collapse_base(graph)
    } else {
        collapse_recursive(graph)
    }
}

fn collapse_base(
    graph: &DiGraph<(Diagram, BiasValue), Rewrite>,
) -> Result<Cocone, ContractionError> {
    let mut zero_graph: DiGraph<(NodeIndex, Generator), Option<Rewrite0>> = graph.map(
        |i, (d, _bias)| (i, d.clone().try_into().unwrap()),
        |_, e| Some(e.clone().try_into().unwrap()),
    );

    // find collapsible edges
    let mut backedges: Vec<_> = Default::default();
    for (s, t) in zero_graph.edge_references().filter_map(|e| {
        e.weight()
            .as_ref()
            .map(|r| r.is_identity().then(|| (e.source(), e.target())))
            .flatten()
    }) {
        if (zero_graph.edges_directed(s, Incoming).all(|p| {
            if let Some(c) = zero_graph.find_edge(p.source(), t) {
                p.weight().as_ref().map(|r| r.label())
                    == zero_graph
                        .edge_weight(c)
                        .unwrap()
                        .as_ref()
                        .map(|r| r.label())
            } else {
                true
            }
        })) && (zero_graph.edges_directed(t, Outgoing).all(|n| {
            if let Some(c) = zero_graph.find_edge(s, n.target()) {
                n.weight().as_ref().map(|r| r.label())
                    == zero_graph
                        .edge_weight(c)
                        .unwrap()
                        .as_ref()
                        .map(|r| r.label())
            } else {
                true
            }
        })) {
            // (s, t) is collapsible
            backedges.push((t, s));
        }
    }
    for (t, s) in backedges {
        zero_graph.add_edge(t, s, None);
    }

    declare_idx! { struct QuotientIx = DefaultIx; }
    let (mut quotient, node_to_scc): (DiGraph<(Vec<_>, Generator), Rewrite0, QuotientIx>, _) = {
        // compute the quotient graph with respect to strongly-connected components
        let sccs = tarjan_scc(&zero_graph);
        let mut quotient: DiGraph<(Vec<_>, Generator), Rewrite0, QuotientIx> =
            DiGraph::with_capacity(sccs.len(), zero_graph.edge_count());

        let mut node_to_scc =
            IdxVec::splat(<NodeIndex<QuotientIx>>::end(), zero_graph.node_count());
        for scc in sccs {
            let first = scc[0];
            if scc.iter().map(|&i| zero_graph[i].1).all_equal() {
                let s = quotient.add_node((Default::default(), zero_graph[first].1));
                for node in scc {
                    node_to_scc[node] = s;
                }
            } else {
                return Err(ContractionError::Invalid);
            }
        }

        let (nodes, edges) = zero_graph.into_nodes_edges();
        for (i, n) in nodes.into_iter().enumerate() {
            quotient[node_to_scc[NodeIndex::new(i)]].0.push(n.weight);
        }
        for e in edges.into_iter().filter(|e| e.weight.is_some()) {
            let source = node_to_scc[e.source()];
            let target = node_to_scc[e.target()];
            let r = e.weight.unwrap();
            if source != target {
                if let Some(i) = quotient.find_edge(source, target) {
                    if quotient.edge_weight(i).unwrap() != &r {
                        return Err(ContractionError::Invalid);
                    }
                } else {
                    quotient.add_edge(source, target, r);
                }
            }
        }

        (quotient, node_to_scc)
    };

    let (max_dim_index, (_, max_dim_diagram)) = quotient
        .node_references()
        .max_by_key(|&(_, (_, g))| g.dimension)
        .ok_or(ContractionError::Invalid)?;

    let colimit = Diagram::Diagram0(*max_dim_diagram);

    // unify all equivalence classes of maximal dimension
    let mut other_max: Vec<_> = Default::default();
    for (i, &(_, g)) in quotient
        .node_references()
        .filter(|&(i, (_, g))| i != max_dim_index && g.dimension == max_dim_diagram.dimension)
    {
        if quotient[i].1 != g {
            // found distinct generators of maximal dimension
            return Err(ContractionError::Invalid);
        }
        let mut incoming: Vec<_> = Default::default();
        let mut outgoing: Vec<_> = Default::default();
        for e in quotient.edges_directed(i, Incoming) {
            if let Some(existing) = quotient.find_edge(e.source(), max_dim_index) {
                if quotient.edge_weight(existing).unwrap() != e.weight() {
                    return Err(ContractionError::Invalid);
                }
            } else {
                incoming.push(e.id());
            }
        }
        for e in quotient.edges_directed(i, Outgoing) {
            if let Some(existing) = quotient.find_edge(max_dim_index, e.target()) {
                if quotient.edge_weight(existing).unwrap() != e.weight() {
                    return Err(ContractionError::Invalid);
                }
            } else {
                outgoing.push(e.id());
            }
        }
        other_max.push((i, incoming, outgoing));
    }
    for (n, incoming, outgoing) in other_max {
        // redirect edges and remove node from quotient graph
        for e in incoming {
            let (source, _) = quotient.edge_endpoints(e).unwrap();
            let w = quotient.remove_edge(e).unwrap();
            quotient.add_edge(source, max_dim_index, w);
        }
        for e in outgoing {
            let (_, target) = quotient.edge_endpoints(e).unwrap();
            let w = quotient.remove_edge(e).unwrap();
            quotient.add_edge(max_dim_index, target, w);
        }
        quotient.remove_node(n);
    }

    // construct colimit legs
    let mut legs = IdxVec::with_capacity(graph.node_count());
    for i in graph.node_indices() {
        let leg = if node_to_scc[i] == max_dim_index {
            Rewrite::identity(0)
        } else {
            quotient
                .edge_weight(
                    quotient
                        .find_edge(node_to_scc[i], max_dim_index)
                        .ok_or(ContractionError::Invalid)?,
                )
                .unwrap()
                .clone()
                .into()
        };
        legs.push(leg);
    }

    Ok(Cocone { colimit, legs })
}

fn collapse_recursive(
    graph: &DiGraph<(Diagram, BiasValue), Rewrite>,
) -> Result<Cocone, ContractionError> {
    // Input: graph of n-diagrams and n-rewrites

    // marker for edges in Δ
    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum DeltaSlice {
        Internal(SingularHeight, Direction),
        SingularSlice,
    }

    // in the exploded graph, each singular node is tagged with its parent's NodeIndex, and height
    //                        each singular slice is tagged with its parent's EdgeIndex
    declare_idx! { struct ExplodedIx = DefaultIx; }
    let ExplosionOutput {
        output: exploded,
        node_to_nodes: node_to_slices,
        ..
    }: ExplosionOutput<_, _, _, ExplodedIx> = graph
        .map(|_, (d, bias)| (bias, d.clone()), |_, e| ((), e.clone()))
        .explode(
            |parent_node, _bias, si| match si {
                SliceIndex::Boundary(_) => None,
                SliceIndex::Interior(h) => Some((parent_node, h)),
            },
            |_parent_node, _bias, internal| match internal {
                InternalRewrite::Boundary(_) => None,
                InternalRewrite::Interior(i, dir) => Some(Some(DeltaSlice::Internal(i, dir))),
            },
            |parent_edge, _, external| match external {
                ExternalRewrite::SingularSlice(_) | ExternalRewrite::Sparse(_) => {
                    Some(Some(DeltaSlice::SingularSlice))
                }
                _ => Some(None),
            },
        )
        .map_err(|_| ContractionError::Invalid)?;

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
        for (&s, &snext) in node_to_slices[singular]
            .iter()
            .filter(|&i| matches!(exploded[*i], ((_, Height::Singular(_)), _)))
            .tuple_windows::<(_, _)>()
        {
            // uni-directional edges between singular heights originating from the same diagram
            delta.add_edge(s, snext, ());
        }
    }

    // construct each morphism of the Δ diagram
    for r in exploded
        .edge_references()
        .filter(|e| matches!(e.weight().0, Some(DeltaSlice::SingularSlice)))
    {
        for s in exploded
            .edges_directed(r.source(), Outgoing)
            .filter(|e| e.id() > r.id() && matches!(e.weight().0, Some(DeltaSlice::SingularSlice)))
        {
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
    let scc_to_priority: IdxVec<NodeIndex<QuotientIx>, (usize, BiasValue)> = {
        let mut scc_to_priority: IdxVec<NodeIndex<QuotientIx>, (usize, BiasValue)> =
            IdxVec::splat(Default::default(), quotient.node_count());
        for (i, scc) in quotient.node_references().rev() {
            let priority = quotient
                .neighbors_directed(i, Incoming)
                .map(|prev| scc_to_priority[prev].0 + 1) // defined because SCCs are already topologically sorted
                .fold(usize::MIN, std::cmp::max);
            let bias = scc
                .iter()
                .map(|&n| graph[exploded[n].0 .0].1)
                .fold(BiasValue::MAX, std::cmp::min);
            scc_to_priority[i] = (priority, bias);
        }
        scc_to_priority
    };
    let linear_components: Vec<_> = {
        let mut components: Vec<_> = quotient.node_references().collect();
        components.sort_by_key(|(i, _)| scc_to_priority[*i]);
        is_strictly_increasing(&components, |(i, _)| scc_to_priority[*i])
            .then(|| components)
            .ok_or(ContractionError::Ambiguous)
    }?;

    // solve recursive subproblems
    let (topo, revmap): (
        UnweightedList<NodeIndex<ExplodedIx>>,
        Vec<NodeIndex<ExplodedIx>>,
    ) = dag_to_toposorted_adjacency_list(&exploded, &toposort(&exploded, None).unwrap());
    let (_, closure) = dag_transitive_reduction_closure(&topo);
    let cocones: Vec<(
        NodeIndex<RestrictionIx>,
        Cocone,
        NodeIndex<RestrictionIx>,
        IdxVec<NodeIndex<RestrictionIx>, NodeIndex<ExplodedIx>>,
    )> = linear_components
        .into_iter()
        .map(|(_, scc)| -> Result<_, ContractionError> {
            // construct subproblem for each SCC
            // the subproblem for each SCC is the subgraph of the exploded graph containing the SCC
            // closed under reverse-reachability
            let mut restriction_to_exploded = IdxVec::new();
            let restriction: DiGraph<(Diagram, BiasValue), _, RestrictionIx> = exploded.filter_map(
                |i, (_, diagram)| {
                    scc.iter()
                        .any(|&c| {
                            i == c || closure.contains_edge(revmap[i.index()], revmap[c.index()])
                        })
                        .then(|| {
                            restriction_to_exploded.push(i);
                            (diagram.clone(), graph[exploded[i].0 .0].1)
                        })
                },
                |_, (ds, rewrite)| Some((ds, rewrite.clone())),
            );
            // note: every SCC spans every input diagram, and all sources (resp. targets) of
            // subdiagrams within an SCC are equal by globularity

            let source = restriction
                .edge_references()
                .sorted_by_key(|e| e.weight().0)
                .find_map(|e| {
                    matches!(
                        e.weight().0,
                        Some(DeltaSlice::Internal(_, Direction::Forward))
                    )
                    .then(|| e.source())
                })
                .ok_or(ContractionError::Invalid)?;
            let target = restriction
                .edge_references()
                .sorted_by_key(|e| e.weight().0)
                .rev() // TODO: need label scheme which identifies certain labels with differing origin coordinates
                .find_map(|e| {
                    matches!(
                        e.weight().0,
                        Some(DeltaSlice::Internal(_, Direction::Backward))
                    )
                    .then(|| e.source())
                })
                .ok_or(ContractionError::Invalid)?;
            // throw away extra information used to compute source and target
            let restriction =
                restriction.filter_map(|_, d| d.clone().into(), |_, (_, r)| r.clone().into());
            let cocone = collapse(&restriction)?;
            Ok((source, cocone, target, restriction_to_exploded))
        })
        .fold_ok(vec![], |mut acc, x| {
            acc.push(x);
            acc
        })?;

    // assemble solutions
    let first = cocones.first().ok_or(ContractionError::Invalid)?;
    let colimit: DiagramN = DiagramN::new(
        first
            .1
            .colimit
            .clone()
            .rewrite_backward(&first.1.legs[first.0])
            .map_err(|_| ContractionError::Invalid)?,
        cocones
            .iter()
            .map(|(source, cocone, target, _)| Cospan {
                forward: cocone.legs[*source].clone(),
                backward: cocone.legs[*target].clone(),
            })
            .collect(),
    );

    let dimension = colimit.dimension();
    let legs = node_to_slices
        .into_iter()
        .map(|(n, _)| {
            let mut regular_slices: Vec<Vec<_>> = Default::default();
            let mut singular_slices: Vec<Vec<_>> = Default::default();
            for (_, cocone, _, restriction_to_exploded) in &cocones {
                let subrewrites = cocone.legs.iter().filter(|&(i, _)| {
                    let ((parent, _), _) = exploded[restriction_to_exploded[i]];
                    parent == n
                });
                let mut rs: Vec<_> = Default::default();
                let mut ss: Vec<_> = Default::default();
                for (i, r) in subrewrites {
                    match &exploded[restriction_to_exploded[i]] {
                        ((_, Height::Regular(_)), _) => rs.push(r.clone()),
                        ((_, Height::Singular(_)), _) => ss.push(r.clone()),
                    }
                }
                regular_slices.push(rs);
                singular_slices.push(ss);
            }
            Some(
                RewriteN::from_slices(
                    dimension,
                    <&DiagramN>::try_from(&graph[n].0).ok()?.cospans(),
                    colimit.cospans(),
                    regular_slices,
                    singular_slices,
                )
                .into(),
            )
        })
        .collect::<Option<Vec<_>>>()
        .ok_or(ContractionError::Invalid)?
        .into_iter()
        .collect();

    Ok(Cocone {
        colimit: colimit.into(),
        legs,
    })
}

fn is_strictly_increasing<T, K, F>(slice: &[T], key: F) -> bool
where
    K: Ord,
    F: Fn(&T) -> K,
{
    for i in 1..slice.len() {
        if key(&slice[i - 1]) >= key(&slice[i]) {
            return false;
        }
    }
    true
}
