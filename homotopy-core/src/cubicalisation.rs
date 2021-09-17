use std::{
    cell::RefCell,
    cmp,
    convert::{From, Into, TryInto},
    iter::FromIterator,
};

use hashconsing::{HConsed, HConsign, HashConsign};
use homotopy_common::{declare_idx, idx::IdxVec};
use itertools::{Itertools, MultiProduct};
use petgraph::{graph::IndexType, unionfind::UnionFind};

use crate::{
    common::{DimensionError, Direction, Height, RegularHeight, SingularHeight},
    diagram::{Diagram, DiagramN},
    monotone::{compose, dual_inv, Monotone, MonotoneIterator},
    rewrite::{
        Composable, CompositionError, Cone, ConeInternal, Cospan, GenericCone, GenericCospan,
        GenericRewrite, GenericRewriteN, Rewrite, RewriteAllocator, RewriteInternal, RewriteN,
    },
    util::Hasher,
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Bias {
    Left,
    Right,
}

impl Bias {
    fn reverse(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

// Cubicalisation

impl Diagram {
    pub fn cubicalise(self, biases: &[Bias]) -> Result<CubicalGraph, DimensionError> {
        match self {
            Self::Diagram0(_) => {
                assert!(biases.is_empty());
                Ok(CubicalGraph::new(self))
            }
            Self::DiagramN(_) => {
                assert!(biases.len() < self.dimension());
                let mut graph = CubicalGraph::new(self);
                graph = graph.explode()?;
                for bias in biases {
                    graph = graph.parallelise(*bias).explode()?;
                }
                Ok(graph)
            }
        }
    }
}

// Explosion and parallelisation

impl CubicalGraph {
    fn explode(self) -> Result<Self, DimensionError> {
        use Height::{Regular, Singular};

        let original_nodes_len = self.nodes.len();

        let mut nodes_exploded: IdxVec<NodeId, Node> = IdxVec::new();
        let mut edges_exploded: IdxVec<EdgeId, Edge> = IdxVec::new();

        // For every node index in the original graph, we record the index in `nodes_exploded` at
        // which the slices of that node start. This allows us to find the index again when
        // constructing the edges of the exploded graph.
        let mut nodes_indices: Vec<usize> = Vec::new();

        for (_, node) in self.nodes {
            let nodes_index_start = nodes_exploded.len();
            nodes_indices.push(nodes_index_start);
            let diagram: DiagramN = node.diagram.try_into()?;

            // Interior slices
            for (i, slice) in diagram.slices().enumerate() {
                let slice_key = mk_coord(&node.key, node.heights[i]);
                let slice_coord = mk_coord(&node.coord, Height::from_int(i));
                let slice_heights = slice.heights();
                nodes_exploded.push(Node {
                    key: slice_key,
                    coord: slice_coord,
                    diagram: slice,
                    heights: slice_heights,
                    incoming_edges: vec![],
                    outgoing_edges: vec![],
                });
            }

            // Rewrites between interior slices
            for (i, cospan) in diagram.cospans().iter().enumerate() {
                let e = EdgeId(edges_exploded.len());
                let s = NodeId(nodes_index_start + Regular(i).to_int());
                let t = NodeId(nodes_index_start + Singular(i).to_int());
                edges_exploded.push(Edge {
                    source: s,
                    target: t,
                    rewrite: cospan.forward.clone().into(),
                });
                nodes_exploded[s].outgoing_edges.push(e);
                nodes_exploded[t].incoming_edges.push(e);

                let e = EdgeId(edges_exploded.len());
                let s = NodeId(nodes_index_start + Regular(i + 1).to_int());
                let t = NodeId(nodes_index_start + Singular(i).to_int());
                edges_exploded.push(Edge {
                    source: s,
                    target: t,
                    rewrite: cospan.backward.clone().into(),
                });
                nodes_exploded[s].outgoing_edges.push(e);
                nodes_exploded[t].incoming_edges.push(e);
            }
        }

        // We push a final index so that the length of any node's contribution to `nodes_exploded`
        // can be computed by subtracting the node's index from that of the next node.
        nodes_indices.push(nodes_exploded.len());

        for (_, edge) in self.edges {
            let source = edge.source.0;
            let target = edge.target.0;
            let rewrite: CubicalRewriteN = edge.rewrite.try_into()?;

            let source_index = nodes_indices[source];
            let source_size = nodes_indices[source + 1] - source_index;
            let target_index = nodes_indices[target];
            let target_size = nodes_indices[target + 1] - target_index;

            // Singular slices
            for source_height in 0..(source_size - 1) / 2 {
                let target_height = rewrite.singular_image(source_height);
                let e = EdgeId(edges_exploded.len());
                let s = NodeId(source_index + Singular(source_height).to_int());
                let t = NodeId(target_index + Singular(target_height).to_int());
                edges_exploded.push(Edge {
                    source: s,
                    target: t,
                    rewrite: rewrite.singular_slice(source_height),
                });
                nodes_exploded[s].outgoing_edges.push(e);
                nodes_exploded[t].incoming_edges.push(e);
            }

            // Regular slices
            for target_height in 0..(target_size + 1) / 2 {
                let source_height = rewrite.regular_image(target_height);
                let e = EdgeId(edges_exploded.len());
                let s = NodeId(source_index + Regular(source_height).to_int());
                let t = NodeId(target_index + Regular(target_height).to_int());
                edges_exploded.push(Edge {
                    source: s,
                    target: t,
                    rewrite: rewrite.regular_slice(target_height),
                });
                nodes_exploded[s].outgoing_edges.push(e);
                nodes_exploded[t].incoming_edges.push(e);
            }
        }

        let mut sizes = self.sizes;
        sizes.push((nodes_exploded.len() / original_nodes_len - 1) / 2);

        Ok(Self {
            sizes,
            nodes: nodes_exploded,
            edges: edges_exploded,
        })
    }

    fn parallelise(&self, bias: Bias) -> Self {
        if let Some(res) = self.parallelise_directed(bias, Direction::Forward) {
            return res;
        }
        if let Some(res) = self.parallelise_directed(bias, Direction::Backward) {
            return res;
        }

        panic!("Failed to parallelise");
    }

    fn parallelise_directed(&self, bias: Bias, direction: Direction) -> Option<Self> {
        if self.edges.values().all(|edge| edge.rewrite.is_parallel()) {
            Some(self.clone())
        } else {
            match direction {
                Direction::Forward => {
                    for x in SingularExpansion::new(self.clone(), bias).ok()? {
                        if let Some(res) = x.parallelise_directed(bias, Direction::Backward) {
                            return Some(res);
                        }
                    }
                    None
                }
                Direction::Backward => {
                    for x in RegularExpansion::new(self.clone(), bias).ok()? {
                        if let Some(res) = x.parallelise_directed(bias, Direction::Forward) {
                            return Some(res);
                        }
                    }
                    None
                }
            }
        }
    }
}

// Singular expansion

struct SingularExpansion {
    graph: CubicalGraph,
    weights: IdxVec<NodeId, Vec<usize>>, // weights[i][j] = weight of singular slice j of node i
    indices: IdxVec<EdgeId, usize>,      // indices[e] = index of edge e in the product below
    product: MultiProduct<BiasedMonotoneIterator>,
}

impl SingularExpansion {
    fn new(graph: CubicalGraph, bias: Bias) -> Result<Self, DimensionError> {
        // Assign weights to the singular slices of every node.
        let mut weights: IdxVec<NodeId, Vec<usize>> =
            IdxVec::from_iter(vec![vec![]; graph.nodes.len()]);
        for i in graph.topological_sort() {
            let diagram: &DiagramN = (&graph.nodes[i].diagram).try_into()?;

            // Start by assigning weight 1 to every singular slice.
            for _ in 0..diagram.size() {
                weights[i].push(1);
            }

            // Now propagate weights forwards along every incoming edge.
            for &e in &graph.nodes[i].incoming_edges {
                let source = graph.edges[e].source;
                let rewrite: &CubicalRewriteN = (&graph.edges[e].rewrite).try_into()?;

                for target_height in 0..diagram.size() {
                    let weight = rewrite
                        .singular_preimage(target_height)
                        .map(|j| weights[source][j])
                        .sum();
                    weights[i][target_height] = cmp::max(weights[i][target_height], weight);
                }
            }
        }

        // Construct a monotone iterator for every edge and identify the trivial edges.
        let mut iterators: IdxVec<EdgeId, (MonotoneIterator, bool)> =
            IdxVec::with_capacity(graph.edges.len());
        for edge in graph.edges.values() {
            let rewrite: &CubicalRewriteN = (&edge.rewrite).try_into()?;

            let source_weights = &weights[edge.source];
            let target_weights = &weights[edge.target];

            // Construct the underlying singular monotone map.
            let f = rewrite.singular_monotone(source_weights.len());

            // Construct the iterator of all injectifications of f.
            let iterator = injectify(&f, source_weights, target_weights, false);

            // Check if the only injectification is the identity.
            let trivial = iterator.is_trivial(target_weights.iter().sum());

            iterators.push((iterator, trivial));
        }

        // Identify the edges that must be equal.
        let mut union_find: UnionFind<EdgeId> = UnionFind::new(graph.edges.len());
        for square in graph.squares() {
            let t = square.top;
            let l = square.left;
            let r = square.right;
            let b = square.bottom;
            // If top and left are trivial, identify right and bottom.
            if iterators[t].1 && iterators[l].1 {
                union_find.union(r, b);
            }
            // If top and bottom are trivial, identify left and right.
            if iterators[t].1 && iterators[b].1 {
                union_find.union(l, r);
            }
            // If left and right are trivial, identify top and bottom.
            if iterators[l].1 && iterators[r].1 {
                union_find.union(t, b);
            }
            // If right and bottom are trivial, identify top and left.
            if iterators[r].1 && iterators[b].1 {
                union_find.union(t, l);
            }
        }

        // This maps each edge to its representative.
        let edge_keys: IdxVec<EdgeId, EdgeId> = IdxVec::from_iter(union_find.into_labeling());

        // Combine the iterators of the identified edges.
        let mut combined_iterators: Vec<MonotoneIterator> = vec![];
        let mut indices: IdxVec<EdgeId, Option<usize>> =
            IdxVec::from_iter(vec![None; graph.edges.len()]);
        for e in graph.edges.keys() {
            let key = edge_keys[e];
            let iterator = &iterators[e].0;
            match indices[key] {
                None => {
                    combined_iterators.push(iterator.clone());
                    indices[key] = Some(combined_iterators.len() - 1);
                }
                Some(i) => {
                    combined_iterators[i].restrict_to(iterator);
                }
            }
            indices[e] = indices[key];
        }

        // By this point, all indices should be non-null.
        let indices: IdxVec<EdgeId, usize> = indices.values().map(|x| x.unwrap()).collect();

        // Construct the cartesian product.
        let product: MultiProduct<BiasedMonotoneIterator> = combined_iterators
            .iter()
            .map(|iterator| BiasedMonotoneIterator(bias, iterator.clone()))
            .multi_cartesian_product();

        Ok(Self {
            graph,
            weights,
            indices,
            product,
        })
    }

    fn check_commutativity(&self, monotones: &[Monotone]) -> bool {
        self.graph.squares().into_iter().all(|square| {
            let t = square.top;
            let l = square.left;
            let r = square.right;
            let b = square.bottom;
            let top = &monotones[self.indices[t]];
            let left = &monotones[self.indices[l]];
            let right = &monotones[self.indices[r]];
            let bottom = &monotones[self.indices[b]];
            compose(top, right).unwrap() == compose(left, bottom).unwrap()
        })
    }

    fn construct_expanded_graph(
        &self,
        monotones: &[Monotone],
    ) -> Result<CubicalGraph, DimensionError> {
        // Expand the nodes.
        let mut expanded_nodes: IdxVec<NodeId, Node> =
            IdxVec::with_capacity(self.graph.nodes.len());
        for (i, node) in self.graph.nodes.iter() {
            let diagram: &DiagramN = (&node.diagram).try_into()?;
            let expanded_diagram = diagram.singular_expansion(&self.weights[i]);
            let expanded_heights = singular_expansion(&node.heights, &self.weights[i]);

            expanded_nodes.push(Node {
                key: node.key.clone(),
                coord: node.coord.clone(),
                diagram: expanded_diagram.into(),
                heights: expanded_heights,
                incoming_edges: node.incoming_edges.clone(),
                outgoing_edges: node.outgoing_edges.clone(),
            });
        }

        // Expand the edges.
        let mut expanded_edges: IdxVec<EdgeId, Edge> =
            IdxVec::with_capacity(self.graph.edges.len());
        for (e, edge) in self.graph.edges.iter() {
            let s = edge.source;
            let t = edge.target;
            let rewrite: &CubicalRewriteN = (&edge.rewrite).try_into()?;

            let f = &monotones[self.indices[e]];
            let expanded_source: &DiagramN = (&expanded_nodes[s].diagram).try_into()?;
            let expanded_target: &DiagramN = (&expanded_nodes[t].diagram).try_into()?;

            let expanded_rewrite = rewrite.singular_expansion(
                f,
                &self.weights[s],
                &self.weights[t],
                &expanded_source.cubical_cospans(),
                &expanded_target.cubical_cospans(),
            );

            expanded_edges.push(Edge {
                source: s,
                target: t,
                rewrite: expanded_rewrite.into(),
            });
        }

        // Reconstruct the expanded graph.
        Ok(CubicalGraph {
            nodes: expanded_nodes,
            edges: expanded_edges,
            sizes: self.graph.sizes.clone(),
        })
    }
}

impl Iterator for SingularExpansion {
    type Item = CubicalGraph;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.product.next() {
                None => return None,
                Some(monotones) => {
                    if self.check_commutativity(&monotones) {
                        return self.construct_expanded_graph(&monotones).ok();
                    }
                }
            }
        }
    }
}

impl DiagramN {
    /// Construct the singular expansion of a diagram.
    fn singular_expansion(&self, weights: &[usize]) -> Self {
        assert_eq!(weights.len(), self.size());

        let cospans: Vec<Cospan> = weights
            .iter()
            .zip(self.cospans())
            .flat_map(|(&weight, cospan)| cospan.expand(weight))
            .collect();

        Self::new_unsafe(self.source(), cospans)
    }
}

impl CubicalRewriteN {
    /// Construct the singular expansion of a rewrite.
    fn singular_expansion(
        &self,
        f: &[usize],                      // singular monotone
        source_weights: &[usize],         // weights of source diagram
        target_weights: &[usize],         // weights of target diagram
        source_cospans: &[CubicalCospan], // cospans of expanded source diagram
        target_cospans: &[CubicalCospan], // cospans of expanded target diagram
    ) -> Self {
        assert_eq!(f.len(), source_cospans.len());

        let mut source_index: SingularHeight = 0;
        let mut target_index: SingularHeight = 0;

        // The slices of the expanded rewrite.
        let mut regular_slices: Vec<CubicalRewrite> = vec![];
        let mut singular_slices: Vec<CubicalRewrite> = vec![];

        // Invariant: target_index = ∑ target_weights[0..target_height).
        // Invariant: regular_slices.len() = target_index && singular_slices.len() = source_index.
        for (target_height, &target_weight) in target_weights.iter().enumerate() {
            let preimage = self.singular_preimage(target_height);
            let preimage_is_empty = preimage.is_empty();

            for source_height in preimage {
                let source_weight = source_weights[source_height];

                for _ in 0..source_weight {
                    singular_slices.push(self.singular_slice(source_height));
                }
            }

            // Invariant: f'[target_index + j] = source_index + i.
            let mut i = 0;
            for j in 0..target_weight {
                regular_slices.push(if preimage_is_empty {
                    if i == 0 && j == 0 {
                        self.regular_slice(target_height)
                    } else {
                        self.regular_slice(target_height)
                            .compose(&target_cospans[target_index].forward)
                            .unwrap()
                    }
                } else if i == 0 && j == 0 {
                    self.regular_slice(target_height)
                } else if i == 0 {
                    source_cospans[source_index + i]
                        .forward
                        .compose(&singular_slices[source_index + i])
                        .unwrap()
                } else {
                    source_cospans[source_index + i - 1]
                        .backward
                        .compose(&singular_slices[source_index + i - 1])
                        .unwrap()
                });

                if source_index + i < f.len() && f[source_index + i] == target_index + j {
                    i += 1;
                }
            }

            source_index += i;
            target_index += target_weight;

            assert_eq!(regular_slices.len(), target_index);
            assert_eq!(singular_slices.len(), source_index);
        }

        regular_slices.push(self.regular_slice(target_weights.len() + 1));

        assert_eq!(source_cospans.len(), singular_slices.len());
        assert_eq!(target_cospans.len() + 1, regular_slices.len());

        Self::from_monotone_with_payload_unsafe(
            self.dimension(),
            source_cospans,
            target_cospans,
            f,
            &singular_slices,
            &CubicalPayload::new(regular_slices),
        )
    }
}

fn singular_expansion(heights: &[Height], weights: &[usize]) -> Vec<Height> {
    assert_eq!(heights.len(), weights.len() * 2 + 1);

    heights
        .iter()
        .enumerate()
        .flat_map(|(i, height)| {
            let n = match Height::from_int(i) {
                Height::Regular(_) => 1,
                Height::Singular(j) => weights[j] * 2 - 1,
            };
            std::iter::repeat(*height).take(n)
        })
        .collect()
}

// Regular expansion

struct RegularExpansion {
    graph: CubicalGraph,
    weights: IdxVec<NodeId, Vec<usize>>, /* weights[i][j] is the weight of regular slice j of node i */
    indices: IdxVec<EdgeId, usize>,      // indices[e] is the index of edge e in the product below
    product: MultiProduct<BiasedMonotoneIterator>,
}

impl RegularExpansion {
    fn new(graph: CubicalGraph, bias: Bias) -> Result<Self, DimensionError> {
        // Assign weights to the regular slices of every node.
        let mut weights: IdxVec<NodeId, Vec<usize>> =
            IdxVec::from_iter(vec![vec![]; graph.nodes.len()]);
        for i in graph.reverse_topological_sort() {
            let diagram: &DiagramN = (&graph.nodes[i].diagram).try_into()?;

            // Start by assigning weight 1 to every regular slice.
            for _ in 0..diagram.size() + 1 {
                weights[i].push(1);
            }

            // Now propagate weights backwards along every outgoing edge.
            for &e in &graph.nodes[i].outgoing_edges {
                let target = graph.edges[e].target;
                let rewrite: &CubicalRewriteN = (&graph.edges[e].rewrite).try_into()?;

                for source_height in 0..diagram.size() + 1 {
                    let weight = rewrite
                        .regular_preimage(source_height)
                        .map(|j| weights[target][j])
                        .sum();
                    weights[i][source_height] = cmp::max(weights[i][source_height], weight);
                }
            }
        }

        // Construct a monotone iterator for every edge and identify the trivial edges.
        let mut iterators: IdxVec<EdgeId, (MonotoneIterator, bool)> =
            IdxVec::with_capacity(graph.edges.len());
        for edge in graph.edges.values() {
            let rewrite: &CubicalRewriteN = (&edge.rewrite).try_into()?;

            let source_weights = &weights[edge.source];
            let target_weights = &weights[edge.target];

            // Construct the underlying regular monotone map.
            let f = rewrite.regular_monotone(target_weights.len() - 1);

            // Construct the iterator of all injectifications of f.
            let iterator = injectify(&f, target_weights, source_weights, true);

            // Check if the only injectification is the identity.
            let trivial = iterator.is_trivial(source_weights.iter().sum());

            iterators.push((iterator, trivial));
        }

        // Identify the edges that must be equal.
        let mut union_find: UnionFind<EdgeId> = UnionFind::new(graph.edges.len());
        for square in graph.squares() {
            let t = square.top;
            let l = square.left;
            let r = square.right;
            let b = square.bottom;
            // If top and left are trivial, identify right and bottom.
            if iterators[t].1 && iterators[l].1 {
                union_find.union(r, b);
            }
            // If top and bottom are trivial, identify left and right.
            if iterators[t].1 && iterators[b].1 {
                union_find.union(l, r);
            }
            // If left and right are trivial, identify top and bottom.
            if iterators[l].1 && iterators[r].1 {
                union_find.union(t, b);
            }
            // If right and bottom are trivial, identify top and left.
            if iterators[r].1 && iterators[b].1 {
                union_find.union(t, l);
            }
        }

        // This maps each edge to its representative.
        let edge_keys: IdxVec<EdgeId, EdgeId> = IdxVec::from_iter(union_find.into_labeling());

        // Combine the iterators of the identified edges.
        let mut combined_iterators: Vec<MonotoneIterator> = vec![];
        let mut indices: IdxVec<EdgeId, Option<usize>> =
            IdxVec::from_iter(vec![None; graph.edges.len()]);
        for e in graph.edges.keys() {
            let key = edge_keys[e];
            let iterator = &iterators[e].0;
            match indices[key] {
                None => {
                    combined_iterators.push(iterator.clone());
                    indices[key] = Some(combined_iterators.len() - 1);
                }
                Some(i) => {
                    combined_iterators[i].restrict_to(iterator);
                }
            }
            indices[e] = indices[key];
        }

        // By this point, all indices should be non-null.
        let indices: IdxVec<EdgeId, usize> = indices.values().map(|x| x.unwrap()).collect();

        // Construct the cartesian product.
        let product: MultiProduct<BiasedMonotoneIterator> = combined_iterators
            .iter()
            .map(|iterator| BiasedMonotoneIterator(bias.reverse(), iterator.clone()))
            .multi_cartesian_product();

        Ok(Self {
            graph,
            weights,
            indices,
            product,
        })
    }

    fn check_commutativity(&self, monotones: &[Monotone]) -> bool {
        self.graph.squares().into_iter().all(|square| {
            let t = square.top;
            let l = square.left;
            let r = square.right;
            let b = square.bottom;
            let top = &monotones[self.indices[t]];
            let left = &monotones[self.indices[l]];
            let right = &monotones[self.indices[r]];
            let bottom = &monotones[self.indices[b]];
            compose(right, top).unwrap() == compose(bottom, left).unwrap()
        })
    }

    fn construct_expanded_graph(
        &self,
        monotones: &[Monotone],
    ) -> Result<CubicalGraph, DimensionError> {
        // Expand the nodes.
        let mut expanded_nodes: IdxVec<NodeId, Node> =
            IdxVec::with_capacity(self.graph.nodes.len());
        for (i, node) in self.graph.nodes.iter() {
            let diagram: &DiagramN = (&node.diagram).try_into()?;
            let expanded_diagram = diagram.regular_expansion(&self.weights[i]);
            let expanded_heights = regular_expansion(&node.heights, &self.weights[i]);

            expanded_nodes.push(Node {
                key: node.key.clone(),
                coord: node.coord.clone(),
                diagram: expanded_diagram.into(),
                heights: expanded_heights,
                incoming_edges: node.incoming_edges.clone(),
                outgoing_edges: node.outgoing_edges.clone(),
            });
        }

        // Expand the edges.
        let mut expanded_edges: IdxVec<EdgeId, Edge> =
            IdxVec::with_capacity(self.graph.edges.len());
        for (e, edge) in self.graph.edges.iter() {
            let s = edge.source;
            let t = edge.target;
            let rewrite: &CubicalRewriteN = (&edge.rewrite).try_into()?;

            let f = &monotones[self.indices[e]];
            let expanded_source: &DiagramN = (&expanded_nodes[s].diagram).try_into()?;
            let expanded_target: &DiagramN = (&expanded_nodes[t].diagram).try_into()?;

            let expanded_rewrite = rewrite.regular_expansion(
                f,
                &self.weights[s],
                &self.weights[t],
                &expanded_source.cubical_cospans(),
                &expanded_target.cubical_cospans(),
            );

            expanded_edges.push(Edge {
                source: s,
                target: t,
                rewrite: expanded_rewrite.into(),
            });
        }

        // Construct the expanded graph.
        Ok(CubicalGraph {
            nodes: expanded_nodes,
            edges: expanded_edges,
            sizes: self.graph.sizes.clone(),
        })
    }
}

impl Iterator for RegularExpansion {
    type Item = CubicalGraph;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.product.next() {
                None => return None,
                Some(monotones) => {
                    if self.check_commutativity(&monotones) {
                        return self.construct_expanded_graph(&monotones).ok();
                    }
                }
            }
        }
    }
}

