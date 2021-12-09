use std::{
    cell::RefCell,
    cmp,
    collections::HashMap,
    convert::{Into, TryInto},
    iter::FromIterator,
};

use hashconsing::{HConsed, HConsign, HashConsign};
use homotopy_common::{
    graph::{Edge, Graph, Node},
    hash::FastHasher,
    idx::IdxVec,
    union_find::UnionFind,
};
use itertools::{Itertools, MultiProduct};

use crate::{
    common::{DimensionError, Direction, Height, RegularHeight, SingularHeight, SliceIndex},
    diagram::{Diagram, DiagramN},
    graph::{explode, mk_coord, Coord, SliceGraph, TopologicalSort},
    monotone::{compose, dual_inv, Monotone, MonotoneIterator},
    rewrite::{
        Composable, CompositionError, ConeInternal, Cospan, GenericCone, GenericCospan,
        GenericRewrite, GenericRewriteN, RewriteAllocator, RewriteInternal,
    },
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
        if self.inner.edge_values().all(|edge| edge.is_parallel()) {
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
    weights: IdxVec<Node, Vec<usize>>, // weights[n][i] = weight of singular slice i of node n
    indices: IdxVec<Edge, usize>,      // indices[e] = index of edge e in the product below
    product: MultiProduct<BiasedMonotoneIterator>,
}

impl SingularExpansion {
    fn new(graph: CubicalGraph, bias: Bias) -> Result<Self, DimensionError> {
        // Assign weights to the singular slices of every node.
        let mut weights: IdxVec<Node, Vec<usize>> =
            IdxVec::from_iter(vec![vec![]; graph.inner.node_count()]);
        for i in TopologicalSort::new(&graph.inner) {
            let diagram: &DiagramN = (&graph.inner[i].1).try_into()?;

            // Start by assigning weight 1 to every singular slice.
            for _ in 0..diagram.size() {
                weights[i].push(1);
            }

            // Now propagate weights forwards along every incoming edge.
            for e in graph.inner.incoming_edges(i) {
                let source = graph.inner.source(e);
                let rewrite: &CubicalRewriteN = (&graph.inner[e]).try_into()?;

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
        let mut iterators: IdxVec<Edge, (MonotoneIterator, bool)> =
            IdxVec::with_capacity(graph.inner.edge_count());
        for edge in graph.inner.edge_values() {
            let rewrite: &CubicalRewriteN = (edge.inner()).try_into()?;

            let source_weights = &weights[edge.source()];
            let target_weights = &weights[edge.target()];

            // Construct the underlying singular monotone map.
            let f = rewrite.singular_monotone(source_weights.len());

            // Construct the iterator of all injectifications of f.
            let iterator = injectify(&f, source_weights, target_weights, false);

            // Check if the only injectification is the identity.
            let trivial = iterator.is_trivial(target_weights.iter().sum());

            iterators.push((iterator, trivial));
        }

        // Identify the edges that must be equal.
        let mut union_find: UnionFind<Edge> = UnionFind::new(graph.inner.edge_count());
        for square in graph.squares() {
            let t = square.top;
            let l = square.left;
            let r = square.right;
            let b = square.bottom;
            // If bottom and left are trivial, identify right and top.
            if iterators[b].1 && iterators[l].1 {
                union_find.union(r, t);
            }
            // If top and bottom are trivial, identify left and right.
            if iterators[t].1 && iterators[b].1 {
                union_find.union(l, r);
            }
            // If left and right are trivial, identify top and bottom.
            if iterators[l].1 && iterators[r].1 {
                union_find.union(t, b);
            }
            // If right and top are trivial, identify bottom and left.
            if iterators[r].1 && iterators[t].1 {
                union_find.union(b, l);
            }
        }

        // Combine the iterators of the identified edges.
        let mut combined_iterators: Vec<MonotoneIterator> = vec![];
        let mut indices: IdxVec<Edge, Option<usize>> =
            IdxVec::from_iter(vec![None; graph.inner.edge_count()]);
        for e in graph.inner.edge_keys() {
            let key = union_find.find(e);
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
        let indices: IdxVec<Edge, usize> = indices.values().map(|x| x.unwrap()).collect();

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
            compose(bottom, right).unwrap() == compose(left, top).unwrap()
        })
    }

    fn construct_expanded_graph(
        &self,
        monotones: &[Monotone],
    ) -> Result<CubicalGraph, DimensionError> {
        let graph = &self.graph.inner;
        let mut expanded_graph = CubicalSliceGraph::new();

        // Expand the nodes.
        for (n, nd) in graph.nodes() {
            let coord: &Coord = &nd.0;
            let diagram: &DiagramN = (&nd.1).try_into()?;
            let expanded_diagram = diagram.singular_expansion(&self.weights[n]);

            expanded_graph.add_node((coord.clone(), expanded_diagram.into()));
        }

        // Expand the edges.
        for (e, ed) in graph.edges() {
            let s = ed.source();
            let t = ed.target();
            let rewrite: &CubicalRewriteN = ed.inner().try_into()?;

            let f = &monotones[self.indices[e]];
            let expanded_source: &DiagramN = (&expanded_graph[s].1).try_into()?;
            let expanded_target: &DiagramN = (&expanded_graph[t].1).try_into()?;

            let expanded_rewrite = rewrite.singular_expansion(
                f,
                &self.weights[s],
                &self.weights[t],
                &expanded_source.cubical_cospans(),
                &expanded_target.cubical_cospans(),
            );

            expanded_graph.add_edge(s, t, expanded_rewrite.into());
        }

        let expanded_internal_labels = self
            .graph
            .internal_labels
            .iter()
            .map(|(n, labels)| singular_expansion(labels, &self.weights[n]))
            .collect();

        Ok(CubicalGraph {
            inner: expanded_graph,
            sizes: self.graph.sizes.clone(),
            coord_to_node: self.graph.coord_to_node.clone(),
            labels: self.graph.labels.clone(),
            internal_labels: expanded_internal_labels,
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

fn singular_expansion<A>(labels: &[A], weights: &[usize]) -> Vec<A>
where
    A: Copy,
{
    assert_eq!(labels.len(), weights.len() * 2 + 3);

    let len = labels.len();

    labels
        .iter()
        .enumerate()
        .flat_map(|(i, label)| {
            let n = if i == 0 || i == len - 1 {
                1
            } else {
                match Height::from_int(i - 1) {
                    Height::Regular(_) => 1,
                    Height::Singular(j) => weights[j] * 2 - 1,
                }
            };
            std::iter::repeat(*label).take(n)
        })
        .collect()
}

// Regular expansion

struct RegularExpansion {
    graph: CubicalGraph,
    weights: IdxVec<Node, Vec<usize>>, /* weights[n][i] = weight of regular slice i of node n */
    indices: IdxVec<Edge, usize>,      /* indices[e] = index of edge e in the product below */
    product: MultiProduct<BiasedMonotoneIterator>,
}

impl RegularExpansion {
    fn new(graph: CubicalGraph, bias: Bias) -> Result<Self, DimensionError> {
        // Assign weights to the regular slices of every node.
        let mut weights: IdxVec<Node, Vec<usize>> =
            IdxVec::from_iter(vec![vec![]; graph.inner.node_count()]);
        for i in TopologicalSort::new(&graph.inner).into_iter().rev() {
            let diagram: &DiagramN = (&graph.inner[i].1).try_into()?;

            // Start by assigning weight 1 to every regular slice.
            for _ in 0..diagram.size() + 1 {
                weights[i].push(1);
            }

            // Now propagate weights backwards along every outgoing edge.
            for e in graph.inner.outgoing_edges(i) {
                let target = graph.inner.target(e);
                let rewrite: &CubicalRewriteN = (&graph.inner[e]).try_into()?;

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
        let mut iterators: IdxVec<Edge, (MonotoneIterator, bool)> =
            IdxVec::with_capacity(graph.inner.edge_count());
        for edge in graph.inner.edge_values() {
            let rewrite: &CubicalRewriteN = (edge.inner()).try_into()?;

            let source_weights = &weights[edge.source()];
            let target_weights = &weights[edge.target()];

            // Construct the underlying regular monotone map.
            let f = rewrite.regular_monotone(target_weights.len() - 1);

            // Construct the iterator of all injectifications of f.
            let iterator = injectify(&f, target_weights, source_weights, true);

            // Check if the only injectification is the identity.
            let trivial = iterator.is_trivial(source_weights.iter().sum());

            iterators.push((iterator, trivial));
        }

        // Identify the edges that must be equal.
        let mut union_find: UnionFind<Edge> = UnionFind::new(graph.inner.edge_count());
        for square in graph.squares() {
            let t = square.top;
            let l = square.left;
            let r = square.right;
            let b = square.bottom;
            // If bottom and left are trivial, identify right and top.
            if iterators[b].1 && iterators[l].1 {
                union_find.union(r, t);
            }
            // If top and bottom are trivial, identify left and right.
            if iterators[t].1 && iterators[b].1 {
                union_find.union(l, r);
            }
            // If left and right are trivial, identify top and bottom.
            if iterators[l].1 && iterators[r].1 {
                union_find.union(t, b);
            }
            // If right and top are trivial, identify bottom and left.
            if iterators[r].1 && iterators[t].1 {
                union_find.union(b, l);
            }
        }

        // Combine the iterators of the identified edges.
        let mut combined_iterators: Vec<MonotoneIterator> = vec![];
        let mut indices: IdxVec<Edge, Option<usize>> =
            IdxVec::from_iter(vec![None; graph.inner.edge_count()]);
        for e in graph.inner.edge_keys() {
            let key = union_find.find(e);
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
        let indices: IdxVec<Edge, usize> = indices.values().map(|x| x.unwrap()).collect();

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
            compose(right, bottom).unwrap() == compose(top, left).unwrap()
        })
    }

    fn construct_expanded_graph(
        &self,
        monotones: &[Monotone],
    ) -> Result<CubicalGraph, DimensionError> {
        let graph = &self.graph.inner;
        let mut expanded_graph = CubicalSliceGraph::new();

        // Expand the nodes.
        for (n, nd) in graph.nodes() {
            let coord: &Coord = &nd.0;
            let diagram: &DiagramN = (&nd.1).try_into()?;
            let expanded_diagram = diagram.regular_expansion(&self.weights[n]);

            expanded_graph.add_node((coord.clone(), expanded_diagram.into()));
        }

        // Expand the edges.
        for (e, ed) in graph.edges() {
            let s = ed.source();
            let t = ed.target();
            let rewrite: &CubicalRewriteN = ed.inner().try_into()?;

            let f = &monotones[self.indices[e]];
            let expanded_source: &DiagramN = (&expanded_graph[s].1).try_into()?;
            let expanded_target: &DiagramN = (&expanded_graph[t].1).try_into()?;

            let expanded_rewrite = rewrite.regular_expansion(
                f,
                &self.weights[s],
                &self.weights[t],
                &expanded_source.cubical_cospans(),
                &expanded_target.cubical_cospans(),
            );

            expanded_graph.add_edge(s, t, expanded_rewrite.into());
        }

        let expanded_internal_labels = self
            .graph
            .internal_labels
            .iter()
            .map(|(n, labels)| regular_expansion(labels, &self.weights[n]))
            .collect();

        Ok(CubicalGraph {
            inner: expanded_graph,
            sizes: self.graph.sizes.clone(),
            coord_to_node: self.graph.coord_to_node.clone(),
            labels: self.graph.labels.clone(),
            internal_labels: expanded_internal_labels,
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

fn regular_expansion<A>(labels: &[A], weights: &[usize]) -> Vec<A>
where
    A: Copy,
{
    assert_eq!(labels.len(), weights.len() * 2 + 1);

    let len = labels.len();

    labels
        .iter()
        .enumerate()
        .flat_map(|(i, label)| {
            let n = if i == 0 || i == len - 1 {
                1
            } else {
                match Height::from_int(i - 1) {
                    Height::Regular(j) => weights[j] * 2 - 1,
                    Height::Singular(_) => 1,
                }
            };
            std::iter::repeat(*label).take(n)
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
    fn slice_indices(&self) -> Vec<SliceIndex> {
        match self {
            Self::Diagram0(_) => vec![],
            Self::DiagramN(d) => (0..d.size() * 2 + 3)
                .map(|h| SliceIndex::from_int(h as isize - 1, d.size()))
                .collect(),
        }
    }
}

impl DiagramN {
    fn cubical_cospans(&self) -> Vec<CubicalCospan> {
        self.cospans()
            .iter()
            .cloned()
            .map(|cospan| cospan.convert())
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

type Orientation = usize;
type CubicalSliceGraph = SliceGraph<CubicalAllocator>;

#[derive(Clone, Debug)]
pub struct CubicalGraph {
    inner: CubicalSliceGraph,
    sizes: Vec<usize>,
    coord_to_node: HashMap<Coord, Node>,
    labels: IdxVec<Node, Vec<SliceIndex>>, /* labels[n] = label of node n (i.e. a coordinate in the original graph) */
    internal_labels: IdxVec<Node, Vec<SliceIndex>>, /* internal_labels[n][i] = label of slice i of node n (i.e. a slice index in the original node) */
}

pub struct Square {
    pub top: Edge,
    pub left: Edge,
    pub right: Edge,
    pub bottom: Edge,
    pub orientation: [Orientation; 2],
}

pub struct Cube {
    pub top_front: Edge,
    pub left_front: Edge,
    pub right_front: Edge,
    pub bottom_front: Edge,
    pub top_back: Edge,
    pub left_back: Edge,
    pub right_back: Edge,
    pub bottom_back: Edge,
    pub top_left: Edge,
    pub top_right: Edge,
    pub bottom_left: Edge,
    pub bottom_right: Edge,
    pub orientation: [Orientation; 3],
}

impl CubicalGraph {
    pub fn new(diagram: Diagram) -> Self {
        let slice_indices = diagram.slice_indices();

        let mut graph = Graph::new();
        let n = graph.add_node((vec![], diagram));

        Self {
            inner: graph,
            sizes: Vec::new(),
            coord_to_node: HashMap::from_iter([(vec![], n)]),
            labels: IdxVec::from_iter([vec![]]),
            internal_labels: IdxVec::from_iter([slice_indices]),
        }
    }

    pub fn inner(&self) -> &SliceGraph<CubicalAllocator> {
        &self.inner
    }

    pub fn explode(&self) -> Result<Self, DimensionError> {
        let graph = &self.inner;
        let exploded_graph = explode(graph, CubicalRewriteN::regular_slice)?;

        let mut sizes = self.sizes.clone();
        sizes.push((exploded_graph.node_count() / graph.node_count() - 3) / 2);

        let mut coord_to_node = HashMap::new();
        let mut labels = IdxVec::new();
        let mut internal_labels = IdxVec::new();

        for n in graph.node_keys() {
            let label = self.label(n);
            for index in &self.internal_labels[n] {
                labels.push(mk_coord(label, *index));
            }
        }
        for (n, nd) in exploded_graph.nodes() {
            let (coord, diagram) = &nd.inner();

            coord_to_node.insert(coord.clone(), n);
            internal_labels.push(diagram.slice_indices());
        }

        Ok(Self {
            inner: exploded_graph,
            sizes,
            coord_to_node,
            labels,
            internal_labels,
        })
    }

    pub fn size(&self, direction: usize) -> usize {
        self.sizes[direction]
    }

    pub fn dimension(&self) -> usize {
        self.sizes.len()
    }

    pub fn label(&self, node: Node) -> &[SliceIndex] {
        &self.labels[node]
    }

    /// Finds the orientation of an edge.
    fn get_orientation(&self, edge: Edge) -> usize {
        let graph = &self.inner;
        let s = graph.source(edge);
        let t = graph.target(edge);
        let source_coord = &graph[s].0;
        let target_coord = &graph[t].0;
        source_coord
            .iter()
            .zip(target_coord)
            .position(|(h, k)| h != k)
            .unwrap()
    }

    pub fn get_direction(&self, edge: Edge) -> Direction {
        let graph = &self.inner;
        let s = graph.source(edge);
        let t = graph.target(edge);
        let source_coord = &graph[s].0;
        let target_coord = &graph[t].0;
        source_coord
            .iter()
            .zip(target_coord)
            .find_map(|(h, k)| match h.cmp(k) {
                cmp::Ordering::Less => Some(Direction::Forward),
                cmp::Ordering::Equal => None,
                cmp::Ordering::Greater => Some(Direction::Backward),
            })
            .unwrap()
    }

    fn complete_square(&self, e0: Edge, e1: Edge, i0: usize, i1: usize) -> [Edge; 2] {
        let graph = &self.inner;
        let bl = graph.source(e0);
        let tl = graph.target(e0);
        let br = graph.target(e1);

        let coord_bl = &graph[bl].0;
        let coord_tl = &graph[tl].0;
        let coord_br = &graph[br].0;

        let mut coord_tr = coord_bl.clone();
        coord_tr[i0] = coord_tl[i0];
        coord_tr[i1] = coord_br[i1];

        // Find the top-right corner of the square.
        let tr = self.coord_to_node[&coord_tr];

        // Find the final two edges.
        let r = graph
            .outgoing_edges(br)
            .find(|&e| graph.target(e) == tr)
            .unwrap();
        let t = graph
            .outgoing_edges(tl)
            .find(|&e| graph.target(e) == tr)
            .unwrap();

        [r, t]
    }

    /// Returns all squares in the graph.
    pub fn squares(&self) -> Vec<Square> {
        let graph = &self.inner;

        let mut squares = Vec::new();

        // Iterate over all possible top-left corners.
        for bl in TopologicalSort::new(graph) {
            let coord_bl = &graph[bl].0;

            let rank = coord_bl
                .iter()
                .map(|h| match h {
                    SliceIndex::Boundary(_) => -1,
                    SliceIndex::Interior(Height::Regular(_)) => 0,
                    SliceIndex::Interior(Height::Singular(_)) => 1,
                })
                .sum::<i32>();

            // If the rank is more than dimension - 2, the node cannot be the apex of a square.
            // Since the nodes are in the topological ordering, all the remaining nodes are also not good, so we can just break.
            if rank > self.dimension() as i32 - 2 {
                break;
            }

            // Pick two orthogonal outgoing edges.
            for l in graph.outgoing_edges(bl) {
                let i = self.get_orientation(l);
                for b in graph.outgoing_edges(bl) {
                    let j = self.get_orientation(b);
                    if i < j {
                        let [r, t] = self.complete_square(l, b, i, j);

                        squares.push(Square {
                            top: t,
                            left: l,
                            right: r,
                            bottom: b,
                            orientation: [i, j],
                        });
                    }
                }
            }
        }
        squares
    }

    /// Returns all cubes in the graph.
    pub fn cubes(&self) -> Vec<Cube> {
        let graph = &self.inner;

        let mut cubes = Vec::new();

        // Iterate over all possible bottom-left-front corners.
        for blf in TopologicalSort::new(graph) {
            let coord_blf = &graph[blf].0;

            let rank = coord_blf
                .iter()
                .map(|h| match h {
                    SliceIndex::Boundary(_) => -1,
                    SliceIndex::Interior(Height::Regular(_)) => 0,
                    SliceIndex::Interior(Height::Singular(_)) => 1,
                })
                .sum::<i32>();

            // If the rank is more than dimension - 3, the node cannot be the apex of a cube.
            // Since the nodes are in the topological ordering, all the remaining nodes are also not good, so we can just break.
            if rank > self.dimension() as i32 - 3 {
                break;
            }

            // Pick three orthogonal outgoing edges.
            for lf in graph.outgoing_edges(blf) {
                let i = self.get_orientation(lf);
                for bf in graph.outgoing_edges(blf) {
                    let j = self.get_orientation(bf);
                    for bl in graph.outgoing_edges(blf) {
                        let k = self.get_orientation(bl);
                        if i < j && j < k {
                            let [rf, tf] = self.complete_square(lf, bf, i, j);
                            let [lb, tl] = self.complete_square(lf, bl, i, k);
                            let [bb, br] = self.complete_square(bf, bl, j, k);

                            let [rb, tb] = self.complete_square(lb, bb, i, j);
                            let [rb1, tr] = self.complete_square(rf, br, i, k);
                            let [tb1, tr1] = self.complete_square(tf, tl, j, k);

                            assert_eq!(rb, rb1);
                            assert_eq!(tb, tb1);
                            assert_eq!(tr, tr1);

                            cubes.push(Cube {
                                top_front: tf,
                                left_front: lf,
                                right_front: rf,
                                bottom_front: bf,
                                top_back: tb,
                                left_back: lb,
                                right_back: rb,
                                bottom_back: bb,
                                top_left: tl,
                                top_right: tr,
                                bottom_left: bl,
                                bottom_right: br,
                                orientation: [i, j, k],
                            });
                        }
                    }
                }
            }
        }

        cubes
    }
}

// Cubical rewrites

thread_local! {
    static CONE_FACTORY: RefCell<HConsign<ConeInternal<CubicalAllocator>, FastHasher>> =
        RefCell::new(HConsign::with_capacity_and_hasher(37, FastHasher::default()));

    static REWRITE_FACTORY: RefCell<HConsign<RewriteInternal<CubicalAllocator>, FastHasher>> =
        RefCell::new(HConsign::with_capacity_and_hasher(37, FastHasher::default()));
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CubicalAllocator;

pub type CubicalCone = GenericCone<CubicalAllocator>;

pub type CubicalCospan = GenericCospan<CubicalAllocator>;

pub type CubicalRewrite = GenericRewrite<CubicalAllocator>;
pub type CubicalRewriteN = GenericRewriteN<CubicalAllocator>;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
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
