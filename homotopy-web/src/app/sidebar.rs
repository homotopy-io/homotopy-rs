use yew::prelude::*;
use yew_functional::function_component;

use homotopy_core::diagram::globularity;
use homotopy_core::Direction::{Backward, Forward};
use homotopy_core::{Boundary, Direction, Height, SliceIndex};

use crate::model::proof::Proof;
use crate::model::{self, history};

use crate::components::sidebar::{SidebarButton, SidebarButtonDesc, SidebarDrawerProps};
use crate::components::Visibility;

use super::attach::AttachView;
use super::project::ProjectView;
use super::settings::SettingsView;
use super::signature::SignatureView;

macro_rules! declare_sidebar_buttons {
    ($(($name:ident, $label:literal, $icon:literal, $shortcut:expr, $action:expr,),)*) => {
        $(pub const $name: SidebarButtonDesc = SidebarButtonDesc {
            label: $label,
            icon: $icon,
            action: {$action},
            shortcut: {$shortcut},
        };)*
        pub const BUTTONS: &[&SidebarButtonDesc] = &[
            $(&$name),*
        ];
    }
}

declare_sidebar_buttons![
    (
        BUTTON_UNDO,
        "Undo",
        "undo",
        Some('u'),
        model::Action::History(history::Action::Move(history::Direction::Linear(Backward))),
    ),
    (
        BUTTON_REDO,
        "Redo",
        "redo",
        None,
        model::Action::History(history::Action::Move(history::Direction::Linear(Forward))),
    ),
    (
        BUTTON_CLEAR,
        "Clear",
        "clear",
        Some('c'),
        model::Action::Proof(model::proof::Action::ClearWorkspace),
    ),
    (
        BUTTON_IDENTITY,
        "Identity",
        "upgrade",
        Some('i'),
        model::Action::Proof(model::proof::Action::TakeIdentityDiagram),
    ),
    (
        BUTTON_SOURCE,
        "Source",
        "arrow_circle_down",
        Some('s'),
        model::Action::Proof(model::proof::Action::SetBoundary(Boundary::Source)),
    ),
    (
        BUTTON_TARGET,
        "Target",
        "arrow_circle_up",
        Some('t'),
        model::Action::Proof(model::proof::Action::SetBoundary(Boundary::Target)),
    ),
    (
        BUTTON_ADD_GENERATOR,
        "Add Generator",
        "add_circle_outline",
        Some('a'),
        model::Action::Proof(model::proof::Action::CreateGeneratorZero),
    ),
    (
        BUTTON_RESTRICT,
        "Restrict",
        "find_replace",
        Some('r'),
        model::Action::Proof(model::proof::Action::Restrict),
    ),
    (
        BUTTON_THEOREM,
        "Theorem",
        "title",
        Some('h'),
        model::Action::Proof(model::proof::Action::Theorem),
    ),
];

// TODO(@doctorn) generate this code
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Drawer {
    Project,
    Settings,
    Signature,
}

// TODO(@doctorn) generate this code
impl Drawer {
    fn view(self, dispatch: &Callback<model::Action>, proof: &Proof) -> Html {
        match self {
            Drawer::Project => {
                html! {
                    <ProjectView dispatch={dispatch} />
                }
            }
            Drawer::Signature => {
                html! {
                    <SignatureView
                        signature={proof.signature()}
                        dispatch={dispatch.reform(model::Action::Proof)}
                    />
                }
            }
            Drawer::Settings => {
                html! {
                    <SettingsView />
                }
            }
        }
    }

    fn title(self) -> &'static str {
        match self {
            Drawer::Project => "Project",
            Drawer::Signature => "List",
            Drawer::Settings => "Settings",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Drawer::Project => "info",
            Drawer::Signature => "list",
            Drawer::Settings => "settings",
        }
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct SidebarProps {
    pub proof: Proof,
    pub dispatch: Callback<model::Action>,
}

pub enum SidebarMsg {
    Toggle(Drawer),
    Dispatch(model::Action),
}

pub struct Sidebar {
    props: SidebarProps,
    link: ComponentLink<Self>,
    open: Option<Drawer>,
}

impl Component for Sidebar {
    type Properties = SidebarProps;
    type Message = SidebarMsg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            open: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            SidebarMsg::Toggle(drawer) if Some(drawer) == self.open => {
                self.open = None;
                true
            }
            SidebarMsg::Toggle(drawer) => {
                self.open = Some(drawer);
                true
            }
            SidebarMsg::Dispatch(action) => {
                self.props.dispatch.emit(action);
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <aside class="sidebar">
                    <a href="https://ncatlab.org/nlab/show/homotopy.io">
                        <img src="/logo.svg" class="sidebar__logo" />
                    </a>
                    {self.nav()}
                    {self.tools()}
                </aside>
                {self.drawer()}
            </>
        }
    }
}

