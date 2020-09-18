use crate::common::*;
use crate::diagram::*;
use crate::rewrite::*;
use petgraph::algo::tarjan_scc;
use petgraph::graphmap::{DiGraphMap, GraphMap};
use petgraph::unionfind::UnionFind;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone)]
struct Span(Rewrite, Diagram, Rewrite);

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord)]
struct Node(usize, usize);

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord)]
struct Component(usize);

type BiasValue = usize;

#[derive(Debug, Clone, Copy)]
pub enum Bias {
    Higher,
    Lower,
}

pub fn contract(
    diagram: &DiagramN,
    height: SingularHeight,
    bias: Option<Bias>,
) -> Option<RewriteN> {
    let slices: Vec<_> = diagram.slices().collect();
    let cospans = diagram.cospans();

    let cospan0 = cospans.get(height)?;
    let cospan1 = cospans.get(height + 1)?;

    let singular0 = slices.get(height * 2 + 1)?;
    let regular = slices.get(height * 2 + 2)?;
    let singular1 = slices.get(height * 2 + 3)?;

    let span = Span(
        cospan0.backward.clone(),
        regular.clone(),
        cospan1.forward.clone(),
    );

    let (bias0, bias1) = match bias {
        None => (0, 0),
        Some(Bias::Higher) => (1, 0),
        Some(Bias::Lower) => (0, 1),
    };

    let result = colimit(
        &[(singular0.clone(), bias0), (singular1.clone(), bias1)],
        &[(0, span, 1)],
    )?;

    let rewrite = RewriteN::new(
        diagram.dimension(),
        vec![Cone {
            index: height,
            target: Cospan {
                forward: Rewrite::compose(cospan0.forward.clone(), result[0].clone()).unwrap(),
                backward: Rewrite::compose(cospan1.backward.clone(), result[1].clone()).unwrap(),
            },
            source: vec![cospan0.clone(), cospan1.clone()],
            slices: result,
        }],
    );

    Some(rewrite)
}

pub fn contract_in_path(
    diagram: &DiagramN,
    path: &[Height],
    height: SingularHeight,
    bias: Option<Bias>,
) -> Option<RewriteN> {
    match path.split_first() {
        None => contract(diagram, height, bias),
        Some((step, rest)) => {
            let slice = diagram.slice(*step)?.to_n()?.clone();
            let rewrite = contract_in_path(&slice, rest, height, bias)?;
            match step {
                Height::Regular(i) => Some(RewriteN::new(
                    diagram.dimension(),
                    vec![Cone {
                        index: *i,
                        source: vec![],
                        target: Cospan {
                            forward: rewrite.clone().into(),
                            backward: rewrite.into(),
                        },
                        slices: vec![],
                    }],
                )),
                Height::Singular(i) => {
                    let source_cospan = &diagram.cospans()[*i];
                    Some(RewriteN::new(
                        diagram.dimension(),
                        vec![Cone {
                            index: *i,
                            source: vec![source_cospan.clone()],
                            target: Cospan {
                                forward: Rewrite::compose(
                                    source_cospan.forward.clone(),
                                    rewrite.clone().into(),
                                )
                                .unwrap(),
                                backward: Rewrite::compose(
                                    source_cospan.backward.clone(),
                                    rewrite.clone().into(),
                                )
                                .unwrap(),
                            },
                            slices: vec![rewrite.into()],
                        }],
                    ))
                }
            }
        }
    }
}

fn colimit(
    diagrams: &[(Diagram, BiasValue)],
    spans: &[(usize, Span, usize)],
) -> Option<Vec<Rewrite>> {
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
    diagrams: &[(Diagram, BiasValue)],
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
                    Rewrite0::new(diagram.to_generator().unwrap(), max_generator).into()
                })
                .collect(),
        )
    } else {
        None
    }
}

