mod tree;
use super::Proof;
use instant::Instant;
use std::{cell::Ref, fmt};
use thiserror::Error;
use tree::{NodeRef, Tree};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Move(Direction),
    // TODO: history pruning
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    Linear(homotopy_core::common::Direction),
    // TODO: branch moves
}

#[derive(Clone, PartialEq, Eq)]
pub struct Snapshot {
    proof: Proof,
    timestamp: instant::Instant,
    action: Option<super::proof::Action>,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            proof: Default::default(),
            timestamp: Instant::now(),
            action: Default::default(),
        }
    }
}

impl fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(T{:?}, {:?})", self.timestamp, self.action)
    }
}

impl Snapshot {
    fn new(action: Option<super::proof::Action>, proof: Proof) -> Self {
        Self {
            proof,
            action,
            timestamp: Instant::now(),
        }
    }

    fn touch(&mut self) {
        self.timestamp = Instant::now();
    }
}

#[derive(Debug, Clone)]
pub struct History {
    start: Tree<Snapshot>,
    current: NodeRef<Snapshot>,
}

impl Default for History {
    fn default() -> Self {
        let start: Tree<Snapshot> = Default::default();
        let current = start.root.clone();
        Self { start, current }
    }
}

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("error while performing undo")]
    Undo,
    #[error("error while performing redo")]
    Redo,
}

impl History {
    pub fn current(&self) -> Ref<Proof> {
        Ref::map(self.current.borrow(), |r| &r.data.proof)
    }

    pub fn add(&mut self, action: super::proof::Action, proof: Proof) {
        // check if this action has been performed at this state previously
        let existing = self
            .current
            .borrow()
            .children
            .iter()
            .cloned()
            .find(|c| c.borrow().data.action.as_ref() == Some(&action));
        if let Some(child) = existing {
            {
                // update timestamp and ensure the action was deterministic
                let s = &mut child.borrow_mut().data;
                assert_eq!(proof, s.proof);
                s.touch();
            }
            self.current = child;
        } else {
            // fresh action
            Tree::from(self.current.clone()).push(Snapshot::new(Some(action), proof));
            let child = self.current.borrow().children.last().unwrap().clone();
            self.current = child;
        }
    }

    pub fn can_undo(&self) -> bool {
        self.current.borrow().parent.strong_count() > 0
    }

    pub fn undo(&mut self) -> Result<(), HistoryError> {
        let prev = self
            .current
            .borrow()
            .parent
            .upgrade()
            .ok_or(HistoryError::Undo)?;
        self.current = prev;
        Ok(())
    }

    pub fn can_redo(&self) -> bool {
        !self.current.borrow().children.is_empty()
    }

    pub fn redo(&mut self) -> Result<(), HistoryError> {
        let next = self
            .current
            .borrow()
            .children
            .last()
            .cloned()
            .ok_or(HistoryError::Redo)?;
        self.current = next;
        Ok(())
    }
}
