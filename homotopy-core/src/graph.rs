use std::ops::Range;

use homotopy_common::idx::IdxVec;
use petgraph::{
    graph::{DefaultIx, DiGraph, EdgeIndex, IndexType, NodeIndex},
    visit::{EdgeRef, IntoNodeReferences},
};

use crate::{
    common::{DimensionError, RegularHeight, SingularHeight},
    Boundary, Diagram, DiagramN, Direction, Height, Rewrite, RewriteN, SliceIndex,
};

/// The output of explosion consists of the exploded graph and some maps from the original graph into the exploded graph.
#[derive(Clone, Debug)]
pub struct ExplosionOutput<V, E, Ix1, Ix2>
where
    Ix1: IndexType,
    Ix2: IndexType,
{
    pub output: SliceGraph<V, E, Ix2>,
    pub node_to_nodes: IdxVec<NodeIndex<Ix1>, Vec<NodeIndex<Ix2>>>,
    pub node_to_edges: IdxVec<NodeIndex<Ix1>, Vec<EdgeIndex<Ix2>>>,
    pub edge_to_edges: IdxVec<EdgeIndex<Ix1>, Vec<EdgeIndex<Ix2>>>,
}

pub trait Explodable<V, E, Ix>
where
    Ix: IndexType,
{
    fn singleton<D>(key: V, diagram: D) -> Self
    where
        D: Into<Diagram>;

    fn explode<F, G, H, V2, E2, Ix2>(
        &self,
        node_map: F,
        internal_edge_map: G,
        external_edge_map: H,
    ) -> Result<ExplosionOutput<V2, E2, Ix, Ix2>, DimensionError>
    where
        Ix2: IndexType,
        F: FnMut(NodeIndex<Ix>, &V, SliceIndex) -> Option<V2>,
        G: FnMut(NodeIndex<Ix>, &V, RewriteOrigin) -> Option<E2>,
        H: FnMut(EdgeIndex<Ix>, &E, RewriteOrigin) -> Option<E2>;
}

/// A graph of diagrams and rewrites obtained by exploding a diagram.
pub type SliceGraph<V = (), E = (), Ix = DefaultIx> = DiGraph<(V, Diagram), (E, Rewrite), Ix>;

/// Describes from where a rewrite in the output of explosion originates.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RewriteOrigin {
    /// Padded identity along boundary.
    Boundary(Boundary),
    /// From a diagram's cospans.
    Internal(SingularHeight, Direction),
    /// Sparse identity from a rewrite.
    Sparse(RegularHeight),
    /// Unit slices from a rewrite.
    UnitSlice,
    /// Cone regular slices from a rewrite.
    RegularSlice,
    /// Cone singular slices from a rewrite.
    SingularSlice(SingularHeight),
}

