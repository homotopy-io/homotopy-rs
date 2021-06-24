use yew::prelude::*;

use homotopy_core::diagram::globularity;
use homotopy_core::Direction::{Backward, Forward};
use homotopy_core::{Boundary, Height, SliceIndex};

use paste::paste;

use crate::components::{Visibility, Visible};
use crate::model;
use crate::model::history::{self, Proof};

use super::{Sidebar, SidebarButton, SidebarMsg};

macro_rules! declare_sidebar_tools {
    ($($name:ident {
        $label:literal,
        $icon:literal,
        $action:expr,
        $shortcut:expr,
        $enable:expr,
    })*) => {
        paste! {
            #[allow(unused)]
            #[allow(non_camel_case_types)]
            #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
            pub enum ToolButton {
                $($name),*
            }

            pub const TOOL_BUTTONS: &'static [ToolButton] = &[
                $(ToolButton::$name),*
            ];

            impl ToolButton {
                pub fn shortcut(self) -> Option<char> {
                    match self {
                        $(ToolButton::$name => $shortcut),*
                    }
                }

                pub fn action(self) -> model::Action {
                    match self {
                        $(ToolButton::$name => $action),*
                    }
                }
            }

            impl Sidebar {
                $(
                    #[allow(non_snake_case)]
                    fn [<$name _visible>](&self) -> Visibility {
                        let enable = $enable;
                        enable(&self.props.proof)
                    }
                )*

                pub(super) fn tools(&self) -> Html {
                    let dispatch = &self.link.callback(|x| x);
                    html! {
                        <nav class="sidebar__tools">
                            $(<SidebarButton
                                label={$label}
                                icon={$icon}
                                action={SidebarMsg::Dispatch($action)}
                                shortcut={$shortcut}
                                dispatch=dispatch
                                visibility={self.[<$name _visible>]()}
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
        Some('u'),
        |proof: &Proof| proof.can_undo().into(),
    }

    BUTTON_REDO {
        "Redo",
        "redo",
        model::Action::History(history::Action::Move(history::Direction::Linear(Forward))),
        None,
        |proof: &Proof| proof.can_redo().into(),
    }

    BUTTON_RESTRICT {
        "Restrict",
        "find_replace",
        model::Action::Proof(model::proof::Action::Restrict),
        Some('r'),
        |proof: &Proof| {
            proof.workspace()
                .map_or(false, |ws| {
                    !ws.path.is_empty()
                        && ws.path.iter().all(|s| {
                            matches!(s, SliceIndex::Boundary(_))
                                || matches!(s, SliceIndex::Interior(Height::Regular(_)))
                        })
                })
                .into()
        },
    }

    BUTTON_THEOREM {
        "Theorem",
        "title",
        model::Action::Proof(model::proof::Action::Theorem),
        Some('h'),
        |proof: &Proof| {
            proof.workspace()
                .map_or(false, |ws| ws.diagram.dimension() > 0)
                .into()
        },
    }

    BUTTON_ADD_GENERATOR {
        "Add Generator",
        "add_circle_outline",
        model::Action::Proof(model::proof::Action::CreateGeneratorZero),
        Some('a'),
        |_: &Proof| Visible,
    }

    BUTTON_SOURCE {
        "Source",
        "arrow_circle_down",
        model::Action::Proof(model::proof::Action::SetBoundary(Boundary::Source)),
        Some('s'),
        |proof: &Proof| {
            proof.workspace()
                .map_or(false, |ws| {
                    proof.boundary().map_or(true, |b| {
                        b.boundary != Boundary::Target
                            || globularity(&b.diagram, &ws.diagram)
                    })
                })
                .into()
        },
    }

    BUTTON_TARGET {
        "Target",
        "arrow_circle_up",
        model::Action::Proof(model::proof::Action::SetBoundary(Boundary::Target)),
        Some('t'),
        |proof: &Proof| {
            proof.workspace()
                .map_or(false, |ws| {
                    proof.boundary().map_or(true, |b| {
                        b.boundary != Boundary::Source
                            || globularity(&b.diagram, &ws.diagram)
                    })
                })
                .into()
        },
    }

    BUTTON_IDENTITY {
        "Identity",
        "upgrade",
        model::Action::Proof(model::proof::Action::TakeIdentityDiagram),
        Some('i'),
        |proof: &Proof| proof.workspace().is_some().into(),
    }

    BUTTON_CLEAR {
        "Clear",
        "clear",
        model::Action::Proof(model::proof::Action::ClearWorkspace),
        Some('c'),
        |proof: &Proof| proof.workspace().is_some().into(),
    }
}
