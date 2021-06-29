use std::convert::{Into, TryFrom, TryInto};

use petgraph::Graph;

use crate::{
    common::{Boundary, DimensionError, Height, SliceIndex},
    diagram::{Diagram, DiagramN},
    rewrite::{Rewrite, RewriteN},
    util::agreeing,
    Generator, Rewrite0,
};

#[derive(Debug, Clone)]
pub struct GraphBuilder<K, N, E> {
    pub nodes: Vec<(K, N)>,
    pub edges: Vec<(usize, usize, E)>,
    dimension: usize,
}

impl<K, E> GraphBuilder<K, Diagram, E> {
    pub fn new(key: K, diagram: Diagram) -> Self {
        Self {
            dimension: diagram.dimension(),
            nodes: vec![(key, diagram)],
            edges: vec![],
        }
    }
}

impl<K> GraphBuilder<K, Option<Generator>, Rewrite> {
    pub fn new(source_key: K, target_key: K, rewrite: Rewrite) -> Self {
        Self {
            dimension: rewrite.dimension(),
            nodes: vec![(source_key, None), (target_key, None)],
            edges: vec![(0, 1, rewrite)],
        }
    }
}

// impl<K, E> GraphBuilder<K, Rewrite, E> {
//     pub fn new(key: K, rewrite: Rewrite) -> Self {
//         Self {
//             dimension: rewrite.dimension(),
//             nodes: vec![(key, rewrite)],
//             edges: vec![],
//         }
//     }
// }

impl<K, N, E> GraphBuilder<K, N, E> {
    pub fn build(self) -> Graph<(K, N), E> {
        let mut graph: Graph<(K, N), E> = Graph::with_capacity(self.nodes.len(), self.edges.len());

        for node in self.nodes {
            graph.add_node(node);
        }

        for (source, target, edge) in self.edges {
            let source = (source as u32).into();
            let target = (target as u32).into();
            graph.add_edge(source, target, edge);
        }

        graph
    }
}

