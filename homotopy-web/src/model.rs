use std::cell::Ref;
use thiserror::Error;

pub mod history;
pub mod proof;
pub mod serialize;

use history::History;
use proof::{Color, GeneratorInfo, Proof, Signature, Workspace};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    ToggleDrawer(Drawer),
    Proof(proof::Action),
    History(history::Action),
    ImportProof(SerializedData),
    ExportProof,
}

impl From<proof::Action> for Action {
    fn from(action: proof::Action) -> Self {
        Self::Proof(action)
    }
}

impl From<history::Action> for Action {
    fn from(action: history::Action) -> Self {
        Self::History(action)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SerializedData(Vec<u8>);

impl std::fmt::Debug for SerializedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SerializedData").finish()
    }
}

impl From<Vec<u8>> for SerializedData {
    fn from(data: Vec<u8>) -> Self {
        Self(data)
    }
}

impl From<SerializedData> for Vec<u8> {
    fn from(data: SerializedData) -> Self {
        data.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub history: History,
    pub drawer: Option<Drawer>,
}

impl State {
    /// Get the proof data
    pub(super) fn proof(&self) -> Ref<Proof> {
        self.history.current()
    }

    pub(super) fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub(super) fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn drawer(&self) -> Option<Drawer> {
        self.drawer
    }

    /// Update the state in response to an [Action].
    pub fn update(&mut self, action: Action) -> Result<(), ModelError> {
        match action {
            Action::ToggleDrawer(drawer) => {
                if self.drawer == Some(drawer) {
                    self.drawer = None;
                } else {
                    self.drawer = Some(drawer);
                }
            }

            Action::Proof(action) => {
                let mut proof = self.proof().clone();
                proof.update(&action).map_err(ModelError::from)?;

                if action == proof::Action::CreateGeneratorZero && self.drawer.is_none() {
                    self.drawer = Some(Drawer::Signature);
                };

                self.history.add(action, proof);
            }

            Action::History(history::Action::Move(dir)) => {
                use homotopy_core::Direction::{Backward, Forward};
                match dir {
                    history::Direction::Linear(Forward) => {
                        self.history.redo()?;
                    }
                    history::Direction::Linear(Backward) => {
                        self.history.undo()?;
                    }
                };
            }

            Action::ExportProof => {
                let data = serialize::serialize(
                    self.proof().signature.clone(),
                    self.proof().workspace.clone(),
                );
                serialize::generate_download(&"filename_todo", data.as_slice())
                    .map_err(ModelError::Export)?;
            }

            Action::ImportProof(data) => {
                let (signature, workspace) =
                    serialize::deserialize(&Vec::<u8>::from(data)).ok_or(ModelError::Import)?;
                let mut proof: Proof = Default::default();
                proof.signature = signature;
                proof.workspace = workspace;
                self.history.add(proof::Action::Imported, proof);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("export failed")]
    Export(wasm_bindgen::JsValue),
    #[error("import failed")]
    Import,
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
    Settings,
    User,
}
