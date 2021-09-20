use history::History;
pub use history::Proof;
use homotopy_core::common::Mode;
use proof::{Color, GeneratorInfo, Signature, Workspace};
use thiserror::Error;

pub mod history;
pub mod proof;
pub mod serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Proof(proof::Action),
    History(history::Action),
    ImportProof(SerializedData),
    ExportProof,
    Test,
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
}

impl State {
    #[inline]
    pub(super) fn with_proof<F, U>(&self, f: F) -> U
    where
        F: Fn(&Proof) -> U,
    {
        self.history.with_proof(f)
    }

    /// Update the state in response to an [Action].
    pub fn update(&mut self, action: Action) -> Result<(), ModelError> {
        match action {
            Action::Proof(action) => {
                let mut proof = self.with_proof(Clone::clone);
                proof.update(&action).map_err(ModelError::from)?;
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
                    self.with_proof(|p| p.signature.clone()),
                    self.with_proof(|p| p.workspace.clone()),
                );
                serialize::generate_download("filename_todo", data.as_slice())
                    .map_err(ModelError::Export)?;
            }

            Action::ImportProof(data) => {
                let (signature, workspace) =
                    serialize::deserialize(&Vec::<u8>::from(data)).ok_or(ModelError::Import)?;
                if let Some(w) = workspace.as_ref() {
                    if w.diagram.check_well_formed(Mode::Deep).is_err() {
                        return Err(ModelError::Import);
                    }
                }
                let mut proof: Proof = Default::default();
                proof.signature = signature;
                proof.workspace = workspace;
                self.history.add(proof::Action::Imported, proof);
            }

            Action::Test => {
                use std::convert::TryInto;

                use homotopy_core::DiagramN;
                let result = self.with_proof(|p| {
                    p.workspace.as_ref().map(|ws| {
                        let diagram: DiagramN = ws.diagram.clone().try_into().unwrap();
                        // let mut simplices: Vec<
                        //     std::collections::HashSet<
                        //         homotopy_core::labelled::Simplex<homotopy_core::Height>,
                        //     >,
                        // > = homotopy_core::labelled::simplices(diagram.into());
                        // simplices.swap_remove(1);
                        diagram
                        // let cs = &diagram.cospans()[0];
                        // (cs.forward.get_label(), cs.backward.get_label())
                        // homotopy_core::labelled::simplices(workspace.diagram.clone())
                    })
                });
                log::info!("Test result: {:#?}", result);
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
