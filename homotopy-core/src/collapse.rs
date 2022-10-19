//! Functions to collapse diagram scaffolds; used in contraction, typechecking etc.
use anyhow::anyhow;
use homotopy_common::declare_idx;
use once_cell::unsync::OnceCell;
use petgraph::{
    prelude::DiGraph,
    stable_graph::{DefaultIx, EdgeIndex, IndexType, NodeIndex, StableDiGraph},
    unionfind::UnionFind,
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences, Topo, Walker},
    Direction::{Incoming, Outgoing},
};

use crate::{
    scaffold::{ScaffoldEdge, ScaffoldNode, StableScaffold},
    Height, Rewrite0,
};

/// Trait for objects which have associated coordinates in `C`.
pub(crate) trait Cartesian<C: Copy> {
    /// Return the coordinate of this object.
    fn coordinate(&self) -> &[C];
}

impl<C: Copy> Cartesian<C> for Vec<C> {
    fn coordinate(&self) -> &[C] {
        self.as_slice()
    }
}

/// Helper function to unify two nodes within a stable graph.
///
/// # Panics
///
/// Panics if `graph` edges are not 0-rewrites.
///
/// # Errors
///
/// This function will return an error if two uncollapsible nodes are unified.
pub(crate) fn unify<N, E, Ix, RN, RE>(
    graph: &mut StableDiGraph<N, ScaffoldEdge<E>, Ix>,
    p: NodeIndex<Ix>,
    q: NodeIndex<Ix>,
    quotient: &mut UnionFind<NodeIndex<Ix>>,
    mut on_remove_node: RN,
    mut on_remove_edge: RE,
) -> anyhow::Result<()>
where
    N: Clone,
    E: Clone,
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
                    if <&Rewrite0>::try_from(&graph[existing].rewrite)
                        .expect("non 0-rewrite passed to collapse unify")
                        .label()
                        != <&Rewrite0>::try_from(&graph[e].rewrite)
                            .expect("non 0-rewrite passed to collapse unify")
                            .label()
                    {
                        return Err(anyhow!("label inconsistency"));
                    }
                } else {
                    graph.add_edge(keep, target, graph[e].clone());
                }
                graph.remove_edge(e);
                on_remove_edge(e);
            }
            Quotient::RetargetTarget(keep, source, e) => {
                if let Some(existing) = graph.find_edge(source, keep) {
                    if <&Rewrite0>::try_from(&graph[existing].rewrite)
                        .expect("non 0-rewrite passed to collapse unify")
                        .label()
                        != <&Rewrite0>::try_from(&graph[e].rewrite)
                            .expect("non 0-rewrite passed to collapse unify")
                            .label()
                    {
                        return Err(anyhow!("label inconsistency"));
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

/// Given a **stable** `graph` of 0-diagrams and 0-rewrites, reduce the graph along the
/// *collapsibility* relation, and return the equivalence class on node indices of the induced
/// relation as a [`UnionFind`]. An edge is collapsible exactly when:
/// 1. it is an identity 0-rewrite;
/// 2. all composable triangles formed with this identity 0-rewrite agree label-wise in the other
///    two components.
///
/// # Panics
///
/// Panics if `graph` edges are not 0-rewrites.
pub(crate) fn collapse<V: Clone + Cartesian<Height>, E: Clone, Ix: IndexType>(
    graph: &mut StableScaffold<V, E, Ix>,
) -> UnionFind<NodeIndex<Ix>> {
    // invariant: #nodes of graph = #equivalence classes of union_find
    let mut union_find = UnionFind::new(graph.node_count());
    // tree tracks which edges descended from other edges by graph explosion
    // collapse subproblems need to be solved in topological order, with the root being the final one
    declare_idx! { struct TreeIx = DefaultIx; }
    let tree = {
        let mut tree: DiGraph<_, _, TreeIx> = Default::default();
        let root = tree.add_node((None, OnceCell::new()));
        for (ix, ScaffoldNode { key, .. }) in graph.node_references() {
            let mut cur = root;
            for &c in key.coordinate() {
                if let Some(existing) = tree
                    .neighbors_directed(cur, Incoming)
                    .find(|n| tree[*n].0 == Some(c))
                {
                    cur = existing;
                } else {
                    let next = tree.add_node((Some(c), OnceCell::new()));
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
        }
        let mut quotient: Vec<_> = Default::default();
        // find collapsible edges wrt nodes
        for e in graph.edge_references().filter(|e| {
            // e is contained within nodes
            nodes.contains(&e.source()) && nodes.contains(&e.target())
            // e is an identity rewrite
            && <&Rewrite0>::try_from(&e.weight().rewrite).unwrap().0.as_ref().map_or(true, |(s, t, _)| s.id == t.id)
            // check triangles within nodes which might refute collapsibility of e
            && graph.edges_directed(e.source(), Incoming).all(|p| {
                if let Some(c) = graph.find_edge(p.source(), e.target()) {
                    <&Rewrite0>::try_from(&p.weight().rewrite).unwrap().label() == <&Rewrite0>::try_from(&graph.edge_weight(c).unwrap().rewrite).unwrap().label()
                } else {
                    true
                }
            })
            && graph.edges_directed(e.target(), Outgoing).all(|n| {
                if let Some(c) = graph.find_edge(e.source(), n.target()) {
                    <&Rewrite0>::try_from(&n.weight().rewrite).unwrap().label() == <&Rewrite0>::try_from(&graph.edge_weight(c).unwrap().rewrite).unwrap().label()
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
                graph,
                s,
                t,
                &mut union_find,
                |rn| {
                    nodes.retain(|&n| n != rn);
                },
                |_re| (),
            )
            .expect("non-determinism in collapse; edge ({s}, {t}) has become uncollapsible");
        }
        tree[n]
            .1
            .set(nodes)
            .expect("failed to propagate collapse subproblem");
    }
    // check the tree of collapse subproblems has been completed
    debug_assert!(tree[NodeIndex::new(0)].1.get().is_some());
    union_find
}
