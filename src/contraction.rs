use crate::diagram::*;
use crate::common::*;
use crate::rewrite::*;
use crate::util::graph::Graph;
use crate::util::union_find::UnionFind;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone)]
struct Span(Rewrite, Diagram, Rewrite);

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord)]
struct Node(usize, usize);

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
struct Component(usize);

type Bias = usize;

fn colimit(diagrams: &[(Diagram, Bias)], spans: &[(usize, Span, usize)]) -> Option<Vec<Rewrite>> {
    let dimension = diagrams[0].0.dimension();

    for diagram in diagrams {
        if diagram.0.dimension() != dimension {
            panic!();
        }
    }

    for (_, Span(backward, diagram, forward), _) in spans {
        if backward.dimension() != dimension
            || diagram.dimension() != dimension
            || forward.dimension() != dimension
        {
            panic!()
        }
    }

    if dimension == 0 {
        colimit_base(diagrams, spans)
    } else {
        colimit_recursive(diagrams, spans)
    }
}

fn colimit_base(
    diagrams: &[(Diagram, Bias)],
    spans: &[(usize, Span, usize)],
) -> Option<Vec<Rewrite>> {
    let mut union_find = UnionFind::new(diagrams.len());

    for (source, Span(_, diagram, _), target) in spans {
        let source_diagram = &diagrams[*source].0;
        let target_diagram = &diagrams[*target].0;

        if source_diagram == diagram && diagram == target_diagram {
            union_find.union(*source, *target);
        }
    }

    let max_generator = diagrams
        .iter()
        .map(|(d, _)| d.to_generator().unwrap())
        .max_by_key(|g| g.dimension)
        .unwrap();

    let mut components: Vec<_> = diagrams
        .iter()
        .enumerate()
        .map(|(i, (d, _))| (union_find.find(i), d.to_generator().unwrap().dimension))
        .filter(|(_, dimension)| *dimension == max_generator.dimension)
        .map(|(component, _)| component)
        .collect();

    components.dedup();

    if components.len() == 1 {
        Some(
            diagrams
                .iter()
                .map(|(diagram, _)| {
                    Rewrite::Rewrite0(diagram.to_generator().unwrap(), max_generator)
                })
                .collect(),
        )
    } else {
        None
    }
}

