use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub(super) type NodeRef<T> = Rc<RefCell<Node<T>>>;
type WeakNodeRef<T> = Weak<RefCell<Node<T>>>;

#[derive(Debug, Clone, Default)]
pub(super) struct Node<T> {
    pub(super) data: T,
    pub(super) children: Vec<NodeRef<T>>,
    pub(super) parent: WeakNodeRef<T>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct Tree<T> {
    pub(super) root: NodeRef<T>,
}

impl<T> Tree<T> {
    pub fn push(&mut self, data: T) {
        let node = Rc::new(RefCell::new(Node {
            data,
            children: Default::default(),
            parent: Rc::downgrade(&self.root),
        }));
        self.root.borrow_mut().children.push(node);
    }
}

impl<T> From<NodeRef<T>> for Tree<T> {
    fn from(root: NodeRef<T>) -> Self {
        Self { root }
    }
}
