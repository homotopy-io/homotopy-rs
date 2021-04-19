use std::{cell::Ref, convert::Into};
use thiserror::Error;
pub mod proof;
use proof::{AttachOption, Color, GeneratorInfo, Proof, Signature, Workspace};

use self::history::History;
pub mod history;
pub mod serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    ToggleDrawer(Drawer),

    Proof(proof::Action),

    History(history::Action),

    Serialize(serialize::Serialize),
}

#[derive(Debug, Clone, Default)]
pub struct State {
    history: History,
    drawer: Option<Drawer>,
}

impl State {
    /// Get the proof data
    pub(super) fn proof(&self) -> Ref<Proof> {
        self.history.current()
    }

    /// Update the state in response to an [Action].
    pub fn update(&mut self, action: Action) -> Result<(), ModelError> {
        match action {
            Action::ToggleDrawer(drawer) => {
                self.toggle_drawer(drawer);
                Ok(())
            }
            Action::Proof(action) => {
                let mut proof = self.proof().clone();
                proof.update(&action).map_err(ModelError::from)?;

                if action == proof::Action::CreateGeneratorZero && self.drawer.is_none() {
                    self.drawer = Some(Drawer::Signature);
                };

                self.history.add(action, proof);
                Ok(())
            }
            Action::History(history::Action::Move(dir)) => {
                use homotopy_core::Direction::{Backward, Forward};
                match dir {
                    history::Direction::Linear(Forward) => {
                        self.history.redo().map_err(ModelError::from)
                    }
                    history::Direction::Linear(Backward) => {
                        self.history.undo().map_err(ModelError::from)
                    }
                }
            }
            Action::Serialize(serialize::Serialize::Export) => self.export(),
            Action::Serialize(serialize::Serialize::Import(data)) => {
                let (signature, workspace) = *data;
                self.import(signature, workspace);
                Ok(())
            }
        }
    }

    /// Handler for [Action::ToggleDrawer].
    fn toggle_drawer(&mut self, drawer: Drawer) {
        if self.drawer == Some(drawer) {
            self.drawer = None;
        } else {
            self.drawer = Some(drawer);
        }
    }

    fn export(&self) -> Result<(), ModelError> {
        let data: serialize::Data = self.proof().workspace.clone().map_or_else(
            || self.proof().signature.clone().into(),
            |ws| (self.proof().signature.clone(), ws).into(),
        );
        serialize::generate_download(
            &"filename_todo.hom",
            &Into::<Vec<u8>>::into(data).as_slice(),
        )
        .map_err(ModelError::Export)
    }

    fn import(&mut self, signature: Signature, workspace: Option<Workspace>) {
        let mut proof: Proof = Default::default();
        proof.signature = signature;
        proof.workspace = workspace;
        self.history.add(proof::Action::Imported, proof);
    }

    pub fn drawer(&self) -> Option<Drawer> {
        self.drawer
    }
}

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("export failed")]
    Export(wasm_bindgen::JsValue),
    #[error(transparent)]
    Proof(#[from] proof::ModelError),
    #[error(transparent)]
    History(#[from] history::HistoryError),
    #[error("internal error")]
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Drawer {
    Project,
    Signature,
    User,
}
