use std::convert::{Into, TryInto};

use homotopy_common::{
    graph::{Graph, Node},
    idx::IdxVec,
};

use crate::{
    common::{Boundary, DimensionError, Height, RegularHeight, SliceIndex},
    diagram::{Diagram, DiagramN},
    rewrite::{DefaultAllocator, GenericRewrite, GenericRewriteN, RewriteAllocator},
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

pub type SliceGraph<A = DefaultAllocator> = Graph<(Coord, Diagram), GenericRewrite<A>>;

pub struct GraphBuilder;

impl GraphBuilder {
    pub fn build<A, T>(diagram: Diagram, depth: usize) -> Result<SliceGraph<A>, DimensionError>
    where
        A: RewriteAllocator<Payload = T>,
        T: Default,
    {
        if depth > diagram.dimension() {
            return Err(DimensionError);
        }

        let mut graph = Graph::new();
        graph.add_node((vec![], diagram));

        for _ in 0..depth {
            graph = explode(&graph, |r, _| GenericRewrite::identity(r.dimension() - 1))?;
        }

        Ok(graph)
    }
}

pub fn explode<A, T, F>(graph: &SliceGraph<A>, f: F) -> Result<SliceGraph<A>, DimensionError>
where
    A: RewriteAllocator<Payload = T>,
    T: Default,
    F: Fn(&GenericRewriteN<A>, RegularHeight) -> GenericRewrite<A>,
{
    use Height::{Regular, Singular};

    let mut exploded_graph: SliceGraph<A> = Graph::new();

    // Maps every node in the original graph to its slices in the exploded graph.
    let mut node_to_slices: IdxVec<Node, Vec<Node>> = IdxVec::with_capacity(graph.node_count());

    for nd in graph.nodes_values() {
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
            let slice_coord = mk_coord(coord, Height::from_int(i));
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
            GenericRewrite::identity(diagram.dimension() - 1),
        );

        // Rewrites between interior slices
        for (i, cospan) in diagram.cospans().iter().enumerate() {
            exploded_graph.add_edge(
                slices[Regular(i).to_int() + 1],
                slices[Singular(i).to_int() + 1],
                cospan.forward.convert(),
            );

            exploded_graph.add_edge(
                slices[Regular(i + 1).to_int() + 1],
                slices[Singular(i).to_int() + 1],
                cospan.backward.convert(),
            );
        }

        // Identity rewrite from target slice
        exploded_graph.add_edge(
            slices[diagram.size() * 2 + 2],
            slices[diagram.size() * 2 + 1],
            GenericRewrite::identity(diagram.dimension() - 1),
        );

        node_to_slices.push(slices);
    }

    for ed in graph.edges_values() {
        let rewrite: &GenericRewriteN<_> = ed.inner().try_into()?;

        let source_slices = &node_to_slices[ed.source()];
        let source_size = source_slices.len();
        let target_slices = &node_to_slices[ed.target()];
        let target_size = target_slices.len();

        // Identity rewrite between source slices
        exploded_graph.add_edge(source_slices[0], target_slices[0], f(rewrite, 0));

        // Identity rewrite between target slices
        exploded_graph.add_edge(
            source_slices[source_size - 1],
            target_slices[target_size - 1],
            f(rewrite, target_size - 1),
        );

        // Singular slices
        for source_height in 0..(source_size - 3) / 2 {
            let target_height = rewrite.singular_image(source_height);
            exploded_graph.add_edge(
                source_slices[Singular(source_height).to_int() + 1],
                target_slices[Singular(target_height).to_int() + 1],
                rewrite.slice(source_height),
            );
        }

        // Regular slices
        for target_height in 0..(target_size - 1) / 2 {
            let source_height = rewrite.regular_image(target_height);
            exploded_graph.add_edge(
                source_slices[Regular(source_height).to_int() + 1],
                target_slices[Regular(target_height).to_int() + 1],
                f(rewrite, target_height),
            );
        }
    }

    Ok(exploded_graph)
}

pub struct TopologicalSort(Vec<Node>);

impl TopologicalSort {
    pub fn new<A>(graph: &SliceGraph<A>) -> Self
    where
        A: RewriteAllocator,
    {
        let mut nodes: Vec<Node> = graph.nodes_keys().collect();
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
