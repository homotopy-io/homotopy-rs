mod panzoom;
mod diagram2d;
mod signature;
mod workspace;
mod signature_stylesheet;
use crate::model;
use homotopy_core::*;
use signature::SignatureView;
use workspace::WorkspaceView;
use signature_stylesheet::SignatureStylesheet;
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
            Icon { props }
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
    signature_stylesheet: SignatureStylesheet
}

impl Component for App {
    type Message = Message;
    type Properties = Props;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let state = model::State::default();
        let dispatch = link.callback(Message::Dispatch);
        let mut signature_stylesheet = SignatureStylesheet::new("generator");
        signature_stylesheet.update(state.signature().clone());
        signature_stylesheet.mount();
        App { state, dispatch, signature_stylesheet }
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
                self.signature_stylesheet.update(self.state.signature().clone());
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        use model::{Action, Drawer};
        let dispatch = &self.dispatch;

        let button_clear = self.view_sidebar_button(
            "Clear (C)",
            "clear",
            dispatch.reform(|_| Action::ClearWorkspace),
        );

        let button_identity = self.view_sidebar_button(
            "Identity (I)",
            "upgrade",
            dispatch.reform(|_| Action::TakeIdentityDiagram),
        );

        let button_source = self.view_sidebar_button(
            "Source (S)",
            "arrow_circle_down",
            dispatch.reform(|_| Action::SetBoundary(Boundary::Source)),
        );

        let button_target = self.view_sidebar_button(
            "Target (T)",
            "arrow_circle_up",
            dispatch.reform(|_| Action::SetBoundary(Boundary::Target)),
        );

        let button_create_generator_zero = self.view_sidebar_button(
            "Add Generator (A)",
            "add_circle_outline",
            dispatch.reform(|_| Action::CreateGeneratorZero),
        );

        let button_project = self.view_sidebar_button(
            "Project",
            "info",
            dispatch.reform(|_| Action::ToggleDrawer(Drawer::Project)),
        );

        let button_signature = self.view_sidebar_button(
            "Signature",
            "list",
            dispatch.reform(|_| Action::ToggleDrawer(Drawer::Signature)),
        );

        let button_user = self.view_sidebar_button(
            "User",
            "perm_identity",
            dispatch.reform(|_| Action::ToggleDrawer(Drawer::User)),
        );

        let workspace = match self.state.workspace() {
            Some(workspace) => {
                html! {
                    <WorkspaceView
                        workspace={workspace}
                        dispatch={dispatch}
                    />
                }
            }
            _ => {
                // TODO: Show onboarding info if workspace and signature is empty
                html! {
                    <content class="workspace workspace--empty">
                    </content>
                }
            }
        };

        let drawer = match self.state.drawer() {
            Some(Drawer::Project) => Default::default(),
            Some(Drawer::Signature) => {
                html! {
                    <SignatureView
                        signature={self.state.signature()}
                        dispatch={dispatch}
                    />
                }
            }
            Some(Drawer::User) => Default::default(),
            None => Default::default(),
        };

        html! {
            <main class="app">
                <aside class="sidebar">
                    <img src="/logo.svg" class="sidebar__logo" />
                    <nav class="sidebar__nav">
                        {button_project}
                        {button_signature}
                        {button_user}
                    </nav>
                    <nav class="sidebar__tools">
                        {button_create_generator_zero}
                        {button_source}
                        {button_target}
                        {button_identity}
                        {button_clear}
                    </nav>
                </aside>
                {drawer}
                {workspace}
            </main>
        }
    }

    fn destroy(&mut self) {
        self.signature_stylesheet.unmount();
    }
}

impl App {
    fn view_sidebar_button(&self, label: &str, icon: &str, callback: Callback<MouseEvent>) -> Html {
        html! {
            <div class="sidebar__button tooltip tooltip--right" onclick={callback} data-tooltip={label}>
                <Icon name={icon} />
            </div>
        }
    }
}