fn colimit_recursive(
    diagrams: &[(Diagram, BiasValue)],
    spans: &[(usize, Span, usize)],
) -> Option<Vec<Rewrite>> {
    let mut span_slices: HashMap<(Node, Node), Span> = HashMap::new();
    let mut diagram_slices: HashMap<Node, Diagram> = HashMap::new();
    let mut node_to_cospan: HashMap<Node, Cospan> = HashMap::new();

    let mut delta: DiGraphMap<Node, ()> = GraphMap::new();

    // Explode the input diagrams into their singular slices and the spans between them.
    for (key, (diagram, _)) in diagrams.iter().enumerate() {
        let diagram = diagram.to_n().unwrap();
        let slices: Vec<_> = diagram.slices().collect();
        let cospans = diagram.cospans();

        for height in 0..diagram.size() {
            delta.add_node(Node(key, height));
            diagram_slices.insert(Node(key, height), slices[height * 2 + 1].clone());
            node_to_cospan.insert(Node(key, height), cospans[height].clone());
        }

        for height in 1..diagram.size() {
            let slice = slices[height * 2].clone();
            let backward = cospans[height].backward.clone();
            let forward = cospans[height].forward.clone();

            span_slices.insert(
                (Node(key, height - 1), Node(key, height)),
                Span(backward, slice, forward),
            );
            delta.add_edge(Node(key, height - 1), Node(key, height), ());
        }
    }

    // Explode the input spans into slice spans.
    for (source, span, target) in spans.iter() {
        let Span(backward, diagram, forward) = span;
        let backward = backward.to_n().unwrap();
        let forward = forward.to_n().unwrap();
        let diagram = diagram.to_n().unwrap();
        let slices: Vec<_> = diagram.slices().collect();

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
    let mut scc_to_nodes = tarjan_scc(&delta);
    scc_to_nodes.reverse();

    // Associate to every node its component
    let node_to_scc: HashMap<Node, Component> = scc_to_nodes
        .iter()
        .enumerate()
        .flat_map(|(i, nodes)| nodes.iter().map(move |node| (*node, Component(i))))
        .collect();

    // Contract the graph to a DAG
    let mut scc_graph: DiGraphMap<Component, ()> = GraphMap::new();
    let mut scc_spans: HashMap<Component, Vec<(Node, Span, Node)>> = HashMap::new();

    for scc in 0..scc_to_nodes.len() {
        scc_graph.add_node(Component(scc));
    }

    for (s, t, _) in delta.all_edges() {
        let s = node_to_scc[&s];
        let t = node_to_scc[&t];

        if s != t {
            scc_graph.add_edge(s, t, ());
        }
    }

    for ((s, t), span) in &span_slices {
        let s_component = node_to_scc[&s];
        let t_component = node_to_scc[&t];

        if s_component == t_component {
            scc_spans
                .entry(s_component)
                .or_insert_with(Vec::new)
                .push((*s, span.clone(), *t));
        }
    }

    // Linearize the DAG if possible.
    let scc_to_priority: HashMap<Component, (usize, BiasValue)> = {
        // This bit relies on the topological order of the strongly connected components
        // to compute the shortest distance from a root.
        let mut scc_to_priority: HashMap<Component, (usize, BiasValue)> = HashMap::new();

        for (scc, _) in scc_to_nodes.iter().enumerate() {
            let distance = scc_graph
                .neighbors_directed(Component(scc), petgraph::Direction::Incoming)
                .filter(|p| p.0 != scc)
                .map(|p| scc_to_priority[&p].0 + 1)
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
        let mut components: Vec<_> = (0..scc_to_nodes.len()).map(Component).collect();
        components.sort_by_key(|scc| scc_to_priority[scc]);
        components
    };

    if !is_strictly_increasing(&components_sorted, |scc| scc_to_priority[scc]) {
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
        let nodes = &scc_to_nodes[component.0];

        let diagrams: Vec<(Diagram, BiasValue)> = nodes
            .iter()
            .map(|node| (diagram_slices[node].clone(), 0))
            .collect();

        let spans: Vec<(usize, Span, usize)> = scc_spans
            .get(&component)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|(s, span, t)| {
                let s_index = nodes.iter().position(|n| *n == s).unwrap();
                let t_index = nodes.iter().position(|n| *n == t).unwrap();
                (s_index, span, t_index)
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
        let mut nodes = scc_to_nodes[component.0].clone();
        nodes.sort();

        let first = nodes.first().unwrap();
        let last = nodes.last().unwrap();

        let forward = Rewrite::compose(
            node_to_cospan[first].forward.clone(),
            rewrite_slices[first].clone(),
        )
        .unwrap();
        let backward = Rewrite::compose(
            node_to_cospan[last].backward.clone(),
            rewrite_slices[last].clone(),
        )
        .unwrap();

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
            diagram.dimension(),
            diagram.cospans(),
            target.cospans(),
            slices,
        )));
    }

    Some(rewrites)
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
    true
}

mod test {
    use super::*;

    #[test]
    fn beads() {
        let x = Generator {
            id: 0,
            dimension: 0,
        };
        let f = Generator {
            id: 1,
            dimension: 1,
        };
        let p = Generator {
            id: 2,
            dimension: 2,
        };

        let fd = DiagramN::new(f, x, x);
        let pd = DiagramN::new(p, fd.clone(), fd.clone());

        let pfd = pd.attach(fd, Boundary::Target, &[]).unwrap();
        let left = pfd.attach(pd.clone(), Boundary::Source, &[1]).unwrap();
        let right = pfd.attach(pd, Boundary::Target, &[1]).unwrap();

        let left_contract = contract(&left, 0, None).expect("failed to contract left diagram");
        let right_contract = contract(&right, 0, None).expect("failed to contract right diagram");

        let left_to_right = DiagramN::new_unsafe(
            left.clone().into(),
            vec![Cospan {
                forward: left_contract.clone().into(),
                backward: right_contract.clone().into(),
            }],
        );

        let right_to_left = DiagramN::new_unsafe(
            right.clone().into(),
            vec![Cospan {
                forward: right_contract.into(),
                backward: left_contract.into(),
            }],
        );

        assert_eq!(left_to_right.target(), right.into());
        assert_eq!(right_to_left.target(), left.into());
        assert_eq!(
            left_to_right.slice(Height::Singular(0)).unwrap(),
            right_to_left.slice(Height::Singular(0)).unwrap()
        );
    }
}