// TODO(@doctorn) generate this code
impl Sidebar {
    fn restrict_visible(&self) -> Visibility {
        self.props
            .proof
            .workspace()
            .map_or(false, |ws| {
                !ws.path.is_empty()
                    && ws.path.iter().all(|s| {
                        matches!(s, SliceIndex::Boundary(_))
                            || matches!(s, SliceIndex::Interior(Height::Regular(_)))
                    })
            })
            .into()
    }

    fn theorem_visible(&self) -> Visibility {
        self.props
            .proof
            .workspace()
            .map_or(false, |ws| ws.diagram.dimension() > 0)
            .into()
    }

    fn source_visible(&self) -> Visibility {
        self.props
            .proof
            .workspace()
            .map_or(false, |ws| {
                self.props.proof.boundary().map_or(true, |b| {
                    b.boundary != Boundary::Target || globularity(&b.diagram, &ws.diagram)
                })
            })
            .into()
    }
    fn target_visible(&self) -> Visibility {
        self.props
            .proof
            .workspace()
            .map_or(false, |ws| {
                self.props.proof.boundary().map_or(true, |b| {
                    b.boundary != Boundary::Source || globularity(&b.diagram, &ws.diagram)
                })
            })
            .into()
    }

    fn identity_visible(&self) -> Visibility {
        self.props.proof.workspace().is_some().into()
    }

    fn clear_visible(&self) -> Visibility {
        self.props.proof.workspace().is_some().into()
    }

    fn tools(&self) -> Html {
        // TODO(@doctorn) fix undo & redo
        let dispatch = &self.props.dispatch;
        html! {
            <nav class="sidebar__tools">
                <SidebarButton
                    desc={BUTTON_UNDO}
                    dispatch={dispatch}
                    visibility={Visibility::Visible}
                />
                <SidebarButton
                    desc={BUTTON_REDO}
                    dispatch={dispatch}
                    visibility={Visibility::Visible}
                />
                <SidebarButton
                    desc={BUTTON_RESTRICT}
                    dispatch={dispatch}
                    visibility={self.restrict_visible()}
                />
                <SidebarButton
                    desc={BUTTON_THEOREM}
                    dispatch={dispatch}
                    visibility={self.theorem_visible()}
                />
                <SidebarButton
                    desc={BUTTON_ADD_GENERATOR}
                    dispatch={dispatch}
                />
                <SidebarButton
                    desc={BUTTON_SOURCE}
                    dispatch={dispatch}
                    visibility={self.source_visible()}
                />
                <SidebarButton
                    desc={BUTTON_TARGET}
                    dispatch={dispatch}
                    visibility={self.target_visible()}
                />
                <SidebarButton
                    desc={BUTTON_IDENTITY}
                    dispatch={dispatch}
                    visibility={self.identity_visible()}
                />
                <SidebarButton
                    desc={BUTTON_CLEAR}
                    dispatch={dispatch}
                    visibility={self.clear_visible()}
                />
            </nav>
        }
    }
}

impl Sidebar {
    fn nav(&self) -> Html {
        html! {
            <nav class="sidebar__nav">
            </nav>
        }
    }
}

impl Sidebar {
    fn drawer(&self) -> Html {
        let dispatch = &self.props.dispatch;
        let attach_options = self
            .props
            .proof
            .workspace()
            .and_then(|workspace| workspace.attach.clone());

        if let Some(attach_options) = attach_options {
            return html! {
                <AttachView
                    dispatch={dispatch.reform(model::Action::Proof)}
                    options={attach_options}
                    signature={self.props.proof.signature()}
                />
            };
        }

        if let Some(drawer) = self.open {
            drawer.view(dispatch, &self.props.proof)
        } else {
            Default::default()
        }
    }
}