impl<K> GraphBuilder<K, Option<Generator>, Rewrite>
where
    K: Clone,
{
    pub fn explode<R, F>(
        self,
        f: F,
    ) -> Result<GraphBuilder<R, Option<Generator>, Rewrite>, DimensionError>
    where
        F: Fn(usize, K) -> R + Copy,
        R: std::fmt::Debug + Clone,
    {
        let mut nodes_exploded: Vec<(R, Option<Generator>)> = Default::default();
        let mut edges_exploded: Vec<(usize, usize, Rewrite)> = Default::default();

        for (source, target, rewrite) in self.edges {
            let rewrite: RewriteN = rewrite.try_into()?;
            let dimension = rewrite.dimension();
            for cone in rewrite.cones() {
                let t = agreeing(vec![
                    cone.internal.slices.first().and_then(|sl| {
                        Rewrite0::try_from(sl.clone())
                            .ok()
                            .and_then(|r0| r0.target())
                    }),
                    Rewrite0::try_from(cone.internal.target.forward.clone())
                        .ok()
                        .and_then(|r0| r0.target()),
                    Rewrite0::try_from(cone.internal.target.backward.clone())
                        .ok()
                        .and_then(|r0| r0.target()),
                ]);
                let pretarget_index = nodes_exploded.len();
                let pre_t = agreeing(vec![
                    Rewrite0::try_from(cone.internal.target.forward.clone())
                        .ok()
                        .and_then(|r0| r0.source().or(t)),
                    cone.internal.source.first().and_then(|f| {
                        Rewrite0::try_from(f.forward.clone())
                            .ok()
                            .and_then(|r0| r0.source())
                    }),
                ]);
                nodes_exploded.push((f(cone.index, self.nodes[target].0.clone()), pre_t));
                let target_index = nodes_exploded.len();
                nodes_exploded.push((f(cone.index + 1, self.nodes[target].0.clone()), t));
                edges_exploded.push((
                    pretarget_index,
                    target_index,
                    cone.internal.target.forward.clone(),
                ));
                let posttarget_index = nodes_exploded.len();
                let post_t = agreeing(vec![
                    Rewrite0::try_from(cone.internal.target.backward.clone())
                        .ok()
                        .and_then(|r0| r0.source().or(t)),
                    cone.internal.source.last().and_then(|l| {
                        Rewrite0::try_from(l.backward.clone())
                            .ok()
                            .and_then(|r0| r0.source())
                    }),
                ]);
                nodes_exploded.push((f(cone.index + 2, self.nodes[target].0.clone()), post_t));
                edges_exploded.push((
                    posttarget_index,
                    target_index,
                    cone.internal.target.backward.clone(),
                ));

                // node for beginning of iterated cospan
                let presource_index = nodes_exploded.len();
                nodes_exploded.push((f(cone.index, self.nodes[source].0.clone()), pre_t));
                edges_exploded.push((
                    presource_index,
                    pretarget_index,
                    Rewrite::identity(dimension - 1),
                ));

                // node for each slice
                let cone_start_index = nodes_exploded.len();
                // edges_exploded.push((
                //     presource_index,
                //     cone_start_index,
                //     cone.internal.source[0].forward.clone(),
                // ));
                for (i, s) in cone.internal.source.iter().enumerate() {
                    // for each source cospan, insert the cospan (except the source regular) and
                    // the slice:
                    //      ^
                    //     /
                    // -> s <- r
                    let slice_index = cone.index + 2 * i + 1;
                    let slice_source = agreeing(vec![
                        Rewrite0::try_from(cone.internal.slices[i].clone())
                            .ok()
                            .and_then(|r0| r0.source()),
                        Rewrite0::try_from(s.forward.clone())
                            .ok()
                            .and_then(|r0| r0.target()),
                        Rewrite0::try_from(s.backward.clone())
                            .ok()
                            .and_then(|r0| r0.target()),
                    ]);
                    nodes_exploded.push((
                        f(cone.index + slice_index, self.nodes[source].0.clone()),
                        slice_source,
                    ));
                    let next_regular = Rewrite0::try_from(s.backward.clone())
                        .ok()
                        .and_then(|r0| r0.source().or(slice_source));
                    nodes_exploded.push((
                        f(cone.index + slice_index + 1, self.nodes[source].0.clone()),
                        next_regular,
                    ));
                    edges_exploded.push((
                        cone_start_index + slice_index - 1,
                        target_index,
                        cone.internal.slices[i].clone(),
                    ));
                    edges_exploded.push((
                        cone_start_index + slice_index - 2,
                        cone_start_index + slice_index - 1,
                        s.forward.clone(),
                    ));
                    edges_exploded.push((
                        cone_start_index + slice_index,
                        cone_start_index + slice_index - 1,
                        s.backward.clone(),
                    ));
                }

                // node for end of iterated cospan
                let postsource_index = nodes_exploded.len() - 1;
                // nodes_exploded.push((
                //     f(cone.index + cone.len() + 2, self.nodes[source].0.clone()),
                //     post_t,
                // ));
                edges_exploded.push((
                    postsource_index,
                    posttarget_index,
                    Rewrite::identity(dimension - 1),
                ));
                // edges_exploded.push((
                //     postsource_index,
                //     cone_start_index + cone.len() - 1,
                //     cone.internal.source[cone.len() - 1].backward.clone(),
                // ));
            }
        }

        Ok(GraphBuilder {
            nodes: nodes_exploded,
            edges: edges_exploded,
            dimension: self.dimension - 1,
        })
    }
}

// impl<K> GraphBuilder<K, Rewrite, Vec<usize>>
// where
//     K: Clone,
// {
//     pub fn explode<R, F>(self, f: F) -> Result<GraphBuilder<R, Rewrite, Vec<usize>>, DimensionError>
//     where
//         F: Fn(usize, K) -> R + Copy,
//     {
//         let mut nodes_exploded: Vec<(R, Rewrite)> = Default::default();
//         let mut edges_exploded: Vec<(usize, usize, Vec<usize>)> = Default::default();
//         // a node n gets exploded into multiple nodes n₀, …, nᵢ
//         // invariant: ∀ i ∈ [0, self.nodes.len()).
//         //              nodes_exploded[nodes_indices[i] ..  nodes_indices[i+1]]
//         //              are the exploded nodes originating from self.nodes[i]
//         let mut nodes_indices: Vec<usize> = vec![0];
//
//         for (key, node) in self.nodes {
//             let node: RewriteN = node.try_into()?;
//             let mut end: usize = 0;
//             for (j, cone) in node.cones().iter().enumerate() {
//                 (j != 0).then(|| {
//                     edges_exploded.push((
//                         nodes_exploded.len() - 1,
//                         nodes_exploded.len(),
//                         vec![cone.index - end],
//                     ));
//                 });
//                 for (i, slice) in cone.internal.slices.iter().enumerate() {
//                     (i != 0).then(|| {
//                         edges_exploded.push((
//                             nodes_exploded.len() - 1,
//                             nodes_exploded.len(),
//                             vec![1],
//                         ))
//                     });
//                     nodes_exploded.push((f(cone.index + i, key.clone()), slice.clone()));
//                 }
//                 nodes_indices.push(nodes_indices[nodes_indices.len() - 1] + nodes_exploded.len());
//                 end = cone.index + cone.len();
//             }
//         }
//
//         for (source, _, distance) in self.edges {
//             for i in nodes_indices[source] .. nodes_indices[source+1] {
//                 edges_exploded.push(todo!());
//             }
//         }
//
//         Ok(GraphBuilder {
//             nodes: nodes_exploded,
//             edges: edges_exploded,
//             dimension: self.dimension - 1,
//         })
//     }
// }