impl DiagramN {
    /// Construct the regulakr expansion of a diagram.
    fn regular_expansion(&self, weights: &[usize]) -> Self {
        assert_eq!(weights.len(), self.size() + 1);

        let cospans: Vec<Cospan> = weights
            .iter()
            .map(|&weight| Cospan::identity(self.dimension() - 1).expand(weight - 1))
            .interleave(self.cospans().iter().map(|cospan| vec![cospan.clone()]))
            .flatten()
            .collect();

        Self::new_unsafe(self.source(), cospans)
    }
}

impl CubicalRewriteN {
    /// Construct the regular expansion of a rewrite.
    fn regular_expansion(
        &self,
        f: &[usize],                      // regular monotone
        source_weights: &[usize],         // weights of source diagram
        target_weights: &[usize],         // weights of target diagram
        source_cospans: &[CubicalCospan], // cospans of expanded source diagram
        target_cospans: &[CubicalCospan], // cospans of expanded target diagram
    ) -> Self {
        assert_eq!(f.len(), target_cospans.len() + 1);

        let mut source_index: RegularHeight = 0;
        let mut target_index: RegularHeight = 0;

        // The slices of the expanded rewrite.
        let mut regular_slices: Vec<CubicalRewrite> = vec![];
        let mut singular_slices: Vec<CubicalRewrite> = vec![];

        // Invariant: source_index = ∑ source_weights[0..source_height).
        // Invariant: regular_slices.len() = target_index && singular_slices.len() = source_index - 1.
        for (source_height, &source_weight) in source_weights.iter().enumerate() {
            let preimage = self.regular_preimage(source_height);
            let preimage_is_empty = preimage.is_empty();

            for target_height in preimage {
                let target_weight = target_weights[target_height];

                for _ in 0..target_weight {
                    regular_slices.push(self.regular_slice(target_height));
                }
            }

            // Invariant: f'[source_index + i - 1] = target_index + j - 1.
            let mut j = 0;
            for i in 0..source_weight {
                if source_index + i > 0 {
                    singular_slices.push(if preimage_is_empty {
                        if i == 0 && j == 0 {
                            self.singular_slice(source_height - 1)
                        } else {
                            source_cospans[source_index - 1]
                                .backward
                                .compose(&self.singular_slice(source_height - 1))
                                .unwrap()
                        }
                    } else if i == 0 && j == 0 {
                        self.singular_slice(source_height - 1)
                    } else if j == 0 {
                        regular_slices[target_index + j]
                            .compose(&target_cospans[target_index + j - 1].backward)
                            .unwrap()
                    } else {
                        regular_slices[target_index + j - 1]
                            .compose(&target_cospans[target_index + j - 1].forward)
                            .unwrap()
                    });
                }

                if target_index + j < f.len() && f[target_index + j] == source_index + i {
                    j += 1;
                }
            }

            target_index += j;
            source_index += source_weight;

            assert_eq!(regular_slices.len(), target_index);
            assert_eq!(singular_slices.len(), source_index - 1);
        }

        assert_eq!(source_cospans.len(), singular_slices.len());
        assert_eq!(target_cospans.len() + 1, regular_slices.len());

        let g = dual_inv(f, source_cospans.len() + 1);
        Self::from_monotone_with_payload_unsafe(
            self.dimension(),
            source_cospans,
            target_cospans,
            &g,
            &singular_slices,
            &CubicalPayload::new(regular_slices),
        )
    }
}

