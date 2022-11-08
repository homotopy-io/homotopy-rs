use std::ops::{Index, Range};

use homotopy_common::idx::{Idx, IdxVec};
use petgraph::{
    data::Build,
    graph::{DefaultIx, DiGraph},
    stable_graph::StableDiGraph,
    visit::{
        Data, EdgeCount, EdgeRef, GraphBase, IntoEdgeReferences, IntoNodeReferences, NodeCount,
    },
};

use crate::{
    common::{Boundary, DimensionError, Height, RegularHeight, SingularHeight, SliceIndex},
    diagram::{Diagram, DiagramN},
    rewrite::{Rewrite, RewriteN},
    Diagram0, Direction,
};

/// Weighted graph where each node has an associated [`Diagram`] and each edge has an associated
/// [`Rewrite`].
pub trait ScaffoldGraph:
    Data<NodeWeight = ScaffoldNode<Self::NodeKey>, EdgeWeight = ScaffoldEdge<Self::EdgeKey>>
{
    type NodeKey;
    type EdgeKey;
}

impl<G, V, E> ScaffoldGraph for G
where
    G: Data<NodeWeight = ScaffoldNode<V>, EdgeWeight = ScaffoldEdge<E>>,
{
    type NodeKey = V;
    type EdgeKey = E;
}

/// The output of explosion consists of the exploded graph and some maps from the original graph into the exploded graph.
#[derive(Clone, Debug)]
pub struct ExplosionOutput<G, G2>
where
    G: GraphBase,
    G2: GraphBase,
{
    pub scaffold: G2,
    pub node_to_nodes: IdxVec<G::NodeId, Vec<G2::NodeId>>,
    pub node_to_edges: IdxVec<G::NodeId, Vec<G2::EdgeId>>,
    pub edge_to_edges: IdxVec<G::EdgeId, Vec<G2::EdgeId>>,
}

pub trait Explodable<'a, G>
where
    G: ScaffoldGraph,
{
    fn explode<G2, NodeMap, InternalEdgeMap, ExternalEdgeMap>(
        &'a self,
        node_map: NodeMap,
        internal_edge_map: InternalEdgeMap,
        external_edge_map: ExternalEdgeMap,
    ) -> Result<ExplosionOutput<G, G2>, DimensionError>
    where
        G2: Default + Build + ScaffoldGraph,
        NodeMap: FnMut(G::NodeId, &G::NodeKey, SliceIndex) -> Option<G2::NodeKey>,
        InternalEdgeMap: FnMut(G::NodeId, &G::NodeKey, InternalRewrite) -> Option<G2::EdgeKey>,
        ExternalEdgeMap: FnMut(G::EdgeId, &G::EdgeKey, ExternalRewrite) -> Option<G2::EdgeKey>;

    #[inline]
    fn explode_simple<G2, NodeMap, InternalEdgeMap, ExternalEdgeMap>(
        &'a self,
        node_map: NodeMap,
        internal_edge_map: InternalEdgeMap,
        external_edge_map: ExternalEdgeMap,
    ) -> Result<G2, DimensionError>
    where
        G2: Default + Build + ScaffoldGraph,
        NodeMap: FnMut(G::NodeId, &G::NodeKey, SliceIndex) -> Option<G2::NodeKey>,
        InternalEdgeMap: FnMut(G::NodeId, &G::NodeKey, InternalRewrite) -> Option<G2::EdgeKey>,
        ExternalEdgeMap: FnMut(G::EdgeId, &G::EdgeKey, ExternalRewrite) -> Option<G2::EdgeKey>,
    {
        self.explode(node_map, internal_edge_map, external_edge_map)
            .map(|output: ExplosionOutput<G, G2>| output.scaffold)
    }
}

/// Weighted graph node with associated [`Diagram`].
#[derive(Clone, Debug)]
pub struct ScaffoldNode<V> {
    pub key: V,
    pub diagram: Diagram,
}

impl<V> ScaffoldNode<V> {
    pub fn new<D>(key: V, diagram: D) -> Self
    where
        D: Into<Diagram>,
    {
        Self {
            key,
            diagram: diagram.into(),
        }
    }
}

impl<V: Extend<V>> Extend<ScaffoldNode<V>> for ScaffoldNode<V> {
    fn extend<T: IntoIterator<Item = ScaffoldNode<V>>>(&mut self, iter: T) {
        self.key
            .extend(iter.into_iter().map(|ScaffoldNode { key, diagram }| {
                debug_assert_eq!(
                    Diagram0::try_from(&self.diagram)
                        .expect("extended non-fully exploded scaffold")
                        .generator,
                    Diagram0::try_from(&diagram)
                        .expect("extended non-fully exploded scaffold")
                        .generator,
                );
                key
            }));
    }
}

impl<V: Default> From<Diagram> for ScaffoldNode<V> {
    fn from(diagram: Diagram) -> Self {
        Self::new(V::default(), diagram)
    }
}

/// Weighted graph edge with associated [`Rewrite`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScaffoldEdge<E> {
    pub key: E,
    pub rewrite: Rewrite,
}

