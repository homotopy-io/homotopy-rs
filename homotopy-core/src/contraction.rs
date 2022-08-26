use std::{
    convert::{Into, TryInto},
    hash::Hash,
};

use homotopy_common::hash::{FastHashMap, FastHashSet};
use petgraph::{
    algo::kosaraju_scc,
    graph::{DiGraph, NodeIndex},
    graphmap::{DiGraphMap, GraphMap},
    unionfind::UnionFind,
    visit::EdgeRef,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    attach::{attach, BoundaryPath},
    common::{Boundary, DimensionError, Height, SingularHeight},
    diagram::{Diagram, DiagramN},
    normalization,
    rewrite::{Cone, Cospan, Rewrite, Rewrite0, RewriteN},
    signature::Signature,
    typecheck::{typecheck_cospan, TypeError},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Span(Rewrite, Diagram, Rewrite);

impl Span {
    fn is_identity(&self) -> bool {
        self.0.is_identity() && self.2.is_identity()
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord)]
struct Node(usize, usize);

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord)]
struct Component(usize);

type BiasValue = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Bias {
    Higher,
    Lower,
}

impl Bias {
    #[must_use]
    pub fn flip(self) -> Self {
        match self {
            Self::Higher => Self::Lower,
            Self::Lower => Self::Higher,
        }
    }
}

#[derive(Debug, Error)]
pub enum ContractionError {
    #[error("contraction invalid")]
    Invalid,
    #[error("contraction ambiguous")]
    Ambiguous,
    #[error("contraction fails to typecheck: {0}")]
    IllTyped(#[from] TypeError),
    #[error("invalid boundary path provided to contraction")]
    Dimension(#[from] DimensionError),
}

impl DiagramN {
    pub fn contract<S>(
        &self,
        boundary_path: BoundaryPath,
        interior_path: &[Height],
        height: SingularHeight,
        bias: Option<Bias>,
        signature: &S,
    ) -> Result<Self, ContractionError>
    where
        S: Signature,
    {
        if boundary_path.1 >= self.dimension() {
            return Err(ContractionError::Invalid);
        }

        attach(self, boundary_path, |slice| {
            let slice = slice.try_into().or(Err(ContractionError::Invalid))?;
            let contract = contract_in_path(&slice, interior_path, height, bias)?;
            let singular = slice.clone().rewrite_forward(&contract).unwrap();
            let normalize = normalization::normalize_singular(&singular.into());

            let cospan = match boundary_path.boundary() {
                Boundary::Source => Cospan {
                    forward: normalize,
                    backward: contract.into(),
                },
                Boundary::Target => Cospan {
                    forward: contract.into(),
                    backward: normalize,
                },
            };

            typecheck_cospan(
                slice.into(),
                cospan.clone(),
                boundary_path.boundary(),
                signature,
            )?;

            Ok(vec![cospan])
        })
    }
}

fn contract_base(
    diagram: &DiagramN,
    height: SingularHeight,
    bias: Option<Bias>,
) -> Result<RewriteN, ContractionError> {
    let slices: Vec<_> = diagram.slices().collect();
    let cospans = diagram.cospans();

    let cospan0 = cospans.get(height).ok_or(ContractionError::Invalid)?;
    let cospan1 = cospans.get(height + 1).ok_or(ContractionError::Invalid)?;

    let singular0 = slices
        .get(height * 2 + 1)
        .ok_or(ContractionError::Invalid)?;
    let regular = slices
        .get(height * 2 + 2)
        .ok_or(ContractionError::Invalid)?;
    let singular1 = slices
        .get(height * 2 + 3)
        .ok_or(ContractionError::Invalid)?;

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
        vec![Cone::new(
            height,
            vec![cospan0.clone(), cospan1.clone()],
            Cospan {
                forward: cospan0.forward.compose(&result[0]).unwrap(),
                backward: cospan1.backward.compose(&result[1]).unwrap(),
            },
            result,
        )],
    );

    Ok(rewrite)
}

fn contract_in_path(
    diagram: &DiagramN,
    path: &[Height],
    height: SingularHeight,
    bias: Option<Bias>,
) -> Result<RewriteN, ContractionError> {
    match path.split_first() {
        None => contract_base(diagram, height, bias),
        Some((step, rest)) => {
            let slice: DiagramN = diagram
                .slice(*step)
                .ok_or(ContractionError::Invalid)?
                .try_into()
                .ok()
                .ok_or(ContractionError::Invalid)?;
            let rewrite = contract_in_path(&slice, rest, height, bias)?;
            match step {
                Height::Regular(i) => Ok(RewriteN::new(
                    diagram.dimension(),
                    vec![Cone::new(
                        *i,
                        vec![],
                        Cospan {
                            forward: rewrite.clone().into(),
                            backward: rewrite.into(),
                        },
                        vec![],
                    )],
                )),
                Height::Singular(i) => {
                    let source_cospan = &diagram.cospans()[*i];
                    let rewrite = rewrite.into();
                    Ok(RewriteN::new(
                        diagram.dimension(),
                        vec![Cone::new(
                            *i,
                            vec![source_cospan.clone()],
                            Cospan {
                                forward: source_cospan.forward.compose(&rewrite).unwrap(),
                                backward: source_cospan.backward.compose(&rewrite).unwrap(),
                            },
                            vec![rewrite],
                        )],
                    ))
                }
            }
        }
    }
}

fn colimit(
    diagrams: &[(Diagram, BiasValue)],
    spans: &[(usize, Span, usize)],
) -> Result<Vec<Rewrite>, ContractionError> {
    let dimension = diagrams[0].0.dimension();

    for diagram in diagrams {
        assert_eq!(diagram.0.dimension(), dimension);
    }

    for (_, Span(backward, diagram, forward), _) in spans {
        if backward.dimension() != dimension
            || diagram.dimension() != dimension
            || forward.dimension() != dimension
        {
            panic!();
        }
    }

    if dimension == 0 {
        colimit_base(diagrams, spans)
    } else {
        colimit_recursive_simplify(diagrams, spans)
    }
}

fn colimit_base(
    diagrams: &[(Diagram, BiasValue)],
    spans: &[(usize, Span, usize)],
) -> Result<Vec<Rewrite>, ContractionError> {
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
        Ok(diagrams
            .iter()
            .map(|(diagram, _)| {
                Rewrite0::new(diagram.to_generator().unwrap(), max_generator).into()
            })
            .collect())
    } else {
        Err(ContractionError::Invalid)
    }
}

