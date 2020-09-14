use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct Graph<K, N, E> {
    nodes: HashMap<K, Node<K, N>>,
    edges: HashMap<(K, K), E>,
}

impl<K, N, E> Graph<K, N, E>
where
    K: Hash + Eq + Copy,
{
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, key: K, label: N) -> Option<N> {
        match self.nodes.get_mut(&key) {
            None => {
                self.nodes.insert(key, Node::new(label));
                None
            }
            Some(node) => Some(std::mem::replace(&mut node.label, label)),
        }
    }

    pub fn add_edge(&mut self, source: K, target: K, label: E) -> Option<E> {
        self.nodes
            .get_mut(&source)
            .unwrap()
            .edges_out
            .push(target.clone());
        self.nodes
            .get_mut(&target)
            .unwrap()
            .edges_in
            .push(source.clone());
        self.edges.insert((source, target), label)
    }

    pub fn pred(&self, key: &K) -> Vec<K> {
        self.nodes
            .get(&key)
            .map(|node| node.edges_in.clone())
            .unwrap_or_default()
    }

    pub fn succ(&self, key: &K) -> Vec<K> {
        self.nodes
            .get(&key)
            .map(|node| node.edges_out.clone())
            .unwrap_or_default()
    }

    pub fn get_node(&self, key: &K) -> Option<&N> {
        self.nodes.get(&key).map(|node| &node.label)
    }

    pub fn get_node_mut(&mut self, key: &K) -> Option<&mut N> {
        self.nodes.get_mut(&key).map(|node| &mut node.label)
    }

    pub fn get_node_or_insert(&mut self, key: &K, value: N) -> &mut N {
        &mut self.nodes.entry(*key).or_insert(Node::new(value)).label
    }

    pub fn get_edge(&self, source: &K, target: &K) -> Option<&E> {
        self.edges.get(&(source.clone(), target.clone()))
    }

    pub fn get_edge_mut(&mut self, source: &K, target: &K) -> Option<&mut E> {
        self.edges.get_mut(&(source.clone(), target.clone()))
    }

    pub fn get_edge_or_insert(&mut self, source: &K, target: &K, edge: E) -> &mut E {
        self.edges.entry((*source, *target)).or_insert(edge)
    }

    pub fn nodes(&self) -> Vec<K> {
        self.nodes.keys().map(|k| *k).collect()
    }

    pub fn edges(&self) -> Vec<(K, K)> {
        self.edges.keys().map(|k| *k).collect()
    }

    /// Calculates the strongly connected components of the graph using Tarjan's algorithm. The
    /// components are ordered in reverse topological order.
    pub fn sccs(&self) -> Vec<Vec<K>> {
        #[derive(Clone, Copy)]
        struct NodeState {
            index: usize,
            lowlink: usize,
            on_stack: bool,
        }

        struct State<K> {
            nodes: HashMap<K, NodeState>,
            index: usize,
            stack: Vec<K>,
            components: Vec<Vec<K>>,
        }

        fn visit<K: Hash + Eq + Copy, N, E>(state: &mut State<K>, graph: &Graph<K, N, E>, v: K) {
            let mut v_state = NodeState {
                index: state.index,
                lowlink: state.index,
                on_stack: true,
            };

            state.nodes.insert(v, v_state);

            state.stack.push(v);
            state.index += 1;

            for w in graph.succ(&v) {
                match state.nodes.get(&w) {
                    None => {
                        visit(state, graph, w);
                        let w_state = state.nodes[&w];
                        v_state.lowlink = std::cmp::min(v_state.lowlink, w_state.lowlink);
                        state.nodes.insert(v, v_state);
                    }
                    Some(w_state) if w_state.on_stack => {
                        v_state.lowlink = std::cmp::min(v_state.lowlink, w_state.index);
                        state.nodes.insert(v, v_state);
                    }
                    Some(_) => {}
                }
            }

            if v_state.index == v_state.lowlink {
                let mut component = vec![];

                loop {
                    let w = state.stack.pop().unwrap();
                    state.nodes.get_mut(&w).unwrap().on_stack = false;
                    component.push(w);

                    if w == v {
                        break;
                    }
                }

                state.components.push(component);
            }
        }

        let mut state = State {
            nodes: HashMap::new(),
            index: 0,
            components: Vec::new(),
            stack: Vec::new(),
        };

        for v in self.nodes() {
            if !state.nodes.contains_key(&v) {
                visit(&mut state, self, v);
            }
        }

        state.components
    }

    pub fn contract<F, L>(&self, component: F) -> Graph<L, Vec<K>, Vec<(K, K)>>
    where
        F: Fn(&K) -> L,
        L: Hash + Eq + Copy,
    {
        let mut contracted = Graph::new();

        for node in self.nodes() {
            contracted
                .get_node_or_insert(&component(&node), vec![])
                .push(node);
        }

        for (source, target) in self.edges() {
            let source_component = component(&source);
            let target_component = component(&target);
            contracted
                .get_edge_or_insert(&source_component, &target_component, vec![])
                .push((source, target));
        }

        contracted
    }
}

#[derive(Debug, Clone)]
struct Node<K, N> {
    edges_in: Vec<K>,
    edges_out: Vec<K>,
    label: N,
}

impl<K, N> Node<K, N> {
    fn new(label: N) -> Self {
        Node {
            label,
            edges_in: vec![],
            edges_out: vec![],
        }
    }
}
