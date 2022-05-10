use std::{
    convert::{Into, TryInto},
    hash::Hash,
};

use homotopy_common::{declare_idx, idx::IdxVec};
use itertools::Itertools;
use petgraph::{
    adj::UnweightedList,
    algo::{
        condensation, toposort,
        tred::{dag_to_toposorted_adjacency_list, dag_transitive_reduction_closure},
    },
    graph::{DefaultIx, DiGraph, IndexType, NodeIndex},
    graphmap::DiGraphMap,
    unionfind::UnionFind,
    visit::{EdgeRef, IntoNodeReferences},
    EdgeDirection::{Incoming, Outgoing},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    attach::{attach, BoundaryPath},
    common::{Boundary, DimensionError, Height, SingularHeight},
    diagram::{Diagram, DiagramN},
    graph::{Explodable, ExplosionOutput, ExternalRewrite, InternalRewrite},
    normalization,
    rewrite::{Cone, Cospan, Label, Rewrite, Rewrite0, RewriteN},
    signature::Signature,
    typecheck::{typecheck_cospan, TypeError},
    Direction, Generator, SliceIndex,
};

type BiasValue = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
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
    #[error("invalid boundary path provided to contraction")]
    Dimension(#[from] DimensionError),
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
            let slice = slice.try_into().or(Err(ContractionError::Invalid))?;
            let contract = contract_in_path(&slice, interior_path, height, bias)?;
            let singular = slice.clone().rewrite_forward(&contract).unwrap();
            // TODO: normalization
            // let normalize = normalization::normalize_singular(&singular.into());
            let normalize = RewriteN::new(
                contract.dimension(),
                singular
                    .cospans()
                    .iter()
                    .enumerate()
                    .filter(|(_, cs)| cs.forward == cs.backward)
                    .map(|(i, cs)| {
                        Cone::new(i, vec![], cs.clone(), vec![cs.forward.clone()], vec![])
                    })
                    .collect(),
            )
            .into();

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
    for (i, r) in result.legs {
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
struct Cocone<Ix = DefaultIx>
where
    Ix: IndexType,
{
    colimit: Diagram,
    legs: IdxVec<NodeIndex<Ix>, Rewrite>,
}

fn collapse<Ix: IndexType>(
    graph: &DiGraph<(Diagram, BiasValue), Rewrite, Ix>,
) -> Result<Cocone<Ix>, ContractionError> {
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

fn collapse_base<'a, Ix: IndexType>(
    graph: &'a DiGraph<(Diagram, BiasValue), Rewrite, Ix>,
) -> Result<Cocone<Ix>, ContractionError> {
    let mut union_find = UnionFind::new(graph.node_count());

    let label = |r: &'a Rewrite| -> Option<&'a Label> {
        let r0: &Rewrite0 = r.try_into().unwrap();
        r0.label()
    };

    // find collapsible edges
    for (s, t) in graph.edge_references().filter_map(|e| {
        let r: &Rewrite0 = e.weight().try_into().unwrap();
        r.is_identity().then(|| (e.source(), e.target()))
    }) {
        if (graph.edges_directed(s, Incoming).all(|p| {
            if let Some(c) = graph.find_edge(p.source(), t) {
                label(p.weight()) == label(graph.edge_weight(c).unwrap())
            } else {
                true
            }
        })) && (graph.edges_directed(t, Outgoing).all(|n| {
            if let Some(c) = graph.find_edge(s, n.target()) {
                label(n.weight()) == label(graph.edge_weight(c).unwrap())
            } else {
                true
            }
        })) {
            // (s, t) collapsible
            union_find.union(s, t);
        }
    }

    // unify all equivalence classes of maximal dimension
    let (max_dim_index, (max_dim_diagram, _)) = graph
        .node_references()
        .max_by_key(|&(_i, (d, _bias))| {
            let g: Generator = d.try_into().unwrap();
            g.dimension
        })
        .ok_or(ContractionError::Invalid)?;
    let max_dim_generator: Generator = max_dim_diagram
        .try_into()
        .map_err(|_err| ContractionError::Invalid)?;
    for (x, y) in graph
        .node_references()
        .filter_map(|(i, (d, _bias))| {
            let g: Generator = d.try_into().unwrap();
            (g.dimension == max_dim_generator.dimension).then(|| i)
        })
        .tuple_windows()
    {
        if graph[x].0 != graph[y].0 {
            // found distinct elements of maximal dimension
            return Err(ContractionError::Invalid);
        }
        union_find.union(x, y);
    }

    // compute quotient graph
    let mut quotient = DiGraphMap::new();
    for e in graph.edge_references() {
        let s = union_find.find_mut(e.source());
        let t = union_find.find_mut(e.target());
        if s != t {
            if let Some(old) = quotient.add_edge(s, t, e.weight()) {
                if old != e.weight() {
                    // quotient graph not well-defined
                    return Err(ContractionError::Invalid);
                }
            }
        }
    }

    // construct colimit legs
    let legs = {
        let mut legs = IdxVec::with_capacity(graph.node_count());
        for n in graph.node_indices() {
            legs.push({
                let (p, q) = (union_find.find_mut(n), union_find.find_mut(max_dim_index));
                if p == q {
                    Rewrite::identity(0)
                } else {
                    quotient
                        .edge_weight(p, q)
                        .copied()
                        .cloned()
                        .ok_or(ContractionError::Invalid)?
                }
            });
        }
        legs
    };

    let cocone = Cocone {
        colimit: max_dim_diagram.clone(),
        legs,
    };
    Ok(cocone)
}

fn collapse_recursive<Ix: IndexType>(
    graph: &DiGraph<(Diagram, BiasValue), Rewrite, Ix>,
) -> Result<Cocone<Ix>, ContractionError> {
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
            |_parent_edge, _, external| match external {
                ExternalRewrite::SingularSlice(_) | ExternalRewrite::Sparse(_) => {
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
    let mut delta_height = 0;

    // construct each object of the Δ diagram
    // these should be the singular heights of the n-diagrams from the input which themselves
    // originate from singular heights (which can be determined by ensuring adjacent edges are all
    // incoming)
    for singular in graph.externals(Outgoing) {
        delta_height += 1;
        if node_to_slices[singular].len() == 3 {
            // only one singular level
            // R -> S <- R
            delta.add_node(node_to_slices[singular][1]);
        } else {
            // more than one singular level
            // R -> S <- ... -> S <- R
            for (&s, &snext) in node_to_slices[singular]
                .iter()
                .filter(|&i| matches!(exploded[*i], ((_, Height::Singular(_)), _)))
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
    //  * each node in the quotient graph is a singular height in the colimit
    //  * the monotone function on singular heights is determined by the inclusion of Δ into the
    //    quotient graph
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
    // linear_components is the inverse image of the singular monotone
    let linear_components: Vec<_> = {
        let mut components: Vec<_> = quotient.node_references().collect();
        components.sort_by_key(|(i, _)| scc_to_priority[*i]);
        is_strictly_increasing(&components, |(i, _)| scc_to_priority[*i])
            .then(|| components.into_iter().map(|(_, scc)| scc).collect())
            .ok_or(ContractionError::Ambiguous)
    }?;

    // determine the dual monotone on regular heights
    let regular_monotone: Vec<Vec<_>> = {
        let mut regular_monotone: Vec<Vec<_>> = Vec::with_capacity(linear_components.len() + 1);
        regular_monotone.push(
            exploded
                .node_references()
                .filter_map(|(i, ((p, h), _))| {
                    (matches!(Height::from(p.index()), Height::Singular(_))
                        && *h == Height::Regular(0))
                    .then(|| i)
                })
                .collect(),
        );
        for &scc in &linear_components {
            regular_monotone.push(
                (0..delta_height)
                    .map(|h| {
                        scc.iter()
                            .filter(|&n| {
                                Height::from(exploded[*n].0 .0.index()) == Height::Singular(h)
                            })
                            .max_by_key(|&n| exploded[*n].0 .1)
                            .map_or_else(
                                || regular_monotone.last().unwrap()[h],
                                |i| NodeIndex::new(i.index() + 1),
                            )
                    })
                    .collect(),
            );
        }
        regular_monotone
    };

    // solve recursive subproblems
    let (topo, revmap): (
        UnweightedList<NodeIndex<ExplodedIx>>,
        Vec<NodeIndex<ExplodedIx>>,
    ) = dag_to_toposorted_adjacency_list(&exploded, &toposort(&exploded, None).unwrap());
    let (_, closure) = dag_transitive_reduction_closure(&topo);
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
            let restriction: DiGraph<(Diagram, BiasValue), _, RestrictionIx> = exploded.filter_map(
                |i, (_, diagram)| {
                    scc.iter()
                        .chain(adjacent_regulars[0].iter())
                        .chain(adjacent_regulars[1].iter())
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
            let cocone: Cocone<RestrictionIx> = collapse(&restriction)?;
            Ok((source, cocone, target, restriction_to_exploded))
        })
        .fold_ok(vec![], |mut acc, x| {
            acc.push(x);
            acc
        })?;

    // assemble solutions
    let (s, first, _, _) = cocones.first().ok_or(ContractionError::Invalid)?;
    let colimit: DiagramN = DiagramN::new(
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
    );

    let dimension = colimit.dimension();
    let (regular_slices_by_height, singular_slices_by_height) = {
        let mut regular_slices_by_height: IdxVec<NodeIndex<Ix>, Vec<Vec<Rewrite>>> =
            IdxVec::splat(Vec::with_capacity(cocones.len()), graph.node_count());
        let mut singular_slices_by_height: IdxVec<NodeIndex<Ix>, Vec<Vec<Rewrite>>> =
            IdxVec::splat(Vec::with_capacity(cocones.len()), graph.node_count());
        for (_, cocone, _, restriction_to_exploded) in cocones {
            for (graph_ix, slices) in &cocone.legs.iter().group_by(|(restriction_ix, _)| {
                exploded[restriction_to_exploded[*restriction_ix]].0 .0
            }) {
                let mut cone_regular_slices: Vec<Rewrite> = Default::default();
                let mut cone_singular_slices: Vec<Rewrite> = Default::default();
                for (restriction_ix, slice) in slices {
                    match exploded[restriction_to_exploded[restriction_ix]].0 .1 {
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
                <&DiagramN>::try_from(&graph[NodeIndex::new(n)].0)
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

#[cfg(test)]
mod test {
    use petgraph::graph::EdgeIndex;

    use super::*;
    use crate::{
        examples::{stacks, two_beads, two_endomorphism},
        graph::SliceGraph,
        Boundary::{Source, Target},
        Height::{Regular, Singular},
        SliceIndex::{Boundary, Interior},
    };

    fn keep_interior_nodes(_: NodeIndex, _: &'_ (), si: SliceIndex) -> Option<()> {
        match si {
            SliceIndex::Boundary(_) => None,
            SliceIndex::Interior(_) => Some(()),
        }
    }
    fn keep_internal_edges(_: NodeIndex, _: &'_ (), ir: InternalRewrite) -> Option<()> {
        match ir {
            InternalRewrite::Boundary(_) => None,
            InternalRewrite::Interior(_, _) => Some(()),
        }
    }
    fn keep_external_edges(_: EdgeIndex, _: &'_ (), _: ExternalRewrite) -> Option<()> {
        Some(())
    }

    fn internal_cospan(g: Generator) -> Cospan {
        let x_generator = Generator::new(0, 0);
        Cospan {
            forward: Rewrite0::new(
                x_generator,
                g,
                (g, vec![Interior(Singular(0)), Boundary(Source)]).into(),
            )
            .into(),
            backward: Rewrite0::new(
                x_generator,
                g,
                (g, vec![Interior(Singular(0)), Boundary(Target)]).into(),
            )
            .into(),
        }
    }

    fn forward_regular(src: Generator, tgt: Generator) -> Rewrite {
        Rewrite0::new(
            src,
            tgt,
            (tgt, vec![Interior(Singular(0)), Boundary(Source)]).into(),
        )
        .into()
    }

    fn backward_regular(src: Generator, tgt: Generator) -> Rewrite {
        Rewrite0::new(
            src,
            tgt,
            (tgt, vec![Interior(Singular(0)), Boundary(Target)]).into(),
        )
        .into()
    }

    fn vertical_cospan(g: Generator) -> Cospan {
        let f_generator = Generator::new(1, 1);
        Cospan {
            forward: Rewrite0::new(
                f_generator,
                g,
                (g, vec![Boundary(Source), Interior(Singular(0))]).into(),
            )
            .into(),
            backward: Rewrite0::new(
                f_generator,
                g,
                (g, vec![Boundary(Target), Interior(Singular(0))]).into(),
            )
            .into(),
        }
    }

    #[test]
    fn collapse_two_beads_base_case() {
        let x_generator = Generator::new(0, 0);
        let f_generator = Generator::new(1, 1);
        let a_generator = Generator::new(2, 2);
        let b_generator = Generator::new(3, 2);
        let (_sig, two_beads) = two_beads();
        let ExplosionOutput { output: graph, .. }: ExplosionOutput<(), (), DefaultIx, DefaultIx> =
            SliceGraph::singleton((), Diagram::from(two_beads))
                .explode(
                    keep_interior_nodes,
                    keep_internal_edges,
                    keep_external_edges,
                )
                .unwrap()
                .explode(
                    keep_interior_nodes,
                    keep_internal_edges,
                    keep_external_edges,
                )
                .unwrap();
        {
            // left side
            let subproblem = graph.filter_map(
                |i, ((), diagram)| (i.index() % 5 <= 2).then(|| (diagram.clone(), 0)),
                |_, ((), edge)| Some(edge.clone()),
            );
            let actual: Cocone<DefaultIx> = collapse_base(&subproblem).unwrap();
            assert_eq!(actual.colimit, Diagram::Diagram0(a_generator));
            assert_eq!(actual.legs.len(), 15);

            let left = Rewrite0::new(
                x_generator,
                a_generator,
                (a_generator, vec![Interior(Singular(0)), Boundary(Source)]).into(),
            )
            .into();
            let right = Rewrite0::new(
                x_generator,
                a_generator,
                (a_generator, vec![Interior(Singular(0)), Boundary(Target)]).into(),
            )
            .into();
            let top = Rewrite0::new(
                f_generator,
                a_generator,
                (a_generator, vec![Boundary(Target), Interior(Singular(0))]).into(),
            )
            .into();
            let bottom = Rewrite0::new(
                f_generator,
                a_generator,
                (a_generator, vec![Boundary(Source), Interior(Singular(0))]).into(),
            )
            .into();
            [
                (NodeIndex::new(0), &left),
                (NodeIndex::new(1), &bottom),
                (NodeIndex::new(2), &right),
                (NodeIndex::new(3), &left),
                (NodeIndex::new(4), &Rewrite::identity(0)),
                (NodeIndex::new(5), &right),
                (NodeIndex::new(6), &left),
                (NodeIndex::new(7), &top),
                (NodeIndex::new(8), &right),
                (NodeIndex::new(9), &left),
                (NodeIndex::new(10), &top),
                (NodeIndex::new(11), &right),
                (NodeIndex::new(12), &left),
                (NodeIndex::new(13), &top),
                (NodeIndex::new(14), &right),
            ]
            .into_iter()
            .for_each(|(i, expected_rewrite)| {
                assert_eq!(actual.legs.get(i).unwrap(), expected_rewrite)
            });
        }
        {
            // right side
            let subproblem = graph.filter_map(
                |i, ((), diagram)| (i.index() % 5 >= 2).then(|| (diagram.clone(), 0)),
                |_, ((), edge)| Some(edge.clone()),
            );
            let actual: Cocone<DefaultIx> = collapse_base(&subproblem).unwrap();
            assert_eq!(actual.legs.len(), 15);
            assert_eq!(actual.colimit, Diagram::Diagram0(b_generator));

            let left = Rewrite0::new(
                x_generator,
                b_generator,
                (b_generator, vec![Interior(Singular(0)), Boundary(Source)]).into(),
            )
            .into();
            let right = Rewrite0::new(
                x_generator,
                b_generator,
                (b_generator, vec![Interior(Singular(0)), Boundary(Target)]).into(),
            )
            .into();
            let top = Rewrite0::new(
                f_generator,
                b_generator,
                (b_generator, vec![Boundary(Target), Interior(Singular(0))]).into(),
            )
            .into();
            let bottom = Rewrite0::new(
                f_generator,
                b_generator,
                (b_generator, vec![Boundary(Source), Interior(Singular(0))]).into(),
            )
            .into();
            [
                (NodeIndex::new(0), &left),
                (NodeIndex::new(1), &bottom),
                (NodeIndex::new(2), &right),
                (NodeIndex::new(3), &left),
                (NodeIndex::new(4), &bottom),
                (NodeIndex::new(5), &right),
                (NodeIndex::new(6), &left),
                (NodeIndex::new(7), &bottom),
                (NodeIndex::new(8), &right),
                (NodeIndex::new(9), &left),
                (NodeIndex::new(10), &Rewrite::identity(0)),
                (NodeIndex::new(11), &right),
                (NodeIndex::new(12), &left),
                (NodeIndex::new(13), &top),
                (NodeIndex::new(14), &right),
            ]
            .into_iter()
            .for_each(|(i, expected_rewrite)| {
                assert_eq!(actual.legs.get(i).unwrap(), expected_rewrite)
            });
        }
    }

    #[test]
    fn collapse_two_beads_recursive_case() {
        let x_generator = Generator::new(0, 0);
        let _f_generator = Generator::new(1, 1);
        let a_generator = Generator::new(2, 2);
        let b_generator = Generator::new(3, 2);
        let (_sig, two_beads) = two_beads();
        let ExplosionOutput { output: graph, .. }: ExplosionOutput<(), (), DefaultIx, DefaultIx> =
            SliceGraph::singleton((), Diagram::from(two_beads.clone()))
                .explode(
                    keep_interior_nodes,
                    keep_internal_edges,
                    keep_external_edges,
                )
                .unwrap();
        let problem = graph.map(
            |_, ((), diagram)| (diagram.clone(), 0),
            |_, ((), edge)| edge.clone(),
        );
        let actual: Cocone<DefaultIx> = collapse_recursive(&problem).unwrap();
        assert_eq!(actual.legs.len(), 5);
        assert_eq!(
            actual.colimit,
            DiagramN::new(
                x_generator.into(),
                vec![internal_cospan(a_generator), internal_cospan(b_generator)]
            )
            .into()
        );

        two_beads
            .slices()
            .enumerate()
            .map(|(i, slice)| {
                (
                    NodeIndex::new(i),
                    RewriteN::from_slices(
                        1,
                        DiagramN::try_from(slice).unwrap().cospans(),
                        &[internal_cospan(a_generator), internal_cospan(b_generator)],
                        // regular slices
                        vec![
                            vec![
                                forward_regular(x_generator, a_generator),
                                backward_regular(x_generator, a_generator),
                            ],
                            vec![
                                forward_regular(x_generator, b_generator),
                                backward_regular(x_generator, b_generator),
                            ],
                        ],
                        // singular slices
                        match i {
                            i if i == 0 => vec![
                                vec![vertical_cospan(a_generator).forward],
                                vec![vertical_cospan(b_generator).forward],
                            ],
                            i if i == 1 => vec![
                                vec![Rewrite::identity(0)],
                                vec![vertical_cospan(b_generator).forward],
                            ],
                            i if i == 2 => vec![
                                vec![vertical_cospan(a_generator).backward],
                                vec![vertical_cospan(b_generator).forward],
                            ],
                            i if i == 3 => vec![
                                vec![vertical_cospan(a_generator).backward],
                                vec![Rewrite::identity(0)],
                            ],
                            i if i == 4 => vec![
                                vec![vertical_cospan(a_generator).backward],
                                vec![vertical_cospan(b_generator).backward],
                            ],
                            _ => unreachable!(),
                        },
                    )
                    .into(),
                )
            })
            .for_each(|(i, expected_rewrite): (NodeIndex, Rewrite)| {
                assert_eq!(
                    actual.legs.get(i).unwrap(),
                    &expected_rewrite,
                    "mismatch at slice {}",
                    i.index()
                )
            });
    }

    #[test]
    fn collapse_stacks() {
        let x_generator = Generator::new(0, 0);
        let _f_generator = Generator::new(1, 1);
        let m_generator = Generator::new(2, 2);
        let (_sig, stacks) = stacks();
        let ExplosionOutput { output: graph, .. }: ExplosionOutput<(), (), DefaultIx, DefaultIx> =
            SliceGraph::singleton((), Diagram::from(stacks.clone()))
                .explode(
                    keep_interior_nodes,
                    keep_internal_edges,
                    keep_external_edges,
                )
                .unwrap();
        let problem = graph.map(
            |_, ((), diagram)| (diagram.clone(), 0),
            |_, ((), edge)| edge.clone(),
        );
        let actual: Cocone<DefaultIx> = collapse_recursive(&problem).unwrap();
        assert_eq!(actual.legs.len(), 5);
        assert_eq!(
            actual.colimit,
            DiagramN::new(
                x_generator.into(),
                vec![internal_cospan(m_generator), internal_cospan(m_generator)]
            )
            .into()
        );

        let upper_regular: Rewrite = Rewrite0::new(
            x_generator,
            m_generator,
            (m_generator, vec![Boundary(Target), Interior(Regular(0))]).into(),
        )
        .into();

        stacks
            .slices()
            .enumerate()
            .map(|(i, slice)| {
                (
                    NodeIndex::new(i),
                    RewriteN::from_slices(
                        1,
                        DiagramN::try_from(slice).unwrap().cospans(),
                        &[internal_cospan(m_generator), internal_cospan(m_generator)],
                        match i {
                            i if i <= 1 => vec![
                                vec![
                                    forward_regular(x_generator, m_generator),
                                    backward_regular(x_generator, m_generator),
                                ],
                                vec![
                                    forward_regular(x_generator, m_generator),
                                    backward_regular(x_generator, m_generator),
                                ],
                            ],
                            i if i == 2 => vec![
                                vec![upper_regular.clone()],
                                vec![
                                    forward_regular(x_generator, m_generator),
                                    backward_regular(x_generator, m_generator),
                                ],
                            ],
                            i if i == 3 => vec![
                                vec![upper_regular.clone()],
                                vec![
                                    forward_regular(x_generator, m_generator),
                                    backward_regular(x_generator, m_generator),
                                ],
                            ],
                            i if i == 4 => {
                                vec![vec![upper_regular.clone()], vec![upper_regular.clone()]]
                            }
                            _ => unreachable!(),
                        },
                        match i {
                            i if i == 0 => vec![
                                vec![vertical_cospan(m_generator).forward],
                                vec![vertical_cospan(m_generator).forward],
                            ],
                            i if i == 1 => vec![
                                vec![Rewrite::identity(0)],
                                vec![vertical_cospan(m_generator).forward],
                            ],
                            i if i == 2 => vec![vec![], vec![vertical_cospan(m_generator).forward]],
                            i if i == 3 => vec![vec![], vec![Rewrite::identity(0)]],
                            i if i == 4 => vec![vec![], vec![]],
                            _ => unreachable!(),
                        },
                    )
                    .into(),
                )
            })
            .for_each(|(i, expected_rewrite): (NodeIndex, Rewrite)| {
                assert_eq!(
                    actual.legs.get(i).unwrap(),
                    &expected_rewrite,
                    "mismatch at slice {}",
                    i.index()
                )
            });
    }

    #[test]
    fn contract_invertible_1d() {
        let x_generator = Generator::new(0, 0);
        let f_generator = Generator::new(1, 1);
        let forward: Rewrite = Rewrite0::new(
            x_generator,
            f_generator,
            (f_generator, vec![Boundary(Source)]).into(),
        )
        .into();
        let backward: Rewrite = Rewrite0::new(
            x_generator,
            f_generator,
            (f_generator, vec![Boundary(Target)]).into(),
        )
        .into();
        let f_cospan = Cospan {
            forward: forward.clone(),
            backward: backward.clone(),
        };
        let f_inverse_cospan = Cospan {
            forward: backward.clone(),
            backward: forward.clone(),
        };

        let f_then_inverse = DiagramN::new(
            Diagram::Diagram0(x_generator).into(),
            vec![f_cospan.clone(), f_inverse_cospan.clone()],
        );
        assert_eq!(
            contract_base(&f_then_inverse, 0, None).unwrap(),
            RewriteN::from_slices(
                1,
                f_then_inverse.cospans(),
                &[Cospan {
                    forward: forward.clone(),
                    backward: forward.clone(),
                }],
                vec![vec![forward.clone(), backward.clone(), forward.clone()]],
                vec![vec![Rewrite::identity(0), Rewrite::identity(0)]]
            )
        );

        let inverse_then_f = DiagramN::new(
            Diagram::Diagram0(x_generator).into(),
            vec![f_inverse_cospan, f_cospan.clone()],
        );
        assert_eq!(
            contract_base(&inverse_then_f, 0, None).unwrap(),
            RewriteN::from_slices(
                1,
                inverse_then_f.cospans(),
                &[Cospan {
                    forward: backward.clone(),
                    backward: backward.clone(),
                }],
                vec![vec![backward.clone(), forward, backward]],
                vec![vec![Rewrite::identity(0), Rewrite::identity(0)]]
            )
        );

        let f_then_f = DiagramN::new(
            Diagram::Diagram0(x_generator).into(),
            vec![f_cospan.clone(), f_cospan],
        );
        assert!(contract_base(&f_then_f, 0, None).is_err())
    }

    #[test]
    fn contract_invertible_2d() {
        let (_sig, e) = two_endomorphism();

        let e_then_inverse = DiagramN::new(
            e.source(),
            [e.cospans(), e.inverse().unwrap().cospans()].concat(),
        );
        assert_eq!(
            contract_base(&e_then_inverse, 0, Some(Bias::Higher)).unwrap(),
            RewriteN::from_slices(
                2,
                e_then_inverse.cospans(),
                &[Cospan {
                    forward: e.cospans()[0].forward.clone(),
                    backward: e.cospans()[0].forward.clone(),
                }],
                vec![vec![
                    e.cospans()[0].forward.clone(),
                    e.cospans()[0].backward.clone(),
                    e.cospans()[0].forward.clone()
                ]],
                vec![vec![Rewrite::identity(1), Rewrite::identity(1)]]
            )
        );

        let inverse_then_e = DiagramN::new(
            e.source(),
            [e.inverse().unwrap().cospans(), e.cospans()].concat(),
        );
        assert_eq!(
            contract_base(&inverse_then_e, 0, None).unwrap(),
            RewriteN::from_slices(
                2,
                inverse_then_e.cospans(),
                &[Cospan {
                    forward: e.cospans()[0].backward.clone(),
                    backward: e.cospans()[0].backward.clone(),
                }],
                vec![vec![
                    e.cospans()[0].backward.clone(),
                    e.cospans()[0].forward.clone(),
                    e.cospans()[0].backward.clone()
                ]],
                vec![vec![Rewrite::identity(1), Rewrite::identity(1)]]
            )
        );

        let e_then_e = DiagramN::new(e.source(), [e.cospans(), e.cospans()].concat());
        assert!(contract_base(&e_then_e, 0, None).is_err());
    }
}
