use homotopy_core::{Boundary, Direction, SliceIndex};
use paste::paste;

use crate::model::{history, proof, Action};

macro_rules! declare_keybindings {
    ($($key:expr => $action:expr)*) => {
        paste! {
            pub struct Keybindings;

            impl Keybindings {
                pub fn get_action(key: &str) -> Option<Action> {
                    match key {
                        $($key => Some($action)),*,
                        _ => None
                    }
                }

                pub fn get_shortcut(action: Action) -> Option<&'static str> {
                    match action {
                        $($action => Some($key)),*,
                        _ => None
                    }
                }

            }
        }
    }
}

declare_keybindings! {
    "y" => Action::History(history::Action::Move(history::Direction::Linear(Direction::Forward)))
    "u" => Action::History(history::Action::Move(history::Direction::Linear(Direction::Backward)))
    "d" => Action::Proof(proof::Action::Behead)
    "f" => Action::Proof(proof::Action::Befoot)
    "r" => Action::Proof(proof::Action::Restrict)
    "h" => Action::Proof(proof::Action::Theorem)
    "a" => Action::Proof(proof::Action::CreateGeneratorZero)
    "s" => Action::Proof(proof::Action::SetBoundary(Boundary::Source))
    "t" => Action::Proof(proof::Action::SetBoundary(Boundary::Target))
    "i" => Action::Proof(proof::Action::TakeIdentityDiagram)
    "c" => Action::Proof(proof::Action::ClearWorkspace)
    "arrowup" => Action::Proof(proof::Action::SwitchSlice(Direction::Forward))
    "arrowdown" => Action::Proof(proof::Action::SwitchSlice(Direction::Backward))
    "arrowleft" => Action::Proof(proof::Action::AscendSlice(1))
    "arrowright" => Action::Proof(proof::Action::DescendSlice(SliceIndex::Boundary(Boundary::Source)))
}
