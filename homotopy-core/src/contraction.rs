use std::{
    cmp::Ordering,
    convert::{Into, TryInto},
    hash::Hash,
};

use homotopy_common::{declare_idx, hash::FastHashMap, idx::IdxVec};
use itertools::Itertools;
use once_cell::unsync::OnceCell;
use petgraph::{
    adj::UnweightedList,
    algo::{
        condensation, toposort,
        tred::{dag_to_toposorted_adjacency_list, dag_transitive_reduction_closure},
    },
    graph::{DefaultIx, DiGraph, IndexType, NodeIndex},
    graphmap::DiGraphMap,
    stable_graph::{EdgeIndex, StableDiGraph},
    unionfind::UnionFind,
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences, Topo, Walker},
    EdgeDirection::{Incoming, Outgoing},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    attach::{attach, BoundaryPath},
    common::{Boundary, DimensionError, Height, Orientation, SingularHeight},
    diagram::{Diagram, DiagramN},
    expansion::expand_propagate,
    graph::{Explodable, ExplosionOutput, ExternalRewrite, InternalRewrite},
    rewrite::{Cone, Cospan, Label, Rewrite, Rewrite0, RewriteN},
    signature::Signature,
    Direction, Generator, SliceIndex,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
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
    #[error("contraction failed: max dimensional generator not unique")]
    NonUniqueMaxDimensionGenerator,
    #[error("contraction failed: orientation error")]
    Orientation,
    #[error("contraction invalid")]
    Invalid,
    #[error("contraction ambiguous")]
    Ambiguous,
    #[error("invalid boundary path provided to contraction")]
    Dimension(#[from] DimensionError),
}

struct ContractExpand {
    contract: RewriteN,
    expand: RewriteN,
}

impl DiagramN {
    pub fn contract<S>(
        &self,
        boundary_path: BoundaryPath,
        interior_path: &[Height],
        height: SingularHeight,
        bias: Option<Bias>,
        _signature: &S,
    ) -> Result<Self, ContractionError>
    where
        S: Signature,
    {
        if boundary_path.1 >= self.dimension() {
            return Err(ContractionError::Invalid);
        }

        attach(self, boundary_path, |slice| {
            let slice = slice.try_into().or(Err(ContractionError::Invalid))?;
            let ContractExpand { contract, expand } =
                contract_in_path(&slice, interior_path, height, bias)?;
            let _singular = slice.rewrite_forward(&contract).unwrap();

            let cospan = match boundary_path.boundary() {
                Boundary::Source => Cospan {
                    forward: expand.into(),
                    backward: contract.into(),
                },
                Boundary::Target => Cospan {
                    forward: contract.into(),
                    backward: expand.into(),
                },
            };

            Ok(vec![cospan])
        })
    }
}

fn contract_base(
    diagram: &DiagramN,
    height: SingularHeight,
    bias: Option<Bias>,
) -> Result<ContractExpand, ContractionError> {
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
        None => (None, None),
        Some(b) => (Some(b.flip()), Some(b)),
    };

    let mut graph = DiGraph::new();
    let r0 = graph.add_node((regular0.clone(), None, vec![Height::Regular(0)]));
    let s0 = graph.add_node((singular0.clone(), bias0, vec![Height::Singular(0)]));
    let r1 = graph.add_node((regular1.clone(), None, vec![Height::Regular(1)]));
    let s1 = graph.add_node((singular1.clone(), bias1, vec![Height::Singular(1)]));
    let r2 = graph.add_node((regular2.clone(), None, vec![Height::Regular(2)]));
    graph.add_edge(r0, s0, cospan0.forward.clone());
    graph.add_edge(r1, s0, cospan0.backward.clone());
    graph.add_edge(r1, s1, cospan1.forward.clone());
    graph.add_edge(r2, s1, cospan1.backward.clone());
    let result = collapse(&graph)?;

    let cospan = Cospan {
        forward: result.legs[r0].clone(),
        backward: result.legs[r2].clone(),
    };

    let contract = RewriteN::new(
        diagram.dimension(),
        vec![Cone::new(
            height,
            vec![cospan0.clone(), cospan1.clone()],
            cospan.clone(),
            vec![
                result.legs[r0].clone(),
                result.legs[r1].clone(),
                result.legs[r2].clone(),
            ],
            vec![result.legs[s0].clone(), result.legs[s1].clone()],
        )],
    );

    let expand = match result.colimit {
        Diagram::Diagram0(_) => {
            // Coarse smoothing
            // A cospan is smoothable if the forward and backward rewrites are identical and redundant.
            let cone = cospan
                .is_redundant()
                .then(|| Cone::new_unit(height, cospan.clone(), cospan.forward));
            RewriteN::new(diagram.dimension(), cone.into_iter().collect())
        }
        Diagram::DiagramN(colimit) => {
            // Cone-wise smoothing
            // A pair of cones over the same target height is smoothable if they are identical (modulo different indices) and redundant.
            let forward: &RewriteN = (&cospan.forward).try_into().unwrap();
            let backward: &RewriteN = (&cospan.backward).try_into().unwrap();

            let mut s_cones = vec![];
            let mut f_cones = vec![];
            let mut b_cones = vec![];
            for height in 0..colimit.size() {
                match (
                    forward.cone_over_target(height),
                    backward.cone_over_target(height),
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
                Cone::new_unit(height, cospan, smooth)
            } else {
                // Keep diagram height the same.
                Cone::new(
                    height,
                    vec![smooth_cospan],
                    cospan.clone(),
                    vec![cospan.forward, cospan.backward],
                    vec![smooth],
                )
            };

            RewriteN::new(diagram.dimension(), vec![cone])
        }
    };

    Ok(ContractExpand { contract, expand })
}