fn regular_expansion(heights: &[Height], weights: &[usize]) -> Vec<Height> {
    assert_eq!(heights.len(), weights.len() * 2 - 1);

    heights
        .iter()
        .enumerate()
        .flat_map(|(i, height)| {
            let n = match Height::from_int(i) {
                Height::Regular(j) => weights[j] * 2 - 1,
                Height::Singular(_) => 1,
            };
            std::iter::repeat(*height).take(n)
        })
        .collect()
}

// Injectification

fn injectify(
    f: &[usize],
    source_weights: &[usize],
    target_weights: &[usize],
    should_preserve_extrema: bool, // whether should preserve top and bottom elements.
) -> MonotoneIterator {
    assert_eq!(f.len(), source_weights.len());

    // Check that f preserves top and bottom elements.
    if should_preserve_extrema {
        assert_eq!(f[0], 0);
        assert_eq!(f[f.len() - 1], target_weights.len() - 1);
    }

    // Invariant: offsets[j] = ∑ target_weights[0..j).
    let mut offsets = vec![0];
    for n in target_weights {
        offsets.push(offsets.last().unwrap() + n);
    }

    // For every f[i] = j, every copy of i is mapped to a copy of j.
    let mut constraints = vec![];
    for (i, &j) in f.iter().enumerate() {
        for _ in 0..source_weights[i] {
            constraints.push(offsets[j]..offsets[j + 1]);
        }
    }

    // Tighten the first and last constraints to make sure we preserve top and bottom elements.
    if should_preserve_extrema {
        let len = constraints.len();
        constraints[0].end = 1;
        constraints[len - 1].start = constraints[len - 1].end - 1;
    }

    MonotoneIterator::new(true, &constraints)
}

