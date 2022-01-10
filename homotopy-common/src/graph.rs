use std::ops::{Deref, DerefMut, Index, IndexMut};

use crate::{declare_idx, idx::IdxVec};

declare_idx! {
    pub struct Node = usize;
    pub struct Edge = usize;
}

#[derive(Clone, Debug)]
pub struct NodeData<T> {
    weight: T,
    incoming: Vec<Edge>,
    outgoing: Vec<Edge>,
}

impl<T> NodeData<T> {
    #[inline]
    pub fn inner(&self) -> &T {
        &self.weight
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.weight
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.weight
    }

    #[inline]
    pub fn map<F, U>(self, f: F) -> NodeData<U>
    where
        F: FnOnce(T) -> U,
    {
        NodeData {
            weight: f(self.weight),
            incoming: self.incoming,
            outgoing: self.outgoing,
        }
    }

    #[inline]
    pub fn incoming_edges(&self) -> impl Iterator<Item = Edge> + '_ {
        self.incoming.iter().copied()
    }

    #[inline]
    pub fn outgoing_edges(&self) -> impl Iterator<Item = Edge> + '_ {
        self.outgoing.iter().copied()
    }
}

impl<T> Deref for NodeData<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl<T> DerefMut for NodeData<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}

#[derive(Clone, Debug)]
pub struct EdgeData<T> {
    weight: T,
    endpoints: [Node; 2],
}

impl<T> EdgeData<T> {
    #[inline]
    pub fn inner(&self) -> &T {
        &self.weight
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.weight
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.weight
    }

    #[inline]
    pub fn map<F, U>(self, f: F) -> EdgeData<U>
    where
        F: FnOnce(T) -> U,
    {
        EdgeData {
            weight: f(self.weight),
            endpoints: self.endpoints,
        }
    }

    #[inline]
    pub fn source(&self) -> Node {
        self.endpoints[0]
    }

    #[inline]
    pub fn target(&self) -> Node {
        self.endpoints[1]
    }
}

impl<T> Deref for EdgeData<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl<T> DerefMut for EdgeData<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}

#[derive(Clone, Debug)]
pub struct Graph<V, E> {
    nodes: IdxVec<Node, NodeData<V>>,
    edges: IdxVec<Edge, EdgeData<E>>,
}

impl<V, E> Graph<V, E> {
    #[inline]
    pub fn new() -> Self {
        Self {
            nodes: IdxVec::new(),
            edges: IdxVec::new(),
        }
    }

    #[inline]
    pub fn add_node(&mut self, v: V) -> Node {
        self.nodes.push(NodeData {
            weight: v,
            incoming: vec![],
            outgoing: vec![],
        })
    }

    #[inline]
    pub fn add_edge(&mut self, s: Node, t: Node, e: E) -> Edge {
        let id = self.edges.push(EdgeData {
            weight: e,
            endpoints: [s, t],
        });
        self.nodes[s].outgoing.push(id);
        self.nodes[t].incoming.push(id);
        id
    }

    #[inline]
    pub fn find_edge(&self, s: Node, t: Node) -> Option<Edge> {
        for e in self.outgoing_edges(s) {
            if self.target(e) == t {
                return Some(e);
            }
        }
        None
    }

    #[inline]
    pub fn update_edge(&mut self, s: Node, t: Node, e: E) -> Edge {
        if let Some(id) = self.find_edge(s, t) {
            if let Some(weight) = self.edge_weight_mut(id) {
                *weight = e;
                return id;
            }
        }
        self.add_edge(s, t, e)
    }

    #[inline]
    pub fn with_node<F, U>(&self, node: Node, f: F) -> U
    where
        F: FnOnce(&NodeData<V>) -> U,
    {
        f(&self.nodes[node])
    }

    #[inline]
    pub fn with_edge<F, U>(&self, edge: Edge, f: F) -> U
    where
        F: FnOnce(&EdgeData<E>) -> U,
    {
        f(&self.edges[edge])
    }

    #[inline]
    pub fn with_node_mut<F, U>(&mut self, node: Node, f: F) -> U
    where
        F: FnOnce(&mut NodeData<V>) -> U,
    {
        f(&mut self.nodes[node])
    }

    #[inline]
    pub fn with_edge_mut<F, U>(&mut self, edge: Edge, f: F) -> U
    where
        F: FnOnce(&mut EdgeData<E>) -> U,
    {
        f(&mut self.edges[edge])
    }

