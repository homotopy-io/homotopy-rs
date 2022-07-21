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
    "1" => Action::Proof(proof::Action::SelectGeneratorAt(0))
    "2" => Action::Proof(proof::Action::SelectGeneratorAt(1))
    "3" => Action::Proof(proof::Action::SelectGeneratorAt(2))
    "4" => Action::Proof(proof::Action::SelectGeneratorAt(3))
    "5" => Action::Proof(proof::Action::SelectGeneratorAt(4))
    "6" => Action::Proof(proof::Action::SelectGeneratorAt(5))
    "7" => Action::Proof(proof::Action::SelectGeneratorAt(6))
    "8" => Action::Proof(proof::Action::SelectGeneratorAt(7))
    "9" => Action::Proof(proof::Action::SelectGeneratorAt(8))
    "arrowup" => Action::Proof(proof::Action::SwitchSlice(Direction::Forward))
    "arrowdown" => Action::Proof(proof::Action::SwitchSlice(Direction::Backward))
    "arrowleft" => Action::Proof(proof::Action::AscendSlice(1))
    "arrowright" => Action::Proof(proof::Action::DescendSlice(SliceIndex::Boundary(Boundary::Source)))
}