// Utils

impl Cospan {
    /// Make an identity cospan.
    fn identity(dimension: usize) -> Self {
        Self {
            forward: GenericRewrite::identity(dimension),
            backward: GenericRewrite::identity(dimension),
        }
    }

    /// Take a cospan and expand it n times.
    fn expand(&self, n: usize) -> Vec<Self> {
        match n {
            0 => vec![],
            1 => vec![self.clone()],
            _ => {
                let mut cospans = Vec::with_capacity(n);
                let dimension = self.forward.dimension();
                // First cospan
                cospans.push(Self {
                    forward: self.forward.clone(),
                    backward: GenericRewrite::identity(dimension),
                });
                // Intermediate cospans
                for _ in 1..n - 1 {
                    cospans.push(Self::identity(dimension));
                }
                // Final cospan
                cospans.push(Self {
                    forward: GenericRewrite::identity(dimension),
                    backward: self.backward.clone(),
                });
                cospans
            }
        }
    }
}

impl Diagram {
    fn heights(&self) -> Vec<Height> {
        match self {
            Self::Diagram0(_) => vec![],
            Self::DiagramN(d) => (0..d.size() * 2 + 1).map(Height::from_int).collect(),
        }
    }
}

impl DiagramN {
    fn cubical_cospans(&self) -> Vec<CubicalCospan> {
        self.cospans()
            .iter()
            .cloned()
            .map(|cospan| cospan.into())
            .collect()
    }
}