/// Solve the recursive step of the contraction algorithm by simplifying the problem first and then
/// calling `colimit_recursive_simplify`. The simplification step identitifies diagrams that are
/// connected with an identity span.
fn colimit_recursive_simplify(
    diagrams: &[(Diagram, BiasValue)],
    spans: &[(usize, Span, usize)],
) -> Result<Vec<Rewrite>, ContractionError> {
    // If every span consists just of identities then also all diagrams must be equal
    // so we can just return the identity rewrite for all of them.
    if spans.iter().all(|span| span.1.is_identity()) {
        let dimension = diagrams[0].0.dimension();
        return Ok((0..diagrams.len())
            .map(|_| Rewrite::identity(dimension))
            .collect());
    }

    // Simplify the problem by treating diagrams that are connected by an identity span as the
    // same. Also keep track of how the index of a diagram in the original problem maps to an
    // index of a diagram in the simplified one.
    let (diagrams_simplified, original_to_simplified) = {
        let mut canonical_index = UnionFind::<usize>::new(diagrams.len());
        let mut biases: Vec<BiasValue> = diagrams.iter().map(|(_, bias)| *bias).collect();

        for (source, span, target) in spans {
            if span.is_identity() {
                let source_bias = biases[canonical_index.find(*source)];
                let target_bias = biases[canonical_index.find(*target)];
                canonical_index.union(*source, *target);
                biases[canonical_index.find(*source)] = std::cmp::min(source_bias, target_bias);
            }
        }

        let mut diagrams_simplified = Vec::with_capacity(diagrams.len());
        let mut original_to_simplified: Vec<Option<usize>> =
            (0..diagrams.len()).map(|_| None).collect();

        for i in 0..diagrams.len() {
            if i == canonical_index.find(i) {
                original_to_simplified[i] = Some(diagrams_simplified.len());
                diagrams_simplified.push((diagrams[i].0.clone(), biases[i]));
            }
        }

        let original_to_simplified = move |original_index| {
            original_to_simplified[canonical_index.find(original_index)].unwrap()
        };
        (diagrams_simplified, original_to_simplified)
    };

    // Update the spans so that their indices reference diagrams in the simplified problem
    // and filter out the identity spans. At this point there can be a lot of duplicate spans
    // so the problem can be further simplified by deduplifying.
    let spans_simplified: Vec<(usize, Span, usize)> = {
        let mut spans_dedup: FastHashSet<(usize, Span, usize)> = FastHashSet::default();

        spans
            .iter()
            .filter(|(_, span, _)| !span.is_identity())
            .cloned()
            .map(|(s, span, t)| (original_to_simplified(s), span, original_to_simplified(t)))
            .filter(|e| spans_dedup.insert(e.clone()))
            .collect()
    };

    // Solve the simplified problem.
    let solution = colimit_recursive(&diagrams_simplified, &spans_simplified)?;

    // Translate the solution of the simplified problem to a solution of the original one.
    Ok((0..diagrams.len())
        .map(|i| solution[original_to_simplified(i)].clone())
        .collect())
}