impl<E> ScaffoldEdge<E> {
    pub fn new<R>(key: E, rewrite: R) -> Self
    where
        R: Into<Rewrite>,
    {
        Self {
            key,
            rewrite: rewrite.into(),
        }
    }
}

impl<E: Default> From<Rewrite> for ScaffoldEdge<E> {
    fn from(rewrite: Rewrite) -> Self {
        Self::new(E::default(), rewrite)
    }
}

/// A graph of diagrams and rewrites obtained by exploding a diagram.
pub type Scaffold<V = (), E = (), Ix = DefaultIx> = DiGraph<ScaffoldNode<V>, ScaffoldEdge<E>, Ix>;
pub(crate) type StableScaffold<V = (), E = (), Ix = DefaultIx> =
    StableDiGraph<ScaffoldNode<V>, ScaffoldEdge<E>, Ix>;

/// Describes from where a rewrite in the output of explosion originates.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InternalRewrite {
    /// Padded identity along boundary.
    Boundary(Boundary),
    /// From a diagram's cospans.
    Interior(SingularHeight, Direction),
}

impl InternalRewrite {
    pub fn direction(self) -> Direction {
        use Boundary::{Source, Target};
        match self {
            Self::Boundary(Source) => Direction::Forward,
            Self::Boundary(Target) => Direction::Backward,
            Self::Interior(_, direction) => direction,
        }
    }
}

/// Describes from where a rewrite in the output of explosion originates.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExternalRewrite {
    /// Padded identity along boundary.
    Boundary(Boundary),
    /// Sparse identity between regular heights.
    Sparse(RegularHeight),
    /// Regular slice going from a regular height to a singular height.
    RegularSlice {
        source_height: RegularHeight,
        target_height: SingularHeight,
        flange: bool,
    },
    /// Singular slice going from a singular height to a singular height.
    SingularSlice {
        source_height: SingularHeight,
        target_height: SingularHeight,
    },
}

impl ExternalRewrite {
    pub fn is_atomic(self) -> bool {
        !matches!(self, Self::RegularSlice { .. })
    }

    pub fn is_flange(self) -> bool {
        match self {
            Self::RegularSlice { flange, .. } => flange,
            _ => false,
        }
    }
}