impl CubicalRewrite {
    /// Checks if the underlying monotone is an identity.
    /// Note this does *not* inductively check if the subslices are themselves parallel.
    fn is_parallel(&self) -> bool {
        match self {
            Self::Rewrite0(_) => true,
            Self::RewriteN(r) => r.cones().iter().all(|cone| cone.internal.source.len() == 1),
        }
    }
}

impl CubicalRewriteN {
    fn singular_slice(&self, source_height: SingularHeight) -> CubicalRewrite {
        self.slice(source_height)
    }

    fn regular_slice(&self, target_height: RegularHeight) -> CubicalRewrite {
        self.payload()
            .0
            .iter()
            .find_map(|(i, fi)| (*i == target_height).then(|| fi.clone()))
            .unwrap_or_else(|| CubicalRewrite::identity(self.dimension() - 1))
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct BiasedMonotoneIterator(Bias, MonotoneIterator);

impl Iterator for BiasedMonotoneIterator {
    type Item = Monotone;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            Bias::Left => self.1.next(),
            Bias::Right => self.1.next_back(),
        }
    }
}

// Cubical graphs

pub type Coord = Vec<Height>;

fn mk_coord(hs: &[Height], height: Height) -> Coord {
    let mut coord = hs.to_owned();
    coord.push(height);
    coord
}