impl<V, E, Ix> Explodable<V, E, Ix> for SliceGraph<V, E, Ix>
where
    Ix: IndexType,
{
    /// Creates a new 0-dimensional slice graph.
    fn singleton<D>(key: V, diagram: D) -> Self
    where
        D: Into<Diagram>,
    {
        let mut graph = Self::default();
        graph.add_node((key, diagram.into()));
        graph
    }

    /// Explodes a slice graph to obtain the slice graph one dimension higher.
    fn explode<F, G, H, V2, E2, Ix2>(
        &self,
        mut node_map: F,
        mut internal_edge_map: G,
        mut external_edge_map: H,
    ) -> Result<ExplosionOutput<V2, E2, Ix, Ix2>, DimensionError>
    where
        Ix2: IndexType,
        F: FnMut(NodeIndex<Ix>, &V, SliceIndex) -> Option<V2>,
        G: FnMut(NodeIndex<Ix>, &V, RewriteOrigin) -> Option<E2>,
        H: FnMut(EdgeIndex<Ix>, &E, RewriteOrigin) -> Option<E2>,
    {
        let mut graph = SliceGraph::default();

        let mut nodes = IdxVec::splat(vec![], self.node_count());
        let mut internal_edges = IdxVec::splat(vec![], self.node_count());
        let mut external_edges = IdxVec::splat(vec![], self.edge_count());

        for (n, (key, diagram)) in self.node_references() {
            let diagram: &DiagramN = diagram.try_into()?;

            let mut add_node = |si: SliceIndex, slice: Diagram| {
                nodes[n].push(|| -> Option<_> {
                    let key = node_map(n, key, si)?;
                    graph.add_node((key, slice)).into()
                }());
            };

            // Source slice
            add_node(Boundary::Source.into(), diagram.source());

            // Interior slices
            for (i, slice) in diagram.slices().enumerate() {
                add_node(Height::from(i).into(), slice);
            }

            // Target slice
            add_node(Boundary::Target.into(), diagram.target());

            let mut add_edge =
                |si: SliceIndex, ti: SliceIndex, ro: RewriteOrigin, rewrite: Rewrite| {
                    let si = (si.to_int(diagram.size()) + 1) as usize;
                    let ti = (ti.to_int(diagram.size()) + 1) as usize;
                    internal_edges[n].push(|| -> Option<_> {
                        let a = nodes[n][si]?;
                        let b = nodes[n][ti]?;
                        let key = internal_edge_map(n, key, ro)?;
                        graph.add_edge(a, b, (key, rewrite)).into()
                    }());
                };

            // Identity rewrite from source slice
            add_edge(
                Boundary::Source.into(),
                Height::Regular(0).into(),
                RewriteOrigin::Boundary(Boundary::Source),
                Rewrite::identity(diagram.dimension() - 1),
            );

            // Rewrites between interior slices
            for (i, cospan) in diagram.cospans().iter().enumerate() {
                add_edge(
                    Height::Regular(i).into(),
                    Height::Singular(i).into(),
                    RewriteOrigin::Internal(i, Direction::Forward),
                    cospan.forward.clone(),
                );
                add_edge(
                    Height::Regular(i + 1).into(),
                    Height::Singular(i).into(),
                    RewriteOrigin::Internal(i, Direction::Backward),
                    cospan.backward.clone(),
                );
            }

            // Identity rewrite from target slice
            add_edge(
                Boundary::Target.into(),
                Height::Regular(diagram.size()).into(),
                RewriteOrigin::Boundary(Boundary::Target),
                Rewrite::identity(diagram.dimension() - 1),
            );
        }

        for e in self.edge_references() {
            let s = e.source();
            let t = e.target();

            let key = &e.weight().0;
            let rewrite: &RewriteN = (&e.weight().1).try_into()?;
            let source_diagram: &DiagramN = (&self[s].1).try_into()?;
            let target_diagram: &DiagramN = (&self[t].1).try_into()?;

            let mut add_edge =
                |si: SliceIndex, ti: SliceIndex, ro: RewriteOrigin, rewrite: Rewrite| {
                    let si = (si.to_int(source_diagram.size()) + 1) as usize;
                    let ti = (ti.to_int(target_diagram.size()) + 1) as usize;
                    external_edges[e.id()].push(|| -> Option<_> {
                        let a = nodes[s][si]?;
                        let b = nodes[t][ti]?;
                        let key = external_edge_map(e.id(), key, ro)?;
                        graph.add_edge(a, b, (key, rewrite)).into()
                    }());
                };

            for ti in SliceIndex::for_size(target_diagram.size()) {
                match ti {
                    // Boundary identity
                    SliceIndex::Boundary(b) => {
                        add_edge(
                            ti,
                            ti,
                            RewriteOrigin::Boundary(b),
                            Rewrite::identity(rewrite.dimension() - 1),
                        );
                    }
                    // Sparse identity
                    SliceIndex::Interior(Height::Regular(target_height)) => {
                        let source_height = rewrite.regular_image(target_height);
                        add_edge(
                            SliceIndex::Interior(Height::Regular(source_height)),
                            ti,
                            RewriteOrigin::Sparse(target_height),
                            Rewrite::identity(rewrite.dimension() - 1),
                        );
                    }
                    // Unit, regular, and singular slices.
                    SliceIndex::Interior(Height::Singular(target_height)) => {
                        let Range { start, end } = rewrite.singular_preimage(target_height);

                        for source_height in start..end {
                            let singular_slice = rewrite.slice(source_height);

                            let ro = if source_height == start {
                                RewriteOrigin::UnitSlice
                            } else {
                                RewriteOrigin::RegularSlice
                            };
                            add_edge(
                                SliceIndex::Interior(Height::Regular(source_height)),
                                ti,
                                ro,
                                source_diagram.cospans()[source_height]
                                    .forward
                                    .compose(&singular_slice)
                                    .unwrap(),
                            );

                            add_edge(
                                SliceIndex::Interior(Height::Singular(source_height)),
                                ti,
                                RewriteOrigin::SingularSlice(source_height),
                                singular_slice,
                            );
                        }

                        let ro = if start < end {
                            RewriteOrigin::UnitSlice
                        } else {
                            RewriteOrigin::RegularSlice
                        };
                        add_edge(
                            SliceIndex::Interior(Height::Regular(end)),
                            ti,
                            ro,
                            target_diagram.cospans()[target_height].backward.clone(),
                        );
                    }
                }
            }
        }

        Ok(ExplosionOutput {
            output: graph,
            node_to_nodes: nodes.map(|ns| ns.into_iter().flatten().collect()),
            node_to_edges: internal_edges.map(|es| es.into_iter().flatten().collect()),
            edge_to_edges: external_edges.map(|es| es.into_iter().flatten().collect()),
        })
    }
}

impl<V, E, Ix1, Ix2> Explodable<V, E, Ix2> for ExplosionOutput<V, E, Ix1, Ix2>
where
    Ix1: IndexType,
    Ix2: IndexType,
{
    fn singleton<D>(key: V, diagram: D) -> Self
    where
        D: Into<Diagram>,
    {
        Self {
            output: SliceGraph::singleton(key, diagram),
            node_to_nodes: Default::default(),
            node_to_edges: Default::default(),
            edge_to_edges: Default::default(),
        }
    }

    fn explode<F, G, H, V2, E2, Ix3>(
        &self,
        node_map: F,
        internal_edge_map: G,
        external_edge_map: H,
    ) -> Result<ExplosionOutput<V2, E2, Ix2, Ix3>, DimensionError>
    where
        Ix3: IndexType,
        F: FnMut(NodeIndex<Ix2>, &V, SliceIndex) -> Option<V2>,
        G: FnMut(NodeIndex<Ix2>, &V, RewriteOrigin) -> Option<E2>,
        H: FnMut(EdgeIndex<Ix2>, &E, RewriteOrigin) -> Option<E2>,
    {
        self.output
            .explode(node_map, internal_edge_map, external_edge_map)
    }
}
