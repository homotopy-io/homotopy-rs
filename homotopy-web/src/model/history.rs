use std::fmt;
use std::ops::{Deref, DerefMut};

use instant::Instant;
use thiserror::Error;

use super::proof::ProofState;

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
    proof: ProofState,
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

pub type Proof = NodeData<Snapshot>;

impl Proof {
    pub fn can_undo(&self) -> bool {
        self.parent().is_some()
    }

    pub fn can_redo(&self) -> bool {
        !self.is_empty()
    }
}

impl Deref for Proof {
    type Target = ProofState;

    fn deref(&self) -> &Self::Target {
        &self.inner().proof
    }
}

impl DerefMut for Proof {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner_mut().proof
    }
}

impl fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(T{:?}, {:?})", self.timestamp, self.action)
    }
}

impl Snapshot {
    fn new(action: Option<super::proof::Action>, proof: ProofState) -> Self {
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
    overlay: Option<ProofState>,
    current: Node,
}

impl Default for History {
    fn default() -> Self {
        let snapshots: Tree<Snapshot> = Default::default();
        let current = snapshots.root();
        Self {
            snapshots,
            overlay: None,
            current,
        }
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
    #[allow(clippy::option_if_let_else)]
    pub fn with_proof<F, U>(&self, f: F) -> U
    where
        F: Fn(&Proof) -> U,
    {
        if let Some(ref overlay) = self.overlay {
            let mut overlayed = self.snapshots.with(self.current, Clone::clone);
            overlayed.inner_mut().proof = overlay.clone();
            f(&overlayed)
        } else {
            self.with_proof_internal(f)
        }
    }

    #[inline]
    pub fn with_proof_internal<F, U>(&self, f: F) -> U
    where
        F: Fn(&Proof) -> U,
    {
        self.snapshots.with(self.current, f)
    }

    pub fn add(&mut self, action: super::proof::Action, proof: Proof) {
        // check if this action has been performed at this state previously
        let existing = self.with_proof_internal(|n| {
            n.children().find(|id| {
                self.snapshots
                    .with(*id, |n| n.inner().action.as_ref() == Some(&action))
            })
        });

        if let Some(child) = existing {
            // update timestamp and ensure the action was deterministic
            self.snapshots.with_mut(child, |n| {
                assert_eq!(proof.inner().proof, n.inner().proof);
                n.inner_mut().touch();
            });
            self.current = child;
        } else if action.relevant() {
            // fresh action
            let child = self.snapshots.push_onto(
                self.current,
                Snapshot::new(Some(action), proof.into_inner().proof),
            );
            self.current = child;
            self.overlay = None;
        } else {
            self.overlay = Some(proof.into_inner().proof);
        }
    }

    pub fn undo(&mut self) -> Result<(), HistoryError> {
        let prev = self
            .with_proof_internal(NodeData::parent)
            .ok_or(HistoryError::Undo)?;
        self.overlay = None;
        self.current = prev;
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), HistoryError> {
        let next = self
            .with_proof_internal(NodeData::last)
            .ok_or(HistoryError::Redo)?;
        self.overlay = None;
        self.current = next;
        Ok(())
    }
}