declare_idx! {
    #[derive(Default)]
    pub struct NodeId = usize;
}

declare_idx! {
    #[derive(Default)]
    pub struct EdgeId = usize;
}

unsafe impl IndexType for NodeId {
    fn new(x: usize) -> Self {
        Self(x)
    }

    fn index(&self) -> usize {
        self.0
    }

    fn max() -> Self {
        Self(::std::usize::MAX)
    }
}

unsafe impl IndexType for EdgeId {
    fn new(x: usize) -> Self {
        Self(x)
    }

    fn index(&self) -> usize {
        self.0
    }

    fn max() -> Self {
        Self(::std::usize::MAX)
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    key: Coord,
    coord: Coord,
    diagram: Diagram,
    heights: Vec<Height>,
    incoming_edges: Vec<EdgeId>,
    outgoing_edges: Vec<EdgeId>,
}

#[derive(Clone, Debug)]
pub struct Edge {
    source: NodeId,
    target: NodeId,
    rewrite: CubicalRewrite,
}

#[derive(Clone, Debug)]
pub struct CubicalGraph {
    sizes: Vec<usize>,
    nodes: IdxVec<NodeId, Node>,
    edges: IdxVec<EdgeId, Edge>,
}

struct Square {
    top: EdgeId,
    left: EdgeId,
    right: EdgeId,
    bottom: EdgeId,
}

impl CubicalGraph {
    pub fn new(diagram: Diagram) -> Self {
        let heights = diagram.heights();
        let node = Node {
            key: vec![],
            coord: vec![],
            diagram,
            heights,
            incoming_edges: vec![],
            outgoing_edges: vec![],
        };
        Self {
            sizes: vec![],
            nodes: IdxVec::from_iter([node]),
            edges: IdxVec::new(),
        }
    }

