use std::{
    collections::VecDeque,
    iter::{FromIterator, FusedIterator},
    mem,
    ops::{Deref, DerefMut, Index, IndexMut},
};

use crate::{declare_idx, hash::FastHashMap, idx::IdxVec};

declare_idx! {
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Node = usize;
}

/// [PartialEq] should only be applied to nodes of the same tree
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct NodeData<T> {
    data: T,
    parent: Option<Node>,
    children: Vec<Node>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tree<T> {
    nodes: IdxVec<Node, NodeData<T>>,
    root: Node,
}

impl<T> NodeData<T> {
    #[inline]
    pub const fn inner(&self) -> &T {
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
    pub fn map<F, U>(self, f: F) -> NodeData<U>
    where
        F: FnOnce(T) -> U,
    {
        NodeData {
            data: f(self.data),
            children: self.children,
            parent: self.parent,
        }
    }

    #[inline]
    pub const fn parent(&self) -> Option<Node> {
        self.parent
    }

    #[inline]
    pub fn children(&self) -> impl Iterator<Item = Node> + '_ {
        self.children.iter().copied()
    }

    #[inline]
    pub fn children_rev(&self) -> impl Iterator<Item = Node> + '_ {
        self.children.iter().rev().copied()
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

impl<T> Tree<T> {
    #[inline]
    pub fn new(root: T) -> Self {
        let mut nodes = IdxVec::new();
        let root = nodes.push(NodeData {
            data: root,
            parent: None,
            children: vec![],
        });

        Self { nodes, root }
    }

    #[inline]
    #[must_use]
    pub fn get(&self, node: Node) -> Option<&NodeData<T>> {
        self.nodes.get(node)
    }

    #[inline]
    pub fn get_mut(&mut self, node: Node) -> Option<&mut NodeData<T>> {
        self.nodes.get_mut(node)
    }

    #[inline]
    pub fn with<F, U>(&self, node: Node, f: F) -> Option<U>
    where
        F: FnOnce(&NodeData<T>) -> U,
    {
        (node.0 < self.nodes.len()).then(|| f(&self.nodes[node]))
    }

    #[inline]
    pub fn with_mut<F, U>(&mut self, node: Node, f: F) -> Option<U>
    where
        F: FnOnce(&mut NodeData<T>) -> U,
    {
        (node.0 < self.nodes.len()).then(|| f(&mut self.nodes[node]))
    }

    /// Removes `node` from the tree. This is done by disconnecting the subtree rooted at `node`
    /// from the rest of the tree, leaving the underlying data untouched. Thus, attempting to
    /// remove `self.root()` is a no-op.
    ///
    /// A clean-up can be performed by executing a normalisation operation after a removal.
    /// Doing so will free the memory associated with all disconnected components.
    #[inline]
    pub fn remove(&mut self, node: Node) {
        if node.0 < self.nodes.len() {
            if let Some(parent) = self.nodes[node].parent {
                self.nodes[parent].children.retain(|child| *child != node);
            }
            self.nodes[node].parent = None;
        }
    }

    #[inline]
    pub fn push_onto(&mut self, node: Node, t: T) -> Option<Node> {
        (node.0 < self.nodes.len()).then(|| {
            let id = self.nodes.push(NodeData {
                data: t,
                children: vec![],
                parent: Some(node),
            });
            self.nodes[node].children.push(id);
            id
        })
    }

    /// Returns an iterator of the ancestors of `node`.
    #[inline]
    pub fn ancestors_of(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        AncestorIterator {
            tree: self,
            current: Some(node),
        }
    }

    /// Returns an iterator of the ancestors of `node`.
    #[inline]
    pub fn descendents_of(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        DescendantsIterator {
            tree: self,
            to_visit: {
                let mut queue = VecDeque::new();
                queue.push_back(node);
                queue
            },
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (Node, &NodeData<T>)> + '_ {
        self.descendents_of(self.root)
            .map(move |n| (n, &self.nodes[n]))
    }

    #[inline]
    pub fn map<F, U>(self, mut f: F) -> Tree<U>
    where
        F: FnMut(T) -> U,
    {
        #[allow(clippy::redundant_closure)]
        Tree {
            root: self.root,
            nodes: self.nodes.map(|nd| nd.map(|x| f(x))),
        }
    }

    pub fn filter_map<F, U>(self, mut f: F) -> Tree<U>
    where
        F: FnMut(&T) -> Option<U>,
        U: Default,
    {
        let mut result: Tree<U> = Default::default();
        let mut node_mappings: FastHashMap<Node, Node> = Default::default();
        node_mappings.insert(self.root, result.root);
        for (node, data) in self.iter().skip(1) {
            let parent = if let Some(parent) = data.parent() {
                node_mappings[&parent]
            } else {
                result.root
            };
            if let Some(d) = f(data.inner()) {
                node_mappings.insert(node, result.push_onto(parent, d).unwrap());
            }
        }
        result
    }

    #[inline]
    #[must_use]
    pub const fn root(&self) -> Node {
        self.root
    }

    fn reparent_at<F>(&mut self, node: Node, parent: Node, index: F)
    where
        F: FnOnce(&[Node]) -> Option<usize>,
    {
        // Can't reparent the root
        assert_ne!(node, self.root);
        // Don't introduce a cycle
        assert!(self.ancestors_of(parent).all(|ancestor| ancestor != node));

        if node.0 >= self.nodes.len() || parent.0 >= self.nodes.len() {
            return;
        }

        if let Some(old_parent) = self.nodes[node].parent {
            self.nodes[old_parent]
                .children
                .retain(|child| *child != node);
        }

        self.nodes[node].parent = Some(parent);
        if let Some(index) = index(&self.nodes[parent].children) {
            self.nodes[parent].children.insert(index, node);
        } else {
            self.nodes[parent].children.push(node);
        }
    }

    #[inline]
    pub fn reparent_under(&mut self, node: Node, parent: Node) {
        self.reparent_at(node, parent, |_| None);
    }

    pub fn reparent_before(&mut self, node: Node, successor: Node) {
        // Can't be a sibling of the root
        assert_ne!(node, self.root);

        // Fast return when we're trying to reparent a node next to itself
        if node == successor {
            return;
        }

        if successor.0 >= self.nodes.len() {
            return;
        }

        if let Some(parent) = self.nodes[successor].parent {
            self.reparent_at(node, parent, |siblings| {
                siblings.iter().position(|child| *child == successor)
            });
        }
    }

    pub fn clean_up(&mut self) {
        // Allocate temporary storage
        let mut nodes = IdxVec::new();

        // Exchange working memory with current storage
        mem::swap(&mut self.nodes, &mut nodes);

        // Update every node from containing a `T` to an `Option<T>` so that we can
        // `take` the data out later
        let mut nodes: IdxVec<_, _> = nodes.into_values().map(|n| n.map(Some)).collect();

        // Initialise a queue for a breadth-first walk (starting at the root)
        let mut to_visit = VecDeque::new();
        to_visit.push_back(self.root);

        // For every node in the walk
        while let Some(node) = to_visit.pop_front() {
            // Get that node's data in the working memory
            let node_data = &mut nodes[node];
            // Add a corresponding node to the cleaned tree
            let idx = self.nodes.push(NodeData {
                // Safe to unwrap here because each node will be visited once
                data: node_data.data.take().unwrap(),
                // We'll add the children as we perform the rest of the walk, but we
                // know ahead of time how many we'll need
                children: Vec::with_capacity(node_data.children.len()),
                // Copy parent directly (this index will have been updated to be
                // valid in the cleaned tree in an earlier iteration)
                parent: node_data.parent,
            });

            // If the node we just added has a parent, make sure we add it to that
            // node's list of children.
            if let Some(parent) = self.nodes[idx].parent {
                self.nodes[parent].children.push(idx);
            }

            // Drain all of the children from the node in working memory
            let children: Vec<_> = nodes[node].children.drain(0..).collect();

            // For each of these children, add them to the back of the queue
            // and set their parent field to point to the newly created node
            // in the cleaned tree
            for child in children {
                to_visit.push_back(child);
                nodes[child].parent = Some(idx);
            }
        }
    }
}

impl<T> Tree<Option<T>> {
    #[must_use]
    pub fn transpose(self) -> Option<Tree<T>> {
        let mut nodes = IdxVec::with_capacity(self.nodes.len());

        for node in self.nodes.into_values() {
            nodes.push(NodeData {
                parent: node.parent,
                children: node.children,
                data: node.data?,
            });
        }

        Some(Tree {
            nodes,
            root: self.root,
        })
    }
}

impl<T> PartialEq for Tree<T>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        // Two trees are identical whenever we can do a breadth-first walk
        // starting at the root of each in which each pair of nodes we visit have
        // an identical number of children and their associated data are identical
        self.iter().zip(other.iter()).all(|((_, lhs), (_, rhs))| {
            lhs.children.len() == rhs.children.len() && lhs.inner() == rhs.inner()
        })
    }
}

impl<T> Eq for Tree<T> where T: Eq {}

impl<T> Default for Tree<T>
where
    T: Default,
{
    #[inline]
    fn default() -> Self {
        let mut nodes = IdxVec::new();
        let root = nodes.push(Default::default());
        Self { nodes, root }
    }
}

impl<T> Index<Node> for Tree<T> {
    type Output = NodeData<T>;

    fn index(&self, index: Node) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<T> IndexMut<Node> for Tree<T> {
    fn index_mut(&mut self, index: Node) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

impl<T> FromIterator<T> for Tree<T>
where
    T: Default,
{
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut tree = Self::default();

        for item in iter {
            tree.push_onto(tree.root(), item);
        }

        tree
    }
}

pub struct AncestorIterator<'a, T> {
    tree: &'a Tree<T>,
    current: Option<Node>,
}

impl<'a, T> Iterator for AncestorIterator<'a, T> {
    type Item = Node;

    #[inline]
    fn next(&mut self) -> Option<Node> {
        let node = self.current?;
        self.current = self.tree.with(node, NodeData::parent)?;
        Some(node)
    }
}

impl<'a, T> FusedIterator for AncestorIterator<'a, T> {}

pub struct DescendantsIterator<'a, T> {
    tree: &'a Tree<T>,
    to_visit: VecDeque<Node>,
}

impl<'a, T> Iterator for DescendantsIterator<'a, T> {
    type Item = Node;

    #[inline]
    fn next(&mut self) -> Option<Node> {
        let node = self.to_visit.pop_front()?;
        self.tree.with(node, |n| {
            for child in n.children() {
                self.to_visit.push_back(child);
            }
        });
        Some(node)
    }
}

impl<'a, T> FusedIterator for DescendantsIterator<'a, T> {}