/// Solve the recursive step of the contraction algorithm.
fn colimit_recursive(
    diagrams: &[(Diagram, BiasValue)],
    spans: &[(usize, Span, usize)],
) -> Result<Vec<Rewrite>, ContractionError> {
    use Height::{Regular, Singular};
    let mut span_slices: FastHashMap<(Node, Node), Vec<Span>> = FastHashMap::default();
    let mut diagram_slices: FastHashMap<Node, Diagram> = FastHashMap::default();
    let mut node_to_cospan: FastHashMap<Node, Cospan> = FastHashMap::default();

    let mut delta: DiGraph<Node, ()> = DiGraph::new();
    let mut node_to_index: FastHashMap<Node, NodeIndex<u32>> = FastHashMap::default();

    // Explode the input diagrams into their singular slices and the spans between them.
    for (key, (diagram, _)) in diagrams.iter().enumerate() {
        let diagram: &DiagramN = diagram.try_into().unwrap();
        let slices: Vec<_> = diagram.slices().collect();
        let cospans = diagram.cospans();

        for height in 0..diagram.size() {
            let node = Node(key, height);
            let index = delta.add_node(node);
            node_to_index.insert(node, index);
            diagram_slices.insert(node, slices[usize::from(Singular(height))].clone());
            node_to_cospan.insert(node, cospans[height].clone());
        }

        for height in 1..diagram.size() {
            let slice = slices[usize::from(Regular(height))].clone();
            let backward = cospans[height - 1].backward.clone();
            let forward = cospans[height].forward.clone();

            span_slices
                .entry((Node(key, height - 1), Node(key, height)))
                .or_default()
                .push(Span(backward, slice, forward));

            let source_index = node_to_index[&Node(key, height - 1)];
            let target_index = node_to_index[&Node(key, height)];
            delta.add_edge(source_index, target_index, ());
        }
    }

    // Explode the input spans into slice spans.
    for (source, span, target) in spans.iter() {
        let Span(backward, diagram, forward) = span;
        let backward: &RewriteN = backward.try_into().unwrap();
        let forward: &RewriteN = forward.try_into().unwrap();
        let diagram: &DiagramN = diagram.try_into().unwrap();
        let slices: Vec<_> = diagram.slices().collect();

        for height in 0..diagram.size() {
            let slice = slices[usize::from(Singular(height))].clone();
            let source_node = Node(*source, backward.singular_image(height));
            let target_node = Node(*target, forward.singular_image(height));

            let span = Span(backward.slice(height), slice, forward.slice(height));

            let source_index = node_to_index[&source_node];
            let target_index = node_to_index[&target_node];

            span_slices
                .entry((source_node, target_node))
                .or_default()
                .push(span);
            delta.add_edge(source_index, target_index, ());
            delta.add_edge(target_index, source_index, ());
        }
    }

    // Get the strongly connected components of the delta graph in topological order.
    let scc_to_nodes: Vec<Vec<Node>> = kosaraju_scc(&delta)
        .into_iter()
        .rev()
        .map(|indices| {
            indices
                .into_iter()
                .map(|index| *delta.node_weight(index).unwrap())
                .collect()
        })
        .collect();

    // Associate to every node its component
    let node_to_scc: FastHashMap<Node, Component> = scc_to_nodes
        .iter()
        .enumerate()
        .flat_map(|(i, nodes)| nodes.iter().map(move |node| (*node, Component(i))))
        .collect();

    // Contract the graph to a DAG
    let mut scc_graph: DiGraphMap<Component, ()> = GraphMap::new();
    let mut scc_spans: FastHashMap<Component, Vec<(Node, Span, Node)>> = FastHashMap::default();

    for scc in 0..scc_to_nodes.len() {
        scc_graph.add_node(Component(scc));
    }

    for e in delta.edge_references() {
        let s = node_to_scc[delta.node_weight(e.source()).unwrap()];
        let t = node_to_scc[delta.node_weight(e.target()).unwrap()];

        if s != t {
            scc_graph.add_edge(s, t, ());
        }
    }

    for ((s, t), slices) in &span_slices {
        let s_component = node_to_scc[s];
        let t_component = node_to_scc[t];

        if s_component == t_component {
            for slice in slices {
                scc_spans
                    .entry(s_component)
                    .or_insert_with(Vec::new)
                    .push((*s, slice.clone(), *t));
            }
        }
    }

    // Linearize the DAG if possible.
    let scc_to_priority: FastHashMap<Component, (usize, BiasValue)> = {
        // This bit relies on the topological order of the strongly connected components
        // to compute the shortest distance from a root.
        let mut scc_to_priority: FastHashMap<Component, (usize, BiasValue)> =
            FastHashMap::default();

        for (scc, _) in scc_to_nodes.iter().enumerate() {
            let distance = scc_graph
                .neighbors_directed(Component(scc), petgraph::Direction::Incoming)
                .filter(|p| p.0 != scc)
                .map(|p| scc_to_priority[&p].0 + 1)
                .fold(0, std::cmp::max);

            let bias = scc_to_nodes[scc]
                .iter()
                .map(|Node(key, _)| diagrams[*key].1)
                .fold(usize::MAX, std::cmp::min);

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
        return Err(ContractionError::Ambiguous);
    }

    let component_to_index: FastHashMap<Component, usize> = components_sorted
        .iter()
        .enumerate()
        .map(|(index, scc)| (*scc, index))
        .collect();

    // Solve the recursive subproblems
    let mut rewrite_slices: FastHashMap<Node, Rewrite> = FastHashMap::default();

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
    let mut cospans: Vec<Cospan> = Vec::with_capacity(components_sorted.len());

    for component in &components_sorted {
        // Sort the nodes. This will assure that the forward rewrite into the first node
        // and the backward rewrite into the last node are rewrites at the boundaries
        // of the component and thus can be used to determine the target cospan.
        let mut nodes = scc_to_nodes[component.0].clone();
        nodes.sort();

        let first = nodes.first().unwrap();
        let last = nodes.last().unwrap();

        let forward = node_to_cospan[first]
            .forward
            .compose(&rewrite_slices[first])
            .unwrap();
        let backward = node_to_cospan[last]
            .backward
            .compose(&rewrite_slices[last])
            .unwrap();

        cospans.push(Cospan { forward, backward });
    }

    let target = DiagramN::new(
        <&DiagramN>::try_from(&diagrams[0].0).unwrap().source(),
        cospans,
    );

    // Assemble the rewrites
    let mut rewrites: Vec<Rewrite> = Vec::with_capacity(diagrams.len());

    for (key, diagram) in diagrams.iter().enumerate() {
        let diagram: &DiagramN = (&diagram.0).try_into().unwrap();
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

    Ok(rewrites)
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