    pub fn size(&self, direction: usize) -> usize {
        self.sizes[direction]
    }

    pub fn dimension(&self) -> usize {
        self.sizes.len()
    }

    /// Finds the node with the given coordinate.
    fn get_node_id(&self, coord: &[Height]) -> NodeId {
        let mut i = 0;
        let mut prod = 1;
        let mut offset = self.dimension();
        for height in coord.iter().rev() {
            i += height.to_int() * prod;
            prod *= 2 * self.sizes[offset - 1] + 1;
            offset -= 1;
        }
        NodeId(i)
    }

    /// Finds the direction of an edge between two keys.
    fn get_edge_direction(source_coord: &[Height], target_coord: &[Height]) -> Option<usize> {
        source_coord
            .iter()
            .zip(target_coord)
            .position(|(h, k)| h != k)
    }

    /// Returns all squares in the graph.
    fn squares(&self) -> Vec<Square> {
        let mut squares = Vec::new();
        // Iterate over all possible top-left nodes.
        for tl in self.topological_sort() {
            let coord_tl = &self.nodes[tl].coord;

            // Count the number of singular heights in the coordinate.
            let singular: usize = coord_tl
                .iter()
                .map(|h| match h {
                    Height::Regular(_) => 0,
                    Height::Singular(_) => 1,
                })
                .sum();

            // If the coordinate contains more than n - 2 singular heights, the node cannot be the top-left node of a square.
            // Since the nodes are in the topological ordering, all the remaining nodes are also not good, so we can just break.
            if singular + 2 > self.dimension() {
                break;
            }

            // Pick two distinct outgoing edges.
            for &t in &self.nodes[tl].outgoing_edges {
                for &l in &self.nodes[tl].outgoing_edges {
                    if t < l {
                        // Find the targets of these edges (i.e. the top-right and bottom-left nodes).
                        let tr = self.edges[t].target;
                        let bl = self.edges[l].target;

                        let coord_bl = &self.nodes[bl].coord;
                        let coord_tr = &self.nodes[tr].coord;

                        // Get the directions of these edges by comparing the source and target keys.
                        let i = Self::get_edge_direction(coord_tl, coord_bl).unwrap();
                        let j = Self::get_edge_direction(coord_tl, coord_tr).unwrap();

                        // The two edges can form a square only if their directions are orthogonal.
                        if i != j {
                            let mut coord_br = coord_tl.clone();
                            coord_br[i] = coord_bl[i];
                            coord_br[j] = coord_tr[j];

                            // Find the bottom-right node of the square.
                            let br = self.get_node_id(&coord_br);

                            // Find the final two edges.
                            let &r = self.nodes[tr]
                                .outgoing_edges
                                .iter()
                                .find(|&&e| self.edges[e].target == br)
                                .unwrap();
                            let &b = self.nodes[bl]
                                .outgoing_edges
                                .iter()
                                .find(|&&e| self.edges[e].target == br)
                                .unwrap();

                            squares.push(Square {
                                top: t,
                                left: l,
                                right: r,
                                bottom: b,
                            });
                        }
                    }
                }
            }
        }
        squares
    }

    /// Returns the nodes in the topological ordering.
    /// For every edge (i, j), we have that i comes before j in this ordering.
    fn topological_sort(&self) -> Vec<NodeId> {
        (0..self.nodes.len())
            .map(NodeId)
            .sorted_by_key(|&i| self.nodes[i].incoming_edges.len())
            .collect()
    }