    #[inline]
    pub fn map<F, G, V2, E2>(self, mut f: F, mut g: G) -> Graph<V2, E2>
    where
        F: FnMut(V) -> V2,
        G: FnMut(E) -> E2,
    {
        Graph {
            nodes: self.nodes.map(|nd| nd.map(|x| f(x))),
            edges: self.edges.map(|ed| ed.map(|x| g(x))),
        }
    }

    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[inline]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    #[inline]
    pub fn nodes(&self) -> impl Iterator<Item = (Node, &NodeData<V>)> {
        self.nodes.iter()
    }

    #[inline]
    pub fn nodes_mut(&mut self) -> impl Iterator<Item = (Node, &mut NodeData<V>)> {
        self.nodes.iter_mut()
    }

    #[inline]
    pub fn node_keys(&self) -> impl Iterator<Item = Node> {
        self.nodes.keys()
    }

    #[inline]
    pub fn node_values(&self) -> impl Iterator<Item = &NodeData<V>> {
        self.nodes.values()
    }

    #[inline]
    pub fn node_values_mut(&mut self) -> impl Iterator<Item = &mut NodeData<V>> {
        self.nodes.values_mut()
    }

    #[inline]
    pub fn edges(&self) -> impl Iterator<Item = (Edge, &EdgeData<E>)> {
        self.edges.iter()
    }

    #[inline]
    pub fn edges_mut(&mut self) -> impl Iterator<Item = (Edge, &mut EdgeData<E>)> {
        self.edges.iter_mut()
    }

    #[inline]
    pub fn edge_keys(&self) -> impl Iterator<Item = Edge> {
        self.edges.keys()
    }

    #[inline]
    pub fn edge_values(&self) -> impl Iterator<Item = &EdgeData<E>> {
        self.edges.values()
    }

    #[inline]
    pub fn edge_values_mut(&mut self) -> impl Iterator<Item = &mut EdgeData<E>> {
        self.edges.values_mut()
    }

    #[inline]
    pub fn node_weight(&self, node: Node) -> Option<&V> {
        self.nodes.get(node).map(NodeData::inner)
    }

    #[inline]
    pub fn edge_weight(&self, edge: Edge) -> Option<&E> {
        self.edges.get(edge).map(EdgeData::inner)
    }

    #[inline]
    pub fn node_weight_mut(&mut self, node: Node) -> Option<&mut V> {
        self.nodes.get_mut(node).map(NodeData::inner_mut)
    }

    #[inline]
    pub fn edge_weight_mut(&mut self, edge: Edge) -> Option<&mut E> {
        self.edges.get_mut(edge).map(EdgeData::inner_mut)
    }

    #[inline]
    pub fn source(&self, edge: Edge) -> Node {
        self.edges[edge].source()
    }

    #[inline]
    pub fn target(&self, edge: Edge) -> Node {
        self.edges[edge].target()
    }

    #[inline]
    pub fn incoming_edges(&self, node: Node) -> impl Iterator<Item = Edge> + '_ {
        self.nodes[node].incoming_edges()
    }

    #[inline]
    pub fn outgoing_edges(&self, node: Node) -> impl Iterator<Item = Edge> + '_ {
        self.nodes[node].outgoing_edges()
    }

    pub fn sources(&self) -> impl Iterator<Item = Node> + '_ {
        self.nodes
            .iter()
            .filter(|(_, nd)| nd.incoming_edges().next().is_none())
            .map(|(n, _)| n)
    }

    pub fn targets(&self) -> impl Iterator<Item = Node> + '_ {
        self.nodes
            .iter()
            .filter(|(_, nd)| nd.outgoing_edges().next().is_none())
            .map(|(n, _)| n)
    }
}

impl<V, E> Default for Graph<V, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V, E> Index<Node> for Graph<V, E> {
    type Output = V;

    #[inline]
    fn index(&self, index: Node) -> &Self::Output {
        self.nodes[index].inner()
    }
}

impl<V, E> Index<Edge> for Graph<V, E> {
    type Output = E;

    #[inline]
    fn index(&self, index: Edge) -> &Self::Output {
        self.edges[index].inner()
    }
}

impl<V, E> IndexMut<Node> for Graph<V, E> {
    #[inline]
    fn index_mut(&mut self, index: Node) -> &mut Self::Output {
        self.nodes[index].inner_mut()
    }
}

impl<V, E> IndexMut<Edge> for Graph<V, E> {
    #[inline]
    fn index_mut(&mut self, index: Edge) -> &mut Self::Output {
        self.edges[index].inner_mut()
    }
}