fn colimit_recursive(
    diagrams: &[(Diagram, Bias)],
    spans: &[(usize, Span, usize)],
) -> Option<Vec<Rewrite>> {
    let mut span_slices: HashMap<(Node, Node), Span> = HashMap::new();
    let mut diagram_slices: HashMap<Node, Diagram> = HashMap::new();
    let mut node_to_cospan: HashMap<Node, Cospan> = HashMap::new();
    let mut delta: Graph<Node, (), ()> = Graph::new();

    // Explode the input diagrams into their singular slices and the spans between them.
    for (key, (diagram, _)) in diagrams.iter().enumerate() {
        let diagram = diagram.to_n().unwrap();
        let slices = diagram.slices();
        let cospans = diagram.cospans();

        for height in 0..diagram.size() {
            delta.add_node(Node(key, height), ());
            diagram_slices.insert(Node(key, height), slices[height * 2 + 1].clone());
            node_to_cospan.insert(Node(key, height), cospans[height].clone());
        }

        for height in 1..diagram.size() {
            let slice = slices[height * 2].clone();
            let backward = cospans[height].backward.clone();
            let forward = cospans[height].forward.clone();

            span_slices.insert(
                (Node(key, height), Node(key, height - 1)),
                Span(backward, slice, forward),
            );
            delta.add_edge(Node(key, height), Node(key, height - 1), ());
        }
    }

    // Explode the input spans into slice spans.
    for (source, span, target) in spans.iter() {
        let Span(backward, diagram, forward) = span;
        let backward = backward.to_n().unwrap();
        let forward = forward.to_n().unwrap();
        let diagram = diagram.to_n().unwrap();
        let slices = diagram.slices();

        for height in 0..diagram.size() {
            let slice = slices[height * 2 + 1].clone();
            let source_node = Node(*source, backward.singular_image(height));
            let target_node = Node(*target, forward.singular_image(height));

            let span = Span(backward.slice(height), slice, forward.slice(height));

            span_slices.insert((source_node, target_node), span);
            delta.add_edge(source_node, target_node, ());
            delta.add_edge(target_node, source_node, ());
        }
    }

    // Get the strongly connected components of the delta graph in topological order.
    let mut scc_to_nodes = delta.sccs();
    scc_to_nodes.reverse();

    // Associate to every node its component
    let node_to_scc: HashMap<Node, Component> = scc_to_nodes
        .iter()
        .enumerate()
        .flat_map(|(i, nodes)| nodes.into_iter().map(move |node| (*node, Component(i))))
        .collect();

    // Contract the graph to a DAG
    let scc_graph: Graph<Component, Vec<Node>, Vec<(Node, Node)>> =
        delta.contract(|node| node_to_scc[node]);

    // Linearize the DAG if possible.
    let scc_to_priority: HashMap<Component, (usize, Bias)> = {
        // This bit relies on the topological order of the strongly connected components
        // to compute the shortest distance from a root.
        let mut scc_to_priority: HashMap<Component, (usize, Bias)> = HashMap::new();

        for scc in 0..scc_to_nodes.len() {
            let distance = scc_graph
                .pred(&Component(scc))
                .iter()
                .map(|p| scc_to_priority[p].0)
                .fold(0, std::cmp::max);

            let bias = scc_to_nodes[scc]
                .iter()
                .map(|Node(key, _)| diagrams[*key].1)
                .fold(0, std::cmp::min);

            scc_to_priority.insert(Component(scc), (distance, bias));
        }

        scc_to_priority
    };

    let components_sorted: Vec<Component> = {
        let mut components: Vec<_> = (0..scc_to_nodes.len()).map(|i| Component(i)).collect();
        components.sort_by_key(|scc| scc_to_priority[scc]);
        components
    };

    if !is_strictly_increasing(&components_sorted, |scc| scc_to_priority[scc]) {
        // Ambiguous
        // TODO: Add bias to priority
        return None;
    }

    let component_to_index: HashMap<Component, usize> = components_sorted
        .iter()
        .enumerate()
        .map(|(index, scc)| (*scc, index))
        .collect();

    // Solve the recursive subproblems
    let mut rewrite_slices: HashMap<Node, Rewrite> = HashMap::new();

    for component in scc_graph.nodes() {
        let nodes = scc_graph.get_node(&component).unwrap();

        // Every node in the strongly connected component corresponds to a slice of the input
        // diagrams. We assign a uniformly zero bias to avoid mysterious and arbitrary resolving of
        // ambiguity in deeper levels.
        let diagrams: Vec<(Diagram, Bias)> = nodes
            .iter()
            .map(|node| (diagram_slices[node].clone(), 0))
            .collect();

        // Edges between nodes in the component correspond to slices of the input spans. We filter
        // out the backwards edges that we added above to ensure that nodes connected by a slice
        // of a span end up in the same strongly connected component.
        let spans: Vec<(usize, Span, usize)> = scc_graph
            .get_edge(&component, &component)
            .cloned()
            .unwrap_or(Vec::new())
            .iter()
            .filter_map(|(s, t)| {
                let s_index = nodes.iter().position(|n| n == s).unwrap();
                let t_index = nodes.iter().position(|n| n == t).unwrap();
                span_slices
                    .get(&(*s, *t))
                    .map(|span| (s_index, span.clone(), t_index))
            })
            .collect();

        let result = colimit(&diagrams, &spans)?;

        for (i, factor) in result.into_iter().enumerate() {
            rewrite_slices.insert(nodes[i], factor);
        }
    }

    // Assemble cospans of the target diagram
    let mut cospans: Vec<Cospan> = Vec::new();

    for component in &components_sorted {
        // Sort the nodes. This will assure that the forward rewrite into the first node
        // and the backward rewrite into the last node are rewrites at the boundaries
        // of the component and thus can be used to determine the target cospan.
        let mut nodes = scc_graph.get_node(component).unwrap().clone();
        nodes.sort();

        let first = nodes.first().unwrap();
        let last = nodes.last().unwrap();

        let forward = Rewrite::compose(
            node_to_cospan[first].forward.clone(),
            rewrite_slices[first].clone(),
        );
        let backward = Rewrite::compose(
            node_to_cospan[last].backward.clone(),
            rewrite_slices[last].clone(),
        );

        cospans.push(Cospan { forward, backward });
    }

    let target = DiagramN::new_unsafe(diagrams[0].0.to_n().unwrap().source(), cospans);

    // Assemble the rewrites
    let mut rewrites: Vec<Rewrite> = Vec::new();

    for (key, diagram) in diagrams.iter().enumerate() {
        let diagram = diagram.0.to_n().unwrap();
        let mut slices: Vec<Vec<Rewrite>> = (0..target.size()).map(|_| Vec::new()).collect();

        for height in 0..diagram.size() {
            let node = Node(key, height);
            let scc = node_to_scc[&node];
            let index = component_to_index[&scc];
            slices[index].push(rewrite_slices[&Node(key, height)].clone());
        }

        rewrites.push(Rewrite::RewriteN(RewriteN::from_slices(
            &diagram, &target, slices,
        )));
    }

    return Some(rewrites);
}

fn is_strictly_increasing<T, K, F>(slice: &[T], key: F) -> bool
where
    K: Ord,
    F: Fn(&T) -> K,
{
    for i in 1..slice.len() {
        if key(&slice[i - 1]) >= key(&slice[i]) {
            return false;
        }
    }
    return true;
}