    /// Returns the nodes in the reverse topological ordering.
    /// For every edge (i, j), we have that j comes before i in this ordering.
    fn reverse_topological_sort(&self) -> Vec<NodeId> {
        (0..self.nodes.len())
            .map(NodeId)
            .sorted_by_key(|&i| self.nodes[i].outgoing_edges.len())
            .collect()
    }
}

// Cubical rewrites

thread_local! {
    static CONE_FACTORY: RefCell<HConsign<ConeInternal<CubicalAllocator>, Hasher>> =
        RefCell::new(HConsign::with_capacity_and_hasher(37, Hasher::default()));

    static REWRITE_FACTORY: RefCell<HConsign<RewriteInternal<CubicalAllocator>, Hasher>> =
        RefCell::new(HConsign::with_capacity_and_hasher(37, Hasher::default()));
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CubicalAllocator;

pub type CubicalCone = GenericCone<CubicalAllocator>;

pub type CubicalCospan = GenericCospan<CubicalAllocator>;

pub type CubicalRewrite = GenericRewrite<CubicalAllocator>;
pub type CubicalRewriteN = GenericRewriteN<CubicalAllocator>;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CubicalPayload(Vec<(RegularHeight, CubicalRewrite)>);

impl CubicalPayload {
    fn new(slices: Vec<CubicalRewrite>) -> Self {
        let mut payload = Vec::new();
        for (i, slice) in slices.into_iter().enumerate() {
            if !slice.is_identity() {
                payload.push((i, slice));
            }
        }
        Self(payload)
    }
}

impl Default for CubicalPayload {
    fn default() -> Self {
        Self(vec![])
    }
}

impl Composable for CubicalPayload {
    fn compose<A>(f: &GenericRewriteN<A>, g: &GenericRewriteN<A>) -> Result<Self, CompositionError>
    where
        A: RewriteAllocator<Payload = Self>,
    {
        let mut payload = g.payload().0.clone();
        for (i, fi) in &f.payload().0 {
            for j in g.regular_preimage(*i) {
                match payload.iter().position(|(k, _)| *k >= j) {
                    None => {
                        payload.push((j, fi.clone()));
                    }
                    Some(index) => {
                        if j == payload[index].0 {
                            payload[index].1 = fi.compose(&payload[index].1)?;
                        } else {
                            payload.insert(index, (j, fi.clone()));
                        }
                    }
                }
            }
        }
        Ok(Self(payload))
    }
}

impl RewriteAllocator for CubicalAllocator {
    type ConeCell = HConsed<ConeInternal<Self>>;
    type Payload = CubicalPayload;
    type RewriteCell = HConsed<RewriteInternal<Self>>;

    #[inline]
    fn mk_cone(internal: ConeInternal<Self>) -> Self::ConeCell {
        CONE_FACTORY.with(|factory| factory.borrow_mut().mk(internal))
    }

    #[inline]
    fn mk_rewrite(internal: RewriteInternal<Self>) -> Self::RewriteCell {
        REWRITE_FACTORY.with(|factory| factory.borrow_mut().mk(internal))
    }

    #[inline]
    fn collect_garbage() {
        CONE_FACTORY.with(|factory| factory.borrow_mut().collect_to_fit());
        REWRITE_FACTORY.with(|factory| factory.borrow_mut().collect_to_fit());
    }
}

// Conversions between rewrites and cubical rewrites

impl From<Cospan> for CubicalCospan {
    fn from(cospan: Cospan) -> Self {
        Self {
            forward: cospan.forward.into(),
            backward: cospan.backward.into(),
        }
    }
}

impl From<CubicalCospan> for Cospan {
    fn from(cospan: CubicalCospan) -> Self {
        Self {
            forward: cospan.forward.into(),
            backward: cospan.backward.into(),
        }
    }
}

impl From<Cone> for CubicalCone {
    fn from(cone: Cone) -> Self {
        Self::new(
            cone.index,
            cone.internal
                .source
                .iter()
                .cloned()
                .map(|cospan| cospan.into())
                .collect(),
            cone.internal.target.clone().into(),
            cone.internal
                .slices
                .iter()
                .cloned()
                .map(|rewrite| rewrite.into())
                .collect(),
        )
    }
}

impl From<CubicalCone> for Cone {
    fn from(cone: CubicalCone) -> Self {
        Self::new(
            cone.index,
            cone.internal
                .source
                .iter()
                .cloned()
                .map(|cospan| cospan.into())
                .collect(),
            cone.internal.target.clone().into(),
            cone.internal
                .slices
                .iter()
                .cloned()
                .map(|rewrite| rewrite.into())
                .collect(),
        )
    }
}

impl From<Rewrite> for CubicalRewrite {
    fn from(rewrite: Rewrite) -> Self {
        match rewrite {
            Rewrite::Rewrite0(f) => Self::Rewrite0(f),
            Rewrite::RewriteN(f) => Self::RewriteN(CubicalRewriteN::new(
                f.dimension(),
                f.cones().iter().cloned().map(|cone| cone.into()).collect(),
            )),
        }
    }
}

impl From<CubicalRewrite> for Rewrite {
    fn from(rewrite: CubicalRewrite) -> Self {
        match rewrite {
            CubicalRewrite::Rewrite0(f) => Self::Rewrite0(f),
            CubicalRewrite::RewriteN(f) => Self::RewriteN(RewriteN::new(
                f.dimension(),
                f.cones().iter().cloned().map(|cone| cone.into()).collect(),
            )),
        }
    }
}