impl<'a, G> Explodable<'a, G> for G
where
    G: 'a + ScaffoldGraph + NodeCount + EdgeCount + Index<G::NodeId, Output = G::NodeWeight>,
    &'a G: GraphBase<NodeId = G::NodeId, EdgeId = G::EdgeId>
        + IntoNodeReferences<NodeRef = (G::NodeId, &'a G::NodeWeight)>
        + IntoEdgeReferences<EdgeWeight = G::EdgeWeight>,
    G::NodeId: Idx,
    G::EdgeId: Idx,
{
    /// Explodes a scaffold to obtain the scaffold one dimension higher.
    fn explode<G2, NodeMap, InternalEdgeMap, ExternalEdgeMap>(
        &'a self,
        mut node_map: NodeMap,
        mut internal_edge_map: InternalEdgeMap,
        mut external_edge_map: ExternalEdgeMap,
    ) -> Result<ExplosionOutput<G, G2>, DimensionError>
    where
        G2: Default + Build + ScaffoldGraph,
        NodeMap: FnMut(G::NodeId, &G::NodeKey, SliceIndex) -> Option<G2::NodeKey>,
        InternalEdgeMap: FnMut(G::NodeId, &G::NodeKey, InternalRewrite) -> Option<G2::EdgeKey>,
        ExternalEdgeMap: FnMut(G::EdgeId, &G::EdgeKey, ExternalRewrite) -> Option<G2::EdgeKey>,
    {
        let mut graph: G2 = Default::default();

        let mut nodes = IdxVec::splat(vec![], self.node_count());
        let mut internal_edges = IdxVec::splat(vec![], self.node_count());
        let mut external_edges = IdxVec::splat(vec![], self.edge_count());

        for (n, node) in self.node_references() {
            let key = &node.key;
            let diagram: &DiagramN = (&node.diagram).try_into()?;

            let mut add_node = |si: SliceIndex, slice: Diagram| {
                nodes[n].push(|| -> Option<_> {
                    let key = node_map(n, key, si)?;
                    graph.add_node(ScaffoldNode::new(key, slice)).into()
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
                |si: SliceIndex, ti: SliceIndex, r: InternalRewrite, rewrite: Rewrite| {
                    internal_edges[n].push(|| -> Option<_> {
                        let a = nodes[n][si]?;
                        let b = nodes[n][ti]?;
                        let key = internal_edge_map(n, key, r)?;
                        graph.add_edge(a, b, ScaffoldEdge::new(key, rewrite))
                    }());
                };

            // Identity rewrite from source slice
            add_edge(
                Boundary::Source.into(),
                Height::Regular(0).into(),
                InternalRewrite::Boundary(Boundary::Source),
                Rewrite::identity(diagram.dimension() - 1),
            );

            // Rewrites between interior slices
            for (i, cospan) in diagram.cospans().iter().enumerate() {
                add_edge(
                    Height::Regular(i).into(),
                    Height::Singular(i).into(),
                    InternalRewrite::Interior(i, Direction::Forward),
                    cospan.forward.clone(),
                );
                add_edge(
                    Height::Regular(i + 1).into(),
                    Height::Singular(i).into(),
                    InternalRewrite::Interior(i, Direction::Backward),
                    cospan.backward.clone(),
                );
            }

            // Identity rewrite from target slice
            add_edge(
                Boundary::Target.into(),
                Height::Regular(diagram.size()).into(),
                InternalRewrite::Boundary(Boundary::Target),
                Rewrite::identity(diagram.dimension() - 1),
            );
        }

        for e in self.edge_references() {
            let s = e.source();
            let t = e.target();

            let key = &e.weight().key;
            let rewrite: &RewriteN = (&e.weight().rewrite).try_into()?;
            let target_diagram: &DiagramN = (&self[t].diagram).try_into()?;

            let mut add_edge =
                |si: SliceIndex, ti: SliceIndex, r: ExternalRewrite, rewrite: Rewrite| {
                    external_edges[e.id()].push(|| -> Option<_> {
                        let a = nodes[s][si]?;
                        let b = nodes[t][ti]?;
                        let key = external_edge_map(e.id(), key, r)?;
                        graph.add_edge(a, b, ScaffoldEdge::new(key, rewrite))
                    }());
                };

            for ti in SliceIndex::for_size(target_diagram.size()) {
                match ti {
                    // Boundary identity
                    SliceIndex::Boundary(b) => {
                        add_edge(
                            ti,
                            ti,
                            ExternalRewrite::Boundary(b),
                            Rewrite::identity(rewrite.dimension() - 1),
                        );
                    }
                    // Sparse identity
                    SliceIndex::Interior(Height::Regular(target_height)) => {
                        let source_height = rewrite.regular_image(target_height);
                        add_edge(
                            SliceIndex::Interior(Height::Regular(source_height)),
                            ti,
                            ExternalRewrite::Sparse(target_height),
                            Rewrite::identity(rewrite.dimension() - 1),
                        );
                    }
                    // Unit, regular, and singular slices.
                    SliceIndex::Interior(Height::Singular(target_height)) => {
                        let cone = rewrite.cone_over_target(target_height);
                        let preimage = rewrite.singular_preimage(target_height);

                        if preimage.is_empty() {
                            let start = preimage.start;
                            // unit slice
                            add_edge(
                                SliceIndex::Interior(Height::Regular(start)),
                                ti,
                                ExternalRewrite::RegularSlice {
                                    source_height: start,
                                    target_height,
                                    flange: false,
                                },
                                cone.unwrap().regular_slices()[0].clone(),
                            );
                        } else {
                            let Range { start, end } = preimage;
                            // add first flange slice
                            add_edge(
                                SliceIndex::Interior(Height::Regular(start)),
                                ti,
                                ExternalRewrite::RegularSlice {
                                    source_height: start,
                                    target_height,
                                    flange: true,
                                },
                                cone.map_or(
                                    target_diagram.cospans()[target_height].forward.clone(),
                                    |c| c.regular_slices()[0].clone(),
                                ),
                            );
                            // add singular singular slice, then regular slice, …
                            for source_height in start..end {
                                add_edge(
                                    SliceIndex::Interior(Height::Singular(source_height)),
                                    ti,
                                    ExternalRewrite::SingularSlice {
                                        source_height,
                                        target_height,
                                    },
                                    cone.map_or(Rewrite::identity(rewrite.dimension() - 1), |c| {
                                        c.singular_slices()[source_height - start].clone()
                                    }),
                                );
                                if source_height < end - 1 {
                                    // one regular slice between each adjacent pair of singular slices
                                    add_edge(
                                        SliceIndex::Interior(Height::Regular(source_height + 1)),
                                        ti,
                                        ExternalRewrite::RegularSlice {
                                            source_height: source_height + 1,
                                            target_height,
                                            flange: false,
                                        },
                                        cone.unwrap().regular_slices()[source_height - start + 1]
                                            .clone(),
                                    );
                                }
                            }
                            // add last flange slice
                            add_edge(
                                SliceIndex::Interior(Height::Regular(end)),
                                ti,
                                ExternalRewrite::RegularSlice {
                                    source_height: end,
                                    target_height,
                                    flange: true,
                                },
                                cone.map_or(
                                    target_diagram.cospans()[target_height].backward.clone(),
                                    |c| c.regular_slices()[end - start].clone(),
                                ),
                            );
                        }
                    }
                }
            }
        }

        Ok(ExplosionOutput {
            scaffold: graph,
            node_to_nodes: nodes.map(|ns| ns.into_iter().flatten().collect()),
            node_to_edges: internal_edges.map(|es| es.into_iter().flatten().collect()),
            edge_to_edges: external_edges.map(|es| es.into_iter().flatten().collect()),
        })
    }
}
