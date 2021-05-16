use im::Vector;
use std::cell::Ref;
use thiserror::Error;
use yew::Callback;
pub mod proof;
use self::history::History;
use gloo_timers::callback::Timeout;
use proof::{Color, GeneratorInfo, Proof, Signature, Workspace};

pub mod history;
pub mod serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    ToggleDrawer(Drawer),
    Proof(proof::Action),
    History(history::Action),
    ImportProof(Vec<u8>),
    ExportProof,
    ShowToast(Toast),
    RemoveToast(usize),
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

#[derive(Debug, Clone, Default)]
pub struct State {
    pub history: History,
    pub drawer: Option<Drawer>,
    pub toaster: Toaster,
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

    /// Update the state in response to an [Action].
    pub fn update(&mut self, action: Action, dispatch: Callback<Action>) -> Result<(), ModelError> {
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
                    serialize::deserialize(&data).ok_or(ModelError::Import)?;
                let mut proof: Proof = Default::default();
                proof.signature = signature;
                proof.workspace = workspace;
                self.history.add(proof::Action::Imported, proof);
            }

            Action::ShowToast(toast) => {
                let next_id = self.toaster.next_id;
                self.toaster.next_id = next_id + 1;
                self.toaster.toasts.push_back((next_id, toast));

                Timeout::new(1500, {
                    move || dispatch.emit(Action::RemoveToast(next_id))
                })
                .forget();
            }

            Action::RemoveToast(id) => {
                self.toaster.toasts.retain(|(toast_id, _)| *toast_id != id);
            }
        }

        Ok(())
    }

    pub fn drawer(&self) -> Option<Drawer> {
        self.drawer
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
    User,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toaster {
    pub toasts: Vector<(ToastId, Toast)>,
    pub next_id: usize,
}

impl Default for Toaster {
    fn default() -> Self {
        Self {
            toasts: Default::default(),
            next_id: 0,
        }
    }
}

type ToastId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ToastKind {
    Success,
    Error,
}
