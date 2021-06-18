mod attach;
mod diagram2d;
mod panzoom;
mod project;
mod settings;
mod signature;
mod signature_stylesheet;
mod util;
mod workspace;

use crate::declare_settings;
use crate::components::icon::{Icon, IconSize};
use crate::components::sidebar::{SidebarButton, SidebarButtonDesc};
use crate::components::settings::{Settings, SettingsAgent};
use crate::components::toast::{Toast, ToastAgent, Toaster};
use crate::components::Visibility;

use crate::model::Drawer;
use crate::model::{self, history};

use attach::AttachView;
use homotopy_core::{diagram::globularity, Direction};
use homotopy_core::{
    Boundary,
    Direction::{Backward, Forward},
    Height, SliceIndex,
};
use project::ProjectView;
use settings::SettingsView;
use signature::SignatureView;
use signature_stylesheet::SignatureStylesheet;
use wasm_bindgen::JsCast;
use workspace::WorkspaceView;

use yew::agent::{Bridge, Dispatcher};
use yew::prelude::*;

macro_rules! declare_sidebar_buttons {
    ($(($name:ident, $label:literal, $icon:literal, $shortcut:expr, $action:expr,),)*) => {
        $(const $name: SidebarButtonDesc = SidebarButtonDesc {
            label: $label,
            icon: $icon,
            action: {$action},
            shortcut: {$shortcut},
        };)*
        const BUTTONS: &[&SidebarButtonDesc] = &[
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
    (
        BUTTON_PROJECT,
        "Project",
        "info",
        None,
        model::Action::ToggleDrawer(model::Drawer::Project),
    ),
    (
        BUTTON_SIGNATURE,
        "Signature",
        "list",
        None,
        model::Action::ToggleDrawer(model::Drawer::Signature),
    ),
    (
        BUTTON_SETTINGS,
        "Settings",
        "settings",
        None,
        model::Action::ToggleDrawer(model::Drawer::Settings),
    ),
    //  (
    //      BUTTON_USER, "User", "perm_identity", None,
    //      model::Action::ToggleDrawer(model::Drawer::User),
    //  )
];

declare_settings! {
    pub struct GlobalSettings {
        type Key = Setting;
        type Message = SettingPayload;

        example_toggle: bool;
    }
}

pub type AppSettings = SettingsAgent<GlobalSettings>;

#[derive(Default, Clone, Debug, PartialEq, Properties)]
pub struct Props {}

#[derive(Debug, Clone)]
pub enum Message {
    Dispatch(model::Action),
    Setting(SettingPayload),
}

pub struct App {
    dispatch: Callback<model::Action>,
    state: model::State,
    signature_stylesheet: SignatureStylesheet,
    toaster: Dispatcher<ToastAgent>,
    settings: Box<dyn Bridge<AppSettings>>,
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

        let mut settings = AppSettings::bridge(link.callback(Message::Setting));
        settings.send(Settings::Subscribe(Setting::example_toggle));

        Self {
            dispatch,
            state,
            signature_stylesheet,
            toaster: ToastAgent::dispatcher(),
            settings,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::Dispatch(action) => {
                log::info!("Received action: {:?}", action);

                let time_start = performance();
                let result = self.state.update(action);
                let time_stop = performance();
                log::info!("State update took {}ms.", time_stop - time_start);

                homotopy_core::collect_garbage();

                match result {
                    Ok(()) => {}
                    Err(error) => {
                        self.toaster.send(Toast::error(format!("{}", error)));
                        log::error!("Error occured: {}", error);
                    }
                }
                self.signature_stylesheet
                    .update(self.state.proof().signature().clone());
                true
            }
            Message::Setting(SettingPayload::example_toggle(true)) => {
                self.toaster.send(Toast::success("Toggle on!"));
                false
            }
            Message::Setting(SettingPayload::example_toggle(false)) => {
                self.toaster.send(Toast::error("Toggle off!"));
                false
            }
        }
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

        let restrict_visible: Visibility = proof
            .workspace()
            .map_or(false, |ws| {
                !ws.path.is_empty()
                    && ws.path.iter().all(|s| {
                        matches!(s, SliceIndex::Boundary(_))
                            || matches!(s, SliceIndex::Interior(Height::Regular(_)))
                    })
            })
            .into();
        let theorem_visible: Visibility = proof
            .workspace()
            .map_or(false, |ws| ws.diagram.dimension() > 0)
            .into();
        let source_visible: Visibility = proof
            .workspace()
            .map_or(false, |ws| {
                proof.boundary().map_or(true, |b| {
                    b.boundary != Boundary::Target || globularity(&b.diagram, &ws.diagram)
                })
            })
            .into();
        let target_visible: Visibility = proof
            .workspace()
            .map_or(false, |ws| {
                proof.boundary().map_or(true, |b| {
                    b.boundary != Boundary::Source || globularity(&b.diagram, &ws.diagram)
                })
            })
            .into();

        html! {
            <main class="app">
                <aside class="sidebar">
                    <a href="https://ncatlab.org/nlab/show/homotopy.io">
                        <img src="/logo.svg" class="sidebar__logo" />
                    </a>
                    <nav class="sidebar__nav">
                        <SidebarButton desc={BUTTON_PROJECT} dispatch={dispatch} />
                        <SidebarButton desc={BUTTON_SETTINGS} dispatch={dispatch} />
                        <SidebarButton desc={BUTTON_SIGNATURE} dispatch={dispatch} />
                        // <SidebarButton desc={BUTTON_USER} dispatch={dispatch} />
                    </nav>
                    <nav class="sidebar__tools">
                        <SidebarButton
                            desc={BUTTON_UNDO}
                            dispatch={dispatch}
                            visibility={Visibility::from(self.state.can_undo())}
                        />
                        <SidebarButton
                            desc={BUTTON_REDO}
                            dispatch={dispatch}
                            visibility={Visibility::from(self.state.can_redo())}
                        />
                        <SidebarButton desc={BUTTON_RESTRICT} dispatch={dispatch} visibility={restrict_visible} />
                        <SidebarButton desc={BUTTON_THEOREM} dispatch={dispatch} visibility={theorem_visible} />
                        <SidebarButton desc={BUTTON_ADD_GENERATOR} dispatch={dispatch} />
                        <SidebarButton desc={BUTTON_SOURCE} dispatch={dispatch} visibility={source_visible} />
                        <SidebarButton desc={BUTTON_TARGET} dispatch={dispatch} visibility={target_visible} />
                        <SidebarButton
                            desc={BUTTON_IDENTITY}
                            dispatch={dispatch}
                            visibility={Visibility::from(proof.workspace().is_some())}
                        />
                        <SidebarButton
                            desc={BUTTON_CLEAR}
                            dispatch={dispatch}
                            visibility={Visibility::from(proof.workspace().is_some())}
                        />
                    </nav>
                </aside>
                {self.drawer()}
                <Toaster timeout={3000} />
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
            Some(Drawer::Settings) => {
                html! {
                    <SettingsView />
                }
            }
            Some(Drawer::User) | None => Default::default(),
        }
    }

    fn install_keyboard_shortcuts(dispatch: Callback<model::Action>) {
        let onkeyup =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                let key = event.key().to_ascii_lowercase();
                let button = BUTTONS.iter().find(|button| match button.shortcut {
                    Some(shortcut) => shortcut.to_string() == key,
                    None => false,
                });

                if let Some(button) = button {
                    dispatch.emit(button.action.clone());
                } else if key == "arrowup" {
                    dispatch.emit(model::proof::Action::SwitchSlice(Direction::Forward).into());
                } else if key == "arrowdown" {
                    dispatch.emit(model::proof::Action::SwitchSlice(Direction::Backward).into());
                } else if key == "arrowleft" {
                    dispatch.emit(model::proof::Action::AscendSlice(1).into());
                }
            }) as Box<dyn FnMut(_)>);

        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("keyup", onkeyup.as_ref().unchecked_ref())
            .unwrap();

        onkeyup.forget();
    }
}

fn performance() -> f64 {
    web_sys::window().unwrap().performance().unwrap().now()
}