fn contract_in_path(
    diagram: &DiagramN,
    path: &[Height],
    height: SingularHeight,
    bias: Option<Bias>,
) -> Result<ContractExpand, ContractionError> {
    match path.split_first() {
        None => contract_base(diagram, height, bias),
        Some((step, rest)) => {
            let slice: DiagramN = diagram
                .slice(*step)
                .ok_or(ContractionError::Invalid)?
                .try_into()
                .ok()
                .ok_or(ContractionError::Invalid)?;
            let ContractExpand {
                contract: contract_base,
                expand: expand_base,
            } = contract_in_path(&slice, rest, height, bias)?;
            match step {
                Height::Regular(i) => {
                    let contract = RewriteN::new(
                        diagram.dimension(),
                        vec![Cone::new_unit(
                            *i,
                            Cospan {
                                forward: contract_base.clone().into(),
                                backward: contract_base.clone().into(),
                            },
                            contract_base.into(),
                        )],
                    );
                    let expand = expand_propagate(
                        &diagram
                            .clone()
                            .rewrite_forward(&contract)
                            .map_err(|_err| ContractionError::Invalid)?,
                        *i,
                        expand_base.into(),
                    )
                    .map_err(|_err| ContractionError::Invalid)?;
                    Ok(ContractExpand { contract, expand })
                }
                Height::Singular(i) => {
                    let source_cospan = &diagram.cospans()[*i];
                    let contract_base = contract_base.into();
                    let (forward, backward) = {
                        // compose by collapse
                        let mut graph = DiGraph::new();
                        let regular_prev = diagram
                            .slice(SliceIndex::Interior(Height::Regular(*i)))
                            .ok_or(ContractionError::Invalid)?;
                        let r_p = graph.add_node((
                            regular_prev.clone(),
                            None,
                            vec![Height::Regular(0), Height::Regular(*i)],
                        ));
                        let singular = regular_prev
                            .rewrite_forward(&source_cospan.forward)
                            .map_err(|_err| ContractionError::Invalid)?;
                        let s = graph.add_node((
                            singular.clone(),
                            None,
                            vec![Height::Regular(0), Height::Singular(*i)],
                        ));
                        graph.add_edge(r_p, s, source_cospan.forward.clone());
                        let regular_next = singular
                            .clone()
                            .rewrite_backward(&source_cospan.backward)
                            .map_err(|_err| ContractionError::Invalid)?;
                        let r_n = graph.add_node((
                            regular_next,
                            None,
                            vec![Height::Regular(0), Height::Regular(*i + 1)],
                        ));
                        graph.add_edge(r_n, s, source_cospan.backward.clone());
                        let c = graph.add_node((
                            singular
                                .rewrite_forward(&contract_base)
                                .map_err(|_err| ContractionError::Invalid)?,
                            None,
                            vec![Height::Singular(0), Height::Singular(*i)],
                        ));
                        graph.add_edge(s, c, contract_base.clone());
                        let cocone = collapse(&graph)?;
                        (cocone.legs[r_p].clone(), cocone.legs[r_n].clone())
                    };
                    let contract = RewriteN::new(
                        diagram.dimension(),
                        vec![Cone::new(
                            *i,
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
                        &diagram
                            .clone()
                            .rewrite_forward(&contract)
                            .map_err(|_err| ContractionError::Invalid)?,
                        *i,
                        expand_base.into(),
                    )
                    .map_err(|_err| ContractionError::Invalid)?;
                    Ok(ContractExpand { contract, expand })
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
    graph: &DiGraph<(Diagram, Option<Bias>, Vec<Height>), Rewrite, Ix>,
) -> Result<Cocone<Ix>, ContractionError> {
    let dimension = graph
        .node_weights()
        .next()
        .ok_or(ContractionError::Invalid)?
        .0
        .dimension();

    for (diagram, _bias, _coord) in graph.node_weights() {
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

fn collapse_base<Ix: IndexType>(
    graph: &DiGraph<(Diagram, Option<Bias>, Vec<Height>), Rewrite, Ix>,
) -> Result<Cocone<Ix>, ContractionError> {
    // helper function to unify two nodes within a stable graph
    fn unify<N, Ix, RN, RE>(
        graph: &mut StableDiGraph<N, Rewrite, Ix>,
        p: NodeIndex<Ix>,
        q: NodeIndex<Ix>,
        quotient: &mut UnionFind<NodeIndex<Ix>>,
        mut on_remove_node: RN,
        mut on_remove_edge: RE,
    ) -> Result<(), ContractionError>
    where
        N: Clone,
        Ix: IndexType,
        RE: FnMut(EdgeIndex<Ix>),
        RN: FnMut(NodeIndex<Ix>),
    {
        enum Quotient<Ix> {
            RetargetSource(NodeIndex<Ix>, NodeIndex<Ix>, EdgeIndex<Ix>),
            RetargetTarget(NodeIndex<Ix>, NodeIndex<Ix>, EdgeIndex<Ix>),
            RemoveNode(NodeIndex<Ix>),
        }
        let (p, q) = (quotient.find_mut(p), quotient.find_mut(q));
        if p == q {
            return Ok(());
        }
        quotient.union(p, q);
        let keep = quotient.find_mut(p);
        let remove = if keep == p { q } else { p };
        let mut ops: Vec<Quotient<Ix>> = Default::default();
        for e in graph
            .edges_directed(remove, Outgoing)
            .filter(|e| e.target() != keep)
        {
            ops.push(Quotient::RetargetSource(keep, e.target(), e.id()));
        }
        for e in graph
            .edges_directed(remove, Incoming)
            .filter(|e| e.source() != keep)
        {
            ops.push(Quotient::RetargetTarget(keep, e.source(), e.id()));
        }
        ops.push(Quotient::RemoveNode(remove));
        for op in ops {
            match op {
                Quotient::RetargetSource(keep, target, e) => {
                    if let Some(existing) = graph.find_edge(keep, target) {
                        if <&Rewrite0>::try_from(&graph[existing])
                            .expect("non 0-rewrite passed to collapse_base unify")
                            .label()
                            .and_then(|l| l.0.as_ref())
                            != <&Rewrite0>::try_from(&graph[e])
                                .expect("non 0-rewrite passed to collapse_base unify")
                                .label()
                                .and_then(|l| l.0.as_ref())
                        {
                            return Err(ContractionError::LabelInconsistency);
                        }
                    } else {
                        graph.add_edge(keep, target, graph[e].clone());
                    }
                    graph.remove_edge(e);
                    on_remove_edge(e);
                }
                Quotient::RetargetTarget(keep, source, e) => {
                    if let Some(existing) = graph.find_edge(source, keep) {
                        if <&Rewrite0>::try_from(&graph[existing])
                            .expect("non 0-rewrite passed to collapse_base unify")
                            .label()
                            .and_then(|l| l.0.as_ref())
                            != <&Rewrite0>::try_from(&graph[e])
                                .expect("non 0-rewrite passed to collapse_base unify")
                                .label()
                                .and_then(|l| l.0.as_ref())
                        {
                            return Err(ContractionError::LabelInconsistency);
                        }
                    } else {
                        graph.add_edge(source, keep, graph[e].clone());
                    }
                    graph.remove_edge(e);
                    on_remove_edge(e);
                }
                Quotient::RemoveNode(remove) => {
                    graph.remove_node(remove);
                    on_remove_node(remove);
                }
            };
        }
        Ok(())
    }
    // mutably construct the collapsed graph
    let mut stable = StableDiGraph::from(graph.clone());
    // invariant: #nodes of stable = #equivalence classes of union_find
    let mut union_find = UnionFind::new(stable.node_count());
    // tree tracks which edges descended from other edges by graph explosion
    // collapse subproblems need to be solved in topological order, with the root being the final one
    declare_idx! { struct TreeIx = DefaultIx; }
    let tree = {
        let mut tree: DiGraph<_, _, TreeIx> = Default::default();
        let root = tree.add_node((None, OnceCell::new()));
        for (ix, (_, _, coord)) in stable.node_references() {
            let mut cur = root;
            for c in coord {
                if let Some(existing) = tree
                    .neighbors_directed(cur, Incoming)
                    .find(|n| tree[*n].0 == Some(*c))
                {
                    cur = existing;
                } else {
                    let next = tree.add_node((Some(*c), OnceCell::new()));
                    tree.add_edge(next, cur, ());
                    cur = next;
                }
            }
            tree[cur]
                .1
                .set(vec![ix])
                .expect("failed to initialise collapse subproblem tree");
        }
        tree
    };
    for n in Topo::new(&tree).iter(&tree) {
        // collapse subproblem
        let mut children = tree.neighbors_directed(n, Incoming).detach();
        let mut nodes = vec![];
        while let Some(child) = children.next_node(&tree) {
            nodes.extend_from_slice(tree[child].1.get().unwrap());
        }
        if nodes.is_empty() {
            // n is a leaf
            continue;
        } else {
            let mut quotient: Vec<_> = Default::default();
            // find collapsible edges wrt nodes
            for e in stable.edge_references().filter(|e| {
                // e is contained within nodes
                nodes.contains(&e.source()) && nodes.contains(&e.target())
                // e is an identity rewrite
                && <&Rewrite0>::try_from(e.weight()).unwrap().0.as_ref().map_or(true, |(s, t, _)| s.id == t.id)
                // check triangles within nodes which might refute collapsibility of e
                && stable.edges_directed(e.source(), Incoming).filter(|p| nodes.contains(&p.source())).all(|p| {
                    if let Some(c) = stable.find_edge(p.source(), e.target()) {
                        <&Rewrite0>::try_from(p.weight()).unwrap().label() == <&Rewrite0>::try_from(stable.edge_weight(c).unwrap()).unwrap().label()
                    } else {
                        true
                    }
                })
                && stable.edges_directed(e.target(), Outgoing).filter(|n| nodes.contains(&n.target())).all(|n| {
                    if let Some(c) = stable.find_edge(e.source(), n.target()) {
                        <&Rewrite0>::try_from(n.weight()).unwrap().label() == <&Rewrite0>::try_from(stable.edge_weight(c).unwrap()).unwrap().label()
                    } else {
                        true
                    }
                })
            }) {
                // e is collapsible
                quotient.push((e.source(), e.target()));
            }

            for (s, t) in quotient {
                unify(
                    &mut stable,
                    s,
                    t,
                    &mut union_find,
                    |rn| {
                        nodes.retain(|&n| n != rn);
                    },
                    |_re| (),
                )
                .expect("collapsed along non collapsible edge!");
            }
        }
        tree[n]
            .1
            .set(nodes)
            .expect("failed to propagate collapse subproblem");
    }
    // check the tree of collapse subproblems has been completed
    debug_assert!(tree[NodeIndex::new(0)].1.get().is_some());

    // unify all nodes of maximal dimension
    let (max_dim_index, max_dim_generator) = stable
        .node_references()
        .map(|(i, (d, _bias, _coord))| {
            let g: Generator = d.try_into().unwrap();
            (i, g)
        })
        .max_by_key(|(_, g)| g.dimension)
        .ok_or(ContractionError::NonUniqueMaxDimensionGenerator)?;

    let codimension = stable[max_dim_index]
        .2
        .len()
        .saturating_sub(max_dim_generator.dimension);

    // Collect the orientations of the maximum-dimensional generator by subslice.
    let mut orientations = FastHashMap::<&[Height], Vec<Orientation>>::default();

    let mut max_dims: Vec<_> = Default::default();
    for (i, (d, _bias, coord)) in graph.node_references() {
        let g: Generator = d.try_into().unwrap();
        if g.dimension == max_dim_generator.dimension {
            if g.id != max_dim_generator.id {
                // found distinct elements of maximal dimension
                return Err(ContractionError::NonUniqueMaxDimensionGenerator);
            }
            if stable.contains_node(i) {
                max_dims.push(i);
            };

            orientations
                .entry(&coord[..codimension])
                .or_default()
                .push(g.orientation);
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
        )?;
    }

    let colimit = Generator {
        orientation,
        ..max_dim_generator
    };

    // construct colimit legs
    let legs = {
        let mut legs = IdxVec::with_capacity(graph.node_count());
        for (n, (d, _bias, _coord)) in graph.node_references() {
            let g: Generator = d.try_into().unwrap();
            let r = {
                let (p, q) = (union_find.find_mut(n), union_find.find_mut(max_dim_index));
                if p == q {
                    debug_assert_eq!(g.id, colimit.id);
                    Rewrite0::new(g, colimit, Label::new(None))
                } else {
                    let label = <&Rewrite0>::try_from(
                        &stable[stable
                            .find_edge(p, q)
                            .expect("node in collapse base does not connect to max_dim_index")],
                    )
                    .expect("non 0-rewrite passed to collapse_base")
                    .label()
                    .cloned()
                    .unwrap();
                    Rewrite0::new(g, colimit, label)
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

fn collapse_recursive<Ix: IndexType>(
    graph: &DiGraph<(Diagram, Option<Bias>, Vec<Height>), Rewrite, Ix>,
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
        .map(
            |_, (d, bias, coord)| ((*bias, coord.clone()), d.clone()),
            |_, e| ((), e.clone()),
        )
        .explode(
            |parent_node, (_bias, coord), si| match si {
                SliceIndex::Boundary(_) => None,
                SliceIndex::Interior(h) => {
                    let mut coord = coord.clone();
                    coord.push(h);
                    Some((parent_node, h, coord))
                }
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
                .filter(|&i| matches!(exploded[*i], ((_, Height::Singular(_), _), _)))
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
                .map(|&n| graph[exploded[n].0 .0].1)
                .min()
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
    let regular_monotone: Vec<IdxVec<NodeIndex<Ix>, _>> = {
        let mut regular_monotone: Vec<IdxVec<NodeIndex<Ix>, _>> =
            Vec::with_capacity(linear_components.len() + 1);
        regular_monotone.push(
            // all targeting Regular(0)
            exploded
                .node_references()
                .filter_map(|(i, ((p, h, _coord), _))| {
                    (graph.externals(Outgoing).contains(p) // comes from singular height (i.e. in Δ)
                        && *h == Height::Regular(0))
                    .then(|| i)
                })
                .collect(),
        );
        for scc in &linear_components {
            // get the right-most boundary of this scc
            regular_monotone.push(
                scc.iter()
                    .group_by(|&i| exploded[*i].0 .0)
                    .into_iter()
                    .map(|(p, group)| {
                        group.max().map_or_else(
                            || regular_monotone.last().unwrap()[p], // TODO: this is wrong
                            |i| NodeIndex::new(i.index() + 1),      // next regular level,
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
            let restriction: DiGraph<(Diagram, Option<Bias>, Vec<Height>), _, RestrictionIx> =
                exploded.filter_map(
                    |i, ((_, _, coord), diagram)| {
                        scc.iter()
                            .chain(adjacent_regulars[0].values())
                            .chain(adjacent_regulars[1].values())
                            .any(|&c| {
                                i == c
                                    || closure.contains_edge(revmap[i.index()], revmap[c.index()])
                            })
                            .then(|| {
                                restriction_to_exploded.push(i);
                                (diagram.clone(), graph[exploded[i].0 .0].1, coord.clone())
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
            let restriction = restriction.filter_map(
                |_, (d, bias, coord)| {
                    (
                        d.clone(),
                        bias.filter(|bias| *bias == Bias::Same),
                        coord.clone(),
                    )
                        .into()
                },
                |_, (_, r)| r.clone().into(),
            );
            let cocone: Cocone<RestrictionIx> = collapse(&restriction)?;
            Ok((source, cocone, target, restriction_to_exploded))
        })
        .fold_ok(vec![], |mut acc, x| {
            acc.push(x);
            acc
        })?;

    // assemble solutions
    let (s, first, _, _) = cocones.first().ok_or(ContractionError::Invalid)?;
    let colimit: DiagramN = if let Ok(terminal) = graph.externals(Outgoing).exactly_one() {
        DiagramN::try_from(graph[terminal].0.clone()).unwrap()
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
                exploded[restriction_to_exploded[*restriction_ix]].0 .0
            }) {
                // each rewrite that will go into legs[graph_ix] from cocone
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