impl<K> GraphBuilder<K, Diagram, Rewrite>
where
    K: Clone,
{
    pub fn explode<R, F>(self, f: F) -> Result<GraphBuilder<R, Diagram, Rewrite>, DimensionError>
    where
        F: Fn(SliceIndex, K) -> R + Copy,
    {
        use Height::{Regular, Singular};

        if self.dimension == 0 {
            return Err(DimensionError);
        }

        let mut nodes_exploded: Vec<(R, Diagram)> = Vec::new();
        let mut edges_exploded: Vec<(usize, usize, Rewrite)> = Vec::new();

        // For every node index in the original graph, we record the index in `nodes_exploded` at
        // which the slices of that node start. This allows us to find the index again when
        // constructing the edges of the exploded graph.
        let mut nodes_indices: Vec<usize> = Vec::new();

        for (key, node) in self.nodes {
            let nodes_index_start = nodes_exploded.len();
            nodes_indices.push(nodes_index_start);
            let node: DiagramN = node.try_into()?;

            // Source slice
            nodes_exploded.push({
                let slice_key = f(Boundary::Source.into(), key.clone());
                let slice = node.source();
                (slice_key, slice)
            });

            // Interior slices
            for (i, slice) in node.slices().enumerate() {
                let height = Height::from_int(i).into();
                let slice_key = f(height, key.clone());
                nodes_exploded.push((slice_key, slice));
            }

            // Target slice
            nodes_exploded.push({
                let slice_key = f(Boundary::Target.into(), key.clone());
                let slice = nodes_exploded.last().unwrap().1.clone();
                (slice_key, slice)
            });

            // Identity rewrite from source slice
            edges_exploded.push((
                nodes_index_start,
                nodes_index_start + 1,
                Rewrite::identity(self.dimension - 1),
            ));

            // Rewrites between interior slices
            for (i, cospan) in node.cospans().iter().enumerate() {
                edges_exploded.push((
                    nodes_index_start + Regular(i).to_int() + 1,
                    nodes_index_start + Singular(i).to_int() + 1,
                    cospan.forward.clone(),
                ));

                edges_exploded.push((
                    nodes_index_start + Regular(i + 1).to_int() + 1,
                    nodes_index_start + Singular(i).to_int() + 1,
                    cospan.backward.clone(),
                ));
            }

            // Identity rewrite from target slice
            edges_exploded.push((
                nodes_index_start + node.size() * 2 + 2,
                nodes_index_start + node.size() * 2 + 1,
                Rewrite::identity(self.dimension - 1),
            ));
        }

        // We push a final index so that the length of any node's contribution to `nodes_exploded`
        // can be computed by subtracting the node's index from that of the next node.
        nodes_indices.push(nodes_exploded.len());

        for (source, target, rewrite) in self.edges {
            let rewrite: RewriteN = rewrite.try_into()?;

            let source_index = nodes_indices[source];
            let source_size = nodes_indices[source + 1] - source_index;
            let target_index = nodes_indices[target];
            let target_size = nodes_indices[target + 1] - target_index;

            // Identity rewrite between source slices
            edges_exploded.push((
                source_index,
                target_index,
                Rewrite::identity(self.dimension - 1),
            ));

            // Identity rewrite between target slices
            edges_exploded.push((
                source_index + source_size - 1,
                target_index + target_size - 1,
                Rewrite::identity(self.dimension - 1),
            ));

            // Singular slices
            for source_height in 0..(source_size - 3) / 2 {
                let target_height = rewrite.singular_image(source_height);
                edges_exploded.push((
                    source_index + Singular(source_height).to_int() + 1,
                    target_index + Singular(target_height).to_int() + 1,
                    rewrite.slice(source_height),
                ));
            }

            // Regular slices
            for target_height in 0..(target_size - 1) / 2 {
                let source_height = rewrite.regular_image(target_height);
                edges_exploded.push((
                    source_index + Regular(source_height).to_int() + 1,
                    target_index + Regular(target_height).to_int() + 1,
                    Rewrite::identity(self.dimension - 1),
                ));
            }
        }

        Ok(GraphBuilder {
            nodes: nodes_exploded,
            edges: edges_exploded,
            dimension: self.dimension - 1,
        })
    }
}
