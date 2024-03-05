use std::ops::Range;

use homotopy_common::idx::{Idx, IdxVec};
use petgraph::{
    data::{Create, DataMap},
    graph::{DefaultIx, DiGraph},
    stable_graph::StableDiGraph,
    visit::{
        Data, EdgeCount, EdgeRef, GraphBase, IntoEdgeReferences, IntoNodeReferences, NodeCount,
        NodeRef,
    },
};

use crate::{
    common::{Boundary, DimensionError, Height, RegularHeight, SingularHeight, SliceIndex},
    diagram::{Diagram, DiagramN},
    rewrite::{Rewrite, RewriteN},
    Direction,
};

/// Weighted graph where each node has an associated [`Diagram`] and each edge has an associated [`Rewrite`].
pub trait ScaffoldGraph:
    Data<NodeWeight = ScaffoldNode<Self::V>, EdgeWeight = ScaffoldEdge<Self::E>>
{
    type V;
    type E;
}

impl<G, V, E> ScaffoldGraph for G
where
    G: Data<NodeWeight = ScaffoldNode<V>, EdgeWeight = ScaffoldEdge<E>>,
{
    type V = V;
    type E = E;
}

/// The output of explosion consists of the exploded graph and some maps from the original graph into the exploded graph.
#[derive(Clone, Debug)]
pub struct ExplosionOutput<G, O>
where
    G: GraphBase,
    O: GraphBase,
{
    pub scaffold: O,
    pub node_to_nodes: IdxVec<G::NodeId, Vec<O::NodeId>>,
    pub node_to_edges: IdxVec<G::NodeId, Vec<O::EdgeId>>,
    pub edge_to_edges: IdxVec<G::EdgeId, Vec<O::EdgeId>>,
}

pub trait Explodable<O: ScaffoldGraph>: ScaffoldGraph + Sized {
    fn explode(
        &self,
        node_map: impl FnMut(Self::NodeId, &Self::V, SliceIndex) -> Option<O::V>,
        internal_edge_map: impl FnMut(Self::NodeId, &Self::V, InternalRewrite) -> Option<O::E>,
        external_edge_map: impl FnMut(Self::EdgeId, &Self::E, ExternalRewrite) -> Option<O::E>,
    ) -> Result<ExplosionOutput<Self, O>, DimensionError>;

    #[inline]
    fn explode_graph(
        &self,
        node_map: impl FnMut(Self::NodeId, &Self::V, SliceIndex) -> Option<O::V>,
        internal_edge_map: impl FnMut(Self::NodeId, &Self::V, InternalRewrite) -> Option<O::E>,
        external_edge_map: impl FnMut(Self::EdgeId, &Self::E, ExternalRewrite) -> Option<O::E>,
    ) -> Result<O, DimensionError> {
        self.explode(node_map, internal_edge_map, external_edge_map)
            .map(|output| output.scaffold)
    }
}

/// Weighted graph node with associated [`Diagram`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScaffoldNode<V> {
    pub key: V,
    pub diagram: Diagram,
}

impl<V> ScaffoldNode<V> {
    pub fn new(key: V, diagram: impl Into<Diagram>) -> Self {
        Self {
            key,
            diagram: diagram.into(),
        }
    }

    pub fn map<T>(self, mut f: impl FnMut(V) -> T) -> ScaffoldNode<T> {
        ScaffoldNode {
            key: f(self.key),
            diagram: self.diagram,
        }
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
    pub fn new(key: E, rewrite: impl Into<Rewrite>) -> Self {
        Self {
            key,
            rewrite: rewrite.into(),
        }
    }

    pub fn map<T>(self, mut f: impl FnMut(E) -> T) -> ScaffoldEdge<T> {
        ScaffoldEdge {
            key: f(self.key),
            rewrite: self.rewrite,
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
    #[must_use]
    pub const fn direction(self) -> Direction {
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
    #[must_use]
    pub const fn is_atomic(self) -> bool {
        !matches!(self, Self::RegularSlice { .. })
    }

    #[must_use]
    pub const fn is_flange(self) -> bool {
        match self {
            Self::RegularSlice { flange, .. } => flange,
            _ => false,
        }
    }
}

impl<G, O> Explodable<O> for G
where
    G: ScaffoldGraph + DataMap + NodeCount + EdgeCount,
    O: ScaffoldGraph + Create,
    for<'a> &'a G: IntoNodeReferences<NodeId = G::NodeId, NodeWeight = G::NodeWeight>
        + IntoEdgeReferences<EdgeId = G::EdgeId, EdgeWeight = G::EdgeWeight>,
    G::NodeId: Idx,
    G::EdgeId: Idx,
{
    /// Explodes a scaffold to obtain the scaffold one dimension higher.
    fn explode(
        &self,
        mut node_map: impl FnMut(G::NodeId, &G::V, SliceIndex) -> Option<O::V>,
        mut internal_edge_map: impl FnMut(G::NodeId, &G::V, InternalRewrite) -> Option<O::E>,
        mut external_edge_map: impl FnMut(G::EdgeId, &G::E, ExternalRewrite) -> Option<O::E>,
    ) -> Result<ExplosionOutput<G, O>, DimensionError> {
        let mut graph = O::default();

        let mut nodes = IdxVec::splat(vec![], self.node_count());
        let mut internal_edges = IdxVec::splat(vec![], self.node_count());
        let mut external_edges = IdxVec::splat(vec![], self.edge_count());

        for n in self.node_references() {
            let key = &n.weight().key;
            let diagram: &DiagramN = (&n.weight().diagram).try_into()?;

            let mut add_node = |si: SliceIndex, slice: Diagram| {
                nodes[n.id()].push(|| -> Option<_> {
                    let key = node_map(n.id(), key, si)?;
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
                    internal_edges[n.id()].push(|| -> Option<_> {
                        let a = nodes[n.id()][si]?;
                        let b = nodes[n.id()][ti]?;
                        let key = internal_edge_map(n.id(), key, r)?;
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
            let target_diagram: &DiagramN = (&self.node_weight(t).unwrap().diagram).try_into()?;

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
                        let cone = rewrite.cone_over_target(target_height).left();
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
                                    cone.map_or_else(
                                        || Rewrite::identity(rewrite.dimension() - 1),
                                        |c| c.singular_slices()[source_height - start].clone(),
                                    ),
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

impl Diagram {
    /// Construct the fully exploded scaffold of the diagram.
    #[must_use]
    pub fn fully_explode<G>(self) -> G
    where
        G: Create + Explodable<G>,
        G::V: Clone + Default + Extend<Height>,
        G::E: Default,
    {
        let mut scaffold = G::default();
        let dimension = self.dimension();
        scaffold.add_node(self.into());
        for _ in 0..dimension {
            scaffold = scaffold
                .explode_graph(
                    |_, key, si| match si {
                        SliceIndex::Boundary(_) => None,
                        SliceIndex::Interior(h) => {
                            let mut key = key.to_owned();
                            key.extend(std::iter::once(h));
                            Some(key)
                        }
                    },
                    |_, _, _| Some(Default::default()),
                    |_, _, _| Some(Default::default()),
                )
                .unwrap();
        }
        scaffold
    }
}
