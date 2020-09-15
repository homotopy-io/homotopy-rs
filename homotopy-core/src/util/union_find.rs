use std::cell::Cell;

pub struct UnionFind(Vec<Node>);

impl UnionFind {
    pub fn new(size: usize) -> Self {
        UnionFind((0..size).map(|_| Node::new()).collect())
    }

    pub fn make(&mut self) -> NodeIndex {
        let index = self.0.len();
        self.0.push(Node::new());
        index
    }

    fn parent(&self, index: NodeIndex) -> NodeIndex {
        self.0[index].parent().unwrap_or(index)
    }

    pub fn find(&self, mut index: NodeIndex) -> NodeIndex {
        loop {
            let node = &self.0[index];

            match node {
                Node::Root(_) => {
                    return index;
                }
                Node::Link(parent_cell) => {
                    let parent = parent_cell.get();
                    let grandparent = self.parent(parent);
                    parent_cell.set(grandparent);
                    index = grandparent;
                }
            }
        }
    }

    pub fn union(&mut self, a: NodeIndex, b: NodeIndex) {
        let a = self.find(a);
        let b = self.find(b);

        if a == b {
            return;
        }

        let a_rank = self.0[a].rank().unwrap();
        let b_rank = self.0[b].rank().unwrap();

        if a_rank > b_rank {
            self.0[b] = Node::Link(Cell::new(a));
        } else if a_rank == b_rank {
            self.0[b] = Node::Link(Cell::new(a));
            self.0[a] = Node::Root(a_rank + 1);
        } else {
            self.0[a] = Node::Link(Cell::new(b));
        }
    }
}

pub type NodeIndex = usize;

type Rank = usize;

enum Node {
    Root(Rank),
    Link(Cell<NodeIndex>),
}

impl Node {
    fn new() -> Self {
        Node::Root(0)
    }

    fn rank(&self) -> Option<usize> {
        match self {
            Node::Root(rank) => Some(*rank),
            Node::Link(_) => None,
        }
    }

    fn parent(&self) -> Option<NodeIndex> {
        match self {
            Node::Root(_) => None,
            Node::Link(parent) => Some(parent.get()),
        }
    }
}
