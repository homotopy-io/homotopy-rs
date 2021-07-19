use std::ops::{Deref, DerefMut, Index};

use crate::declare_idx;
use crate::idx::IdxVec;

declare_idx! {
    pub struct Node = usize;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeData<T> {
    data: T,
    children: Vec<Node>,
    parent: Option<Node>,
}

#[derive(Debug, Clone)]
pub struct Tree<T> {
    nodes: IdxVec<Node, NodeData<T>>,
    root: Node,
}

impl<T> NodeData<T> {
    #[inline]
    pub fn inner(&self) -> &T {
        &self.data
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.data
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.data
    }

    #[inline]
    pub fn parent(&self) -> Option<Node> {
        self.parent
    }

    #[inline]
    pub fn children(&self) -> impl Iterator<Item = Node> + '_ {
        self.children.iter().copied()
    }

    #[inline]
    pub fn last(&self) -> Option<Node> {
        self.children.last().copied()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
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

impl<T> Index<usize> for NodeData<T> {
    type Output = Node;

    #[inline]
    fn index(&self, idx: usize) -> &Self::Output {
        &self.children[idx]
    }
}

impl<T> Tree<T> {
    #[inline]
    pub fn with<F, U>(&self, node: Node, f: F) -> U
    where
        F: Fn(&NodeData<T>) -> U,
    {
        f(&self.nodes[node])
    }

    #[inline]
    pub fn with_mut<F, U>(&mut self, node: Node, f: F) -> U
    where
        F: Fn(&mut NodeData<T>) -> U,
    {
        f(&mut self.nodes[node])
    }

    pub fn push_onto(&mut self, node: Node, t: T) -> Node {
        let id = self.nodes.push(NodeData {
            data: t,
            children: vec![],
            parent: Some(node),
        });
        self.nodes[node].children.push(id);
        id
    }

    #[inline]
    pub fn root(&self) -> Node {
        self.root
    }
}

impl<T> Default for Tree<T>
where
    T: Default,
{
    fn default() -> Self {
        let mut nodes = IdxVec::new();
        let root = nodes.push(Default::default());
        Self { nodes, root }
    }
}
