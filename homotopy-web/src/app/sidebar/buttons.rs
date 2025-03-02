use homotopy_core::{
    Boundary,
    Direction::{Backward, Forward},
};
use paste::paste;
use yew::prelude::*;

use super::{Sidebar, SidebarButton, SidebarMsg};
use crate::{app::keybindings::Keybindings, components::Visibility, model, model::history};

macro_rules! declare_sidebar_tools {
    ($($name:ident {
        $label:literal,
        $icon:literal,
        $action:expr,
    })*) => {
        paste! {
            impl Sidebar {
                pub(super) fn tools(&self, ctx: &Context<Self>) -> Html {
                    let dispatch = &ctx.link().callback(|x| x);
                    let proof = &ctx.props().proof;
                    html! {
                        <nav class="sidebar__tools">
                            $(<SidebarButton
                                label={$label}
                                icon={$icon}
                                action={SidebarMsg::Dispatch($action)}
                                shortcut={Keybindings::get_shortcut($action)}
                                dispatch={dispatch}
                                visibility={Visibility::from($action.is_valid(proof))}
                            />)*
                        </nav>
                    }
                }
            }
        }
    };
}

declare_sidebar_tools! {
    BUTTON_UNDO {
        "Undo",
        "undo",
        model::Action::History(history::Action::Move(history::Direction::Linear(Backward))),
    }

    BUTTON_REDO {
        "Redo",
        "redo",
        model::Action::History(history::Action::Move(history::Direction::Linear(Forward))),
    }

    BUTTON_BEHEAD {
        "Behead",
        "align_vertical_top",
        model::Action::Proof(model::proof::Action::Behead),
    }

    BUTTON_BEFOOT {
        "Befoot",
        "align_vertical_bottom",
        model::Action::Proof(model::proof::Action::Befoot),
    }

    BUTTON_INVERT {
        "Invert",
        "swap_calls",
        model::Action::Proof(model::proof::Action::Invert),
    }

    BUTTON_RESTRICT {
        "Restrict",
        "find_replace",
        model::Action::Proof(model::proof::Action::Restrict),
    }

    BUTTON_SQUASH {
        "Squash",
        "unfold_less",
        model::Action::Proof(model::proof::Action::Squash),
    }

    BUTTON_THEOREM {
        "Theorem",
        "title",
        model::Action::Proof(model::proof::Action::Theorem),
    }

    BUTTON_ADD_GENERATOR {
        "Add 0-cell",
        "add_circle_outline",
        model::Action::Proof(model::proof::Action::CreateGeneratorZero),
    }

    BUTTON_SOURCE {
        "Source",
        "arrow_circle_down",
        model::Action::Proof(model::proof::Action::SetBoundary(Boundary::Source)),
    }

    BUTTON_TARGET {
        "Target",
        "arrow_circle_up",
        model::Action::Proof(model::proof::Action::SetBoundary(Boundary::Target)),
    }

    BUTTON_IDENTITY {
        "Identity",
        "upgrade",
        model::Action::Proof(model::proof::Action::TakeIdentityDiagram),
    }

    BUTTON_CLEAR {
        "Clear",
        "clear",
        model::Action::Proof(model::proof::Action::ClearWorkspace),
    }

}
