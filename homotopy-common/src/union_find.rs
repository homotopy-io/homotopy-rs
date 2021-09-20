use std::mem;

use crate::idx::{Idx, IdxVec};

struct NodeData<I> {
    parent: I,
    size: usize,
}

pub struct UnionFind<I> {
    forest: IdxVec<I, NodeData<I>>,
}

impl<I> NodeData<I>
where
    I: Idx,
{
    fn new(parent: I) -> Self {
        Self { parent, size: 0 }
    }
}

impl<I> UnionFind<I>
where
    I: Idx,
{
    pub fn new<T>(data: &IdxVec<I, T>) -> Self {
        Self {
            forest: data.keys().map(NodeData::new).collect(),
        }
    }

    pub fn find(&mut self, mut x: I) -> I {
        loop {
            // Get the parent of the current node
            let parent = self.forest[x].parent;

            // If the current node is it's own parent, we're done
            if parent == x {
                break x;
            }

            // Otherwise, get the grandparent of the current node
            let grandparent = self.forest[parent].parent;
            // Update the parent of the current node to its grandparent
            // (this shortens the path for future find operations)
            self.forest[x].parent = grandparent;
            // Update the current node to the parent
            x = parent;
        }
    }

    pub fn union(&mut self, mut l: I, mut r: I) {
        // Grab the representatives of the nodes in question
        l = self.find(l);
        r = self.find(r);

        // If these representatives are equal, we're done
        if l == r {
            return;
        }

        // Otherwise, perform size ordering optimisation
        if self.forest[l].size < self.forest[r].size {
            mem::swap(&mut l, &mut r);
        }

        // Then, merge the two nodes
        self.forest[r].parent = l;
        self.forest[l].size += self.forest[r].size;
    }
}
