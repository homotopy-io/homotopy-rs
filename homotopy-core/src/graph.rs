use std::convert::TryInto;

use homotopy_common::{
    graph::{Graph, Node},
    idx::IdxVec,
};

use crate::{
    common::{Boundary, DimensionError, Height, SliceIndex},
    diagram::{Diagram, DiagramN},
    rewrite::{Rewrite, RewriteN},
};

pub type Coord = Vec<SliceIndex>;

pub fn mk_coord<I>(coord: &[SliceIndex], index: I) -> Coord
where
    I: Into<SliceIndex>,
{
    let mut coord = coord.to_owned();
    coord.push(index.into());
    coord
}

pub type SliceGraph = Graph<(Coord, Diagram), Rewrite>;

pub struct GraphBuilder;

impl GraphBuilder {
    pub fn build(diagram: Diagram, depth: usize) -> Result<SliceGraph, DimensionError> {
        if depth > diagram.dimension() {
            return Err(DimensionError);
        }

        let mut graph = Graph::new();
        graph.add_node((vec![], diagram));

        for _ in 0..depth {
            graph = explode_graph(&graph)?.0;
        }

        Ok(graph)
    }
}

pub fn explode_graph(
    graph: &SliceGraph,
) -> Result<(SliceGraph, IdxVec<Node, Vec<Node>>), DimensionError> {
    use Height::{Regular, Singular};

    let mut exploded_graph: SliceGraph = Graph::new();

    // Maps every node in the original graph to its slices in the exploded graph.
    let mut node_to_slices: IdxVec<Node, Vec<Node>> = IdxVec::with_capacity(graph.node_count());

    for nd in graph.node_values() {
        let coord: &Coord = &nd.0;
        let diagram: &DiagramN = (&nd.1).try_into()?;

        let mut slices = Vec::with_capacity(diagram.size() * 2 + 3);

        // Source slice
        slices.push(exploded_graph.add_node({
            let slice_coord = mk_coord(coord, Boundary::Source);
            let slice = diagram.source();
            (slice_coord, slice)
        }));

        // Interior slices
        for (i, slice) in diagram.slices().enumerate() {
            let slice_coord = mk_coord(coord, Height::from(i));
            slices.push(exploded_graph.add_node((slice_coord, slice)));
        }

        // Target slice
        slices.push(exploded_graph.add_node({
            let slice_coord = mk_coord(coord, Boundary::Target);
            let slice = diagram.target();
            (slice_coord, slice)
        }));

        // Identity rewrite from source slice
        exploded_graph.add_edge(
            slices[0],
            slices[1],
            Rewrite::identity(diagram.dimension() - 1),
        );

        // Rewrites between interior slices
        for (i, cospan) in diagram.cospans().iter().enumerate() {
            exploded_graph.add_edge(
                slices[usize::from(Regular(i)) + 1],
                slices[usize::from(Singular(i)) + 1],
                cospan.forward.clone(),
            );

            exploded_graph.add_edge(
                slices[usize::from(Regular(i + 1)) + 1],
                slices[usize::from(Singular(i)) + 1],
                cospan.backward.clone(),
            );
        }

        // Identity rewrite from target slice
        exploded_graph.add_edge(
            slices[diagram.size() * 2 + 2],
            slices[diagram.size() * 2 + 1],
            Rewrite::identity(diagram.dimension() - 1),
        );

        node_to_slices.push(slices);
    }

    for ed in graph.edge_values() {
        let s = ed.source();
        let t = ed.target();
        let rewrite: &RewriteN = ed.inner().try_into()?;

        let source_diagram: &DiagramN = (&graph[s].1).try_into()?;
        let source_slices = &node_to_slices[s];
        let source_size = source_slices.len();
        let target_diagram: &DiagramN = (&graph[t].1).try_into()?;
        let target_slices = &node_to_slices[t];
        let target_size = target_slices.len();

        // Identity rewrite between source slices
        exploded_graph.add_edge(
            source_slices[0],
            target_slices[0],
            Rewrite::identity(rewrite.dimension() - 1),
        );

        // Identity rewrite between target slices
        exploded_graph.add_edge(
            source_slices[source_size - 1],
            target_slices[target_size - 1],
            Rewrite::identity(rewrite.dimension() - 1),
        );

        // Singular slices
        for source_height in 0..(source_size - 3) / 2 {
            let target_height = rewrite.singular_image(source_height);
            exploded_graph.add_edge(
                source_slices[usize::from(Singular(source_height)) + 1],
                target_slices[usize::from(Singular(target_height)) + 1],
                rewrite.slice(source_height),
            );
        }

        // Regular slices
        for target_height in 0..(target_size - 1) / 2 {
            let source_height = rewrite.regular_image(target_height);
            exploded_graph.add_edge(
                source_slices[usize::from(Regular(source_height)) + 1],
                target_slices[usize::from(Regular(target_height)) + 1],
                Rewrite::identity(rewrite.dimension() - 1),
            );
        }

        // Composite slices
        for source_height in 0..(source_size - 1) / 2 {
            let preimage = rewrite.regular_preimage(source_height);
            if preimage.is_empty() {
                let target_height = preimage.start;
                exploded_graph.add_edge(
                    source_slices[usize::from(Regular(source_height)) + 1],
                    target_slices[usize::from(Singular(target_height)) + 1],
                    source_diagram.cospans()[source_height]
                        .forward
                        .compose(&rewrite.slice(source_height))
                        .unwrap(),
                );
            }
        }
        for target_height in 0..(target_size - 3) / 2 {
            let preimage = rewrite.singular_preimage(target_height);
            if preimage.is_empty() {
                let source_height = preimage.start;
                exploded_graph.add_edge(
                    source_slices[usize::from(Regular(source_height)) + 1],
                    target_slices[usize::from(Singular(target_height)) + 1],
                    target_diagram.cospans()[target_height].forward.clone(),
                );
            }
        }
    }

    Ok((exploded_graph, node_to_slices))
}

pub struct TopologicalSort(Vec<Node>);

impl TopologicalSort {
    pub fn new(graph: &SliceGraph) -> Self {
        let mut nodes: Vec<Node> = graph.node_keys().collect();
        nodes.sort_by_cached_key(|&n| {
            graph[n]
                .0
                .iter()
                .map(|&index| match index {
                    SliceIndex::Boundary(_) => -1,
                    SliceIndex::Interior(Height::Regular(_)) => 0,
                    SliceIndex::Interior(Height::Singular(_)) => 1,
                })
                .sum::<i32>()
        });
        Self(nodes)
    }
}

impl IntoIterator for TopologicalSort {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = Node;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
