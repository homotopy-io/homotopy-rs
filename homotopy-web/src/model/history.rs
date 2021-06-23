use instant::Instant;
use std::fmt;
use thiserror::Error;

use super::Proof;

mod tree;

use self::tree::{Node, NodeData, Tree};

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
    snapshots: Tree<Snapshot>,
    current: Node,
}

impl Default for History {
    fn default() -> Self {
        let snapshots: Tree<Snapshot> = Default::default();
        let current = snapshots.root();
        Self { snapshots, current }
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
    #[inline]
    pub fn with_current<F, U>(&self, f: F) -> U
    where
        F: Fn(&NodeData<Snapshot>) -> U,
    {
        self.snapshots.with(self.current, f)
    }

    #[inline]
    pub fn with_proof<F, U>(&self, f: F) -> U
    where
        F: Fn(&Proof) -> U,
    {
        self.with_current(|n| f(&n.inner().proof))
    }

    pub fn add(&mut self, action: super::proof::Action, proof: Proof) {
        // check if this action has been performed at this state previously
        let existing = self.with_current(|n| {
            n.children().find(|id| {
                self.snapshots
                    .with(*id, |n| n.inner().action.as_ref() == Some(&action))
            })
        });

        if let Some(child) = existing {
            // update timestamp and ensure the action was deterministic
            self.snapshots.with_mut(child, |n| {
                assert_eq!(proof, n.inner().proof);
                n.inner_mut().touch();
            });
            self.current = child;
        } else {
            // fresh action
            let child = self
                .snapshots
                .push_onto(self.current, Snapshot::new(Some(action), proof));
            self.current = child;
        }
    }

    pub fn can_undo(&self) -> bool {
        self.with_current(|n| n.parent().is_some())
    }

    pub fn undo(&mut self) -> Result<(), HistoryError> {
        let prev = self
            .with_current(NodeData::parent)
            .ok_or(HistoryError::Undo)?;
        self.current = prev;
        Ok(())
    }

    pub fn can_redo(&self) -> bool {
        !self.with_current(|n| n.is_empty())
    }

    pub fn redo(&mut self) -> Result<(), HistoryError> {
        let next = self
            .with_current(NodeData::last)
            .ok_or(HistoryError::Redo)?;
        self.current = next;
        Ok(())
    }
}
