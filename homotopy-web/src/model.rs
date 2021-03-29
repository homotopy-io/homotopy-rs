use std::convert::*;
use thiserror::Error;
pub mod proof;
use proof::*;
pub mod serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    ToggleDrawer(Drawer),

    Proof(proof::Action),

    Serialize(serialize::Serialize),
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub(crate) proof: Proof,
    drawer: Option<Drawer>,
}

impl State {
    /// Update the state in response to an [Action].
    pub fn update(&mut self, action: Action) -> Result<(), ModelError> {
        match action {
            Action::ToggleDrawer(drawer) => self.toggle_drawer(drawer),
            Action::Serialize(serialize::Serialize::Export) => self.export(),
            Action::Serialize(serialize::Serialize::Import(signature, workspace)) => {
                self.import(signature, workspace)
            }
            Action::Proof(action) => self.proof.update(action).map_err(|e| e.into()),
        }
    }

    /// Handler for [Action::ToggleDrawer].
    fn toggle_drawer(&mut self, drawer: Drawer) -> Result<(), ModelError> {
        if self.drawer == Some(drawer) {
            self.drawer = None;
        } else {
            self.drawer = Some(drawer);
        }

        Ok(())
    }

    fn export(&self) -> Result<(), ModelError> {
        let data: serialize::Data = if let Some(ws) = self.proof.workspace.clone() {
            (self.proof.signature.clone(), ws).into()
        } else {
            self.proof.signature.clone().into()
        };
        serialize::generate_download(
            "filename_todo.hom".to_string(),
            &Into::<Vec<u8>>::into(data).as_slice(),
        )
        .expect("failed to generate download");
        Ok(())
    }

    fn import(
        &mut self,
        signature: Signature,
        workspace: Option<Workspace>,
    ) -> Result<(), ModelError> {
        self.proof.signature = signature;
        self.proof.workspace = workspace;
        Ok(())
    }

    pub fn drawer(&self) -> Option<Drawer> {
        self.drawer
    }
}

#[derive(Debug, Error)]
pub enum ModelError {
    #[error(transparent)]
    ProofError(#[from] proof::ModelError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Drawer {
    Project,
    Signature,
    User,
}
