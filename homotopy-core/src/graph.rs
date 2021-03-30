use crate::common::{Boundary, DimensionError, Height, SliceIndex};
use crate::diagram::{Diagram, DiagramN};
use crate::rewrite::{Rewrite, RewriteN};
use std::convert::{Into, TryInto};

#[derive(Debug, Clone)]
pub struct GraphBuilder<K> {
    pub nodes: Vec<(K, Diagram)>,
    pub edges: Vec<(usize, usize, Rewrite)>,
    dimension: usize,
}

impl<K> GraphBuilder<K> {
    pub fn new(key: K, diagram: Diagram) -> Self {
        Self {
            dimension: diagram.dimension(),
            nodes: vec![(key, diagram)],
            edges: vec![],
        }
    }
}

impl<K> GraphBuilder<K>
where
    K: Clone,
{
    pub fn explode<R, F>(self, f: F) -> Result<GraphBuilder<R>, DimensionError>
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
