mod attach;
mod diagram2d;
mod panzoom;
mod project;
mod signature;
mod signature_stylesheet;
mod workspace;
use crate::model::Drawer;
use crate::model::{self, history};
use attach::AttachView;
use homotopy_core::{
    Boundary,
    Direction::{Backward, Forward},
};
use project::ProjectView;
use signature::SignatureView;
use signature_stylesheet::SignatureStylesheet;
use wasm_bindgen::JsCast;
use workspace::WorkspaceView;
use yew::prelude::*;

pub mod icon {
    use yew::prelude::*;
    use yewtil::NeqAssign;

    #[derive(Debug, Clone, PartialEq, Properties)]
    pub struct Props {
        pub name: String,
    }

    pub struct Icon {
        props: Props,
    }

    impl Component for Icon {
        type Message = ();
        type Properties = Props;

        fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
            Self { props }
        }

        fn update(&mut self, _msg: Self::Message) -> ShouldRender {
            false
        }

        fn change(&mut self, props: Self::Properties) -> ShouldRender {
            self.props.neq_assign(props)
        }

        fn view(&self) -> Html {
            html! {
                <i class="material-icons md-light">{&self.props.name}</i>
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SidebarButton {
    label: &'static str,
    icon: &'static str,
    action: model::Action,
    shortcut: Option<char>,
}

impl SidebarButton {
    pub fn view(&self, dispatch: &Callback<model::Action>) -> Html {
        let action = self.action.clone();

        html! {
            <div
                class="sidebar__button tooltip tooltip--right"
                onclick={dispatch.reform(move |_| action.clone())}
                data-tooltip={self.label}
            >
                <Icon name={self.icon} />
            </div>
        }
    }
}

// TODO: Automatically add shortcut name to label

const BUTTON_UNDO: SidebarButton = SidebarButton {
    label: "Undo (U)",
    icon: "undo",
    action: model::Action::History(history::Action::Move(history::Direction::Linear(Backward))),
    shortcut: Some('u'),
};

const BUTTON_REDO: SidebarButton = SidebarButton {
    label: "Redo",
    icon: "redo",
    action: model::Action::History(history::Action::Move(history::Direction::Linear(Forward))),
    shortcut: None,
};

const BUTTON_CLEAR: SidebarButton = SidebarButton {
    label: "Clear (C)",
    icon: "clear",
    action: model::Action::Proof(model::proof::Action::ClearWorkspace),
    shortcut: Some('c'),
};

const BUTTON_IDENTITY: SidebarButton = SidebarButton {
    label: "Identity (I)",
    icon: "upgrade",
    action: model::Action::Proof(model::proof::Action::TakeIdentityDiagram),
    shortcut: Some('i'),
};

const BUTTON_SOURCE: SidebarButton = SidebarButton {
    label: "Source (S)",
    icon: "arrow_circle_down",
    action: model::Action::Proof(model::proof::Action::SetBoundary(Boundary::Source)),
    shortcut: Some('s'),
};

const BUTTON_TARGET: SidebarButton = SidebarButton {
    label: "Target (T)",
    icon: "arrow_circle_up",
    action: model::Action::Proof(model::proof::Action::SetBoundary(Boundary::Target)),
    shortcut: Some('t'),
};

const BUTTON_ADD_GENERATOR: SidebarButton = SidebarButton {
    label: "Add Generator (A)",
    icon: "add_circle_outline",
    action: model::Action::Proof(model::proof::Action::CreateGeneratorZero),
    shortcut: Some('a'),
};

const BUTTON_PROJECT: SidebarButton = SidebarButton {
    label: "Project",
    icon: "info",
    action: model::Action::ToggleDrawer(model::Drawer::Project),
    shortcut: None,
};

const BUTTON_SIGNATURE: SidebarButton = SidebarButton {
    label: "Signature",
    icon: "list",
    action: model::Action::ToggleDrawer(model::Drawer::Signature),
    shortcut: None,
};

const BUTTON_USER: SidebarButton = SidebarButton {
    label: "User",
    icon: "perm_identity",
    action: model::Action::ToggleDrawer(model::Drawer::User),
    shortcut: None,
};

const BUTTONS: &[&SidebarButton] = &[
    &BUTTON_UNDO,
    &BUTTON_REDO,
    &BUTTON_CLEAR,
    &BUTTON_IDENTITY,
    &BUTTON_SOURCE,
    &BUTTON_TARGET,
    &BUTTON_ADD_GENERATOR,
    &BUTTON_PROJECT,
    &BUTTON_SIGNATURE,
    &BUTTON_USER,
];

use icon::Icon;

#[derive(Default, Clone, Debug, PartialEq, Properties)]
pub struct Props {}

#[derive(Debug, Clone)]
pub enum Message {
    Dispatch(model::Action),
}

pub struct App {
    dispatch: Callback<model::Action>,
    state: model::State,
    signature_stylesheet: SignatureStylesheet,
}

impl Component for App {
    type Message = Message;
    type Properties = Props;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let state = model::State::default();
        let dispatch = link.callback(Message::Dispatch);

        // Install the signature stylesheet
        let mut signature_stylesheet = SignatureStylesheet::new("generator");
        signature_stylesheet.update(state.proof().signature().clone());
        signature_stylesheet.mount();

        // Install the keyboard listener for shortcuts
        // TODO: Remove these when App is destroyed.
        Self::install_keyboard_shortcuts(dispatch.clone());

        Self {
            dispatch,
            state,
            signature_stylesheet,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::Dispatch(action) => {
                log::info!("Received action: {:?}", action);
                match self.state.update(action) {
                    Ok(()) => {}
                    Err(error) => {
                        // TODO: Display a toast
                        log::error!("Error occured: {}", error);
                    }
                }
                self.signature_stylesheet
                    .update(self.state.proof().signature().clone());
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let dispatch = &self.dispatch;
        let proof = self.state.proof();
        let signature = proof.signature();

        let workspace = match self.state.proof().workspace() {
            Some(workspace) => {
                html! {
                    <WorkspaceView
                        workspace={workspace}
                        signature={signature}
                        dispatch={dispatch.reform(model::Action::Proof)}
                    />
                }
            }
            None => {
                // TODO: Show onboarding info if workspace and signature is empty
                html! {
                    <content class="workspace workspace--empty">
                    </content>
                }
            }
        };

        let drawer = self.drawer();

        html! {
            <main class="app">
                <aside class="sidebar">
                    <img src="/logo.svg" class="sidebar__logo" />
                    <nav class="sidebar__nav">
                        {BUTTON_PROJECT.view(dispatch)}
                        {BUTTON_SIGNATURE.view(dispatch)}
                        {BUTTON_USER.view(dispatch)}
                    </nav>
                    <nav class="sidebar__tools">
                        {BUTTON_UNDO.view(dispatch)}
                        {BUTTON_REDO.view(dispatch)}
                        {BUTTON_ADD_GENERATOR.view(dispatch)}
                        {BUTTON_SOURCE.view(dispatch)}
                        {BUTTON_TARGET.view(dispatch)}
                        {BUTTON_IDENTITY.view(dispatch)}
                        {BUTTON_CLEAR.view(dispatch)}
                    </nav>
                </aside>
                {drawer}
                {workspace}
                <span class="version">
                    {format!("Version: {}", option_env!("GIT_DESCRIBE").unwrap_or(env!("CARGO_PKG_VERSION")))}
                </span>
            </main>
        }
    }

    fn destroy(&mut self) {
        self.signature_stylesheet.unmount();
    }
}

impl App {
    fn drawer(&self) -> Html {
        let dispatch = &self.dispatch;
        let attach_options = self
            .state
            .proof()
            .workspace()
            .and_then(|workspace| workspace.attach.clone());

        if let Some(attach_options) = attach_options {
            return html! {
                <AttachView
                    dispatch={dispatch.reform(model::Action::Proof)}
                    options={attach_options}
                    signature={self.state.proof().signature()}
                />
            };
        }

        match self.state.drawer() {
            Some(Drawer::Project) => {
                html! {
                    <ProjectView dispatch={dispatch} />
                }
            }
            Some(Drawer::Signature) => {
                html! {
                    <SignatureView
                        signature={self.state.proof().signature()}
                        dispatch={dispatch.reform(model::Action::Proof)}
                    />
                }
            }
            Some(Drawer::User) | None => Default::default(),
        }
    }

    fn install_keyboard_shortcuts(dispatch: Callback<model::Action>) {
        let onkeypress =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                let key = event.key().chars().next().unwrap();
                let button = BUTTONS.iter().find(|button| button.shortcut == Some(key));

                if let Some(button) = button {
                    dispatch.emit(button.action.clone());
                }
            }) as Box<dyn FnMut(_)>);

        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("keypress", onkeypress.as_ref().unchecked_ref())
            .unwrap();

        onkeypress.forget();
    }
}
