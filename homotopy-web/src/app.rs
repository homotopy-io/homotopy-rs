mod diagram2d;
mod panzoom;
mod signature;
use crate::model;
use closure::closure;
use diagram2d::Diagram2D;
use homotopy_core::*;
use panzoom::PanZoom;
use serde::{Deserialize, Serialize};
use signature::SignatureView;
use std::collections::HashMap;
use std::convert::TryInto;
use std::rc::Rc;
use yew::prelude::*;

mod icon {
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
}

impl Component for App {
    type Message = Message;
    type Properties = Props;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let state = Default::default();
        let dispatch = link.callback(Message::Dispatch);
        App { state, dispatch }
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
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let example = {
            let x = Generator {
                id: 0,
                dimension: 0,
            };
            let f = Generator {
                id: 1,
                dimension: 1,
            };
            let m = Generator {
                id: 2,
                dimension: 2,
            };

            let fd = DiagramN::new(f, x, x).unwrap();
            let ffd = fd.attach(fd.clone(), Boundary::Target, &[]).unwrap();
            let md = DiagramN::new(m, ffd, fd).unwrap();

            let mut result = md.clone();

            for _ in 0..1 {
                result = result.attach(md.clone(), Boundary::Source, &[0]).unwrap();
            }

            result
        };

        use model::Action;
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
            Callback::from(|_| {})
        );

        let button_signature = self.view_sidebar_button(
            "Signature",
            "list",
            Callback::from(|_| {})
        );

        let button_user = self.view_sidebar_button(
            "User",
            "perm_identity",
            Callback::from(|_| {})
        );

        // TODO: Show onboarding info if workspace and signature is empty
        // TODO: Render 1d and 0d diagrams
        let workspace = match self.state.workspace() {
            Some(workspace) if workspace.visible_dimension() >= 2 => {
                // let diagram: DiagramN = workspace.visible_diagram().try_into().unwrap();
                // TODO: The workspace needs to be its own component
                html! {
                    <content class="workspace">
                        // <PanZoom class="workspace__panzoom">
                        //     <Diagram2D diagram={workspace.diagram.clone()} />
                        // </PanZoom>
                    </content>
                }
            }
            _ => {
                html! {
                    <content class="workspace workspace--empty">
                    </content>
                }
            }
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
                <SignatureView
                    signature={self.state.signature()}
                    dispatch={dispatch}
                />
                {workspace}
            </main>
        }
    }
}

impl App {
    fn view_sidebar_button(&self, label: &str, icon: &str, callback: Callback<MouseEvent>) -> Html {
        html! {
            <div class="sidebar__button tooltip" onclick={callback} data-tooltip={label}>
                <Icon name={icon} />
            </div>
        }
    }
}
