use yew::prelude::*;
use yew_functional::function_component;

use wasm_bindgen::closure::Closure;

use wasm_bindgen::JsCast;

use crate::app::attach::AttachView;
use crate::components::icon::{Icon, IconSize};
use crate::components::Visibility;
use crate::model::{self, proof, Proof};

mod buttons;
mod drawers;

pub use buttons::TOOL_BUTTONS;

#[derive(Properties, Clone, PartialEq)]
pub struct SidebarButtonProps {
    pub label: &'static str,
    pub icon: &'static str,
    pub action: SidebarMsg,
    #[prop_or_default]
    pub shortcut: Option<char>,
    pub dispatch: Callback<SidebarMsg>,
    #[prop_or(Visibility::Visible)]
    pub visibility: Visibility,
}

#[function_component(SidebarButton)]
pub fn sidebar_button(props: &SidebarButtonProps) -> Html {
    let action = props.action.clone();

    html! {
        <div
            class="sidebar__button tooltip tooltip--right"
            onclick={props.dispatch.reform(move |_| action.clone())}
            data-tooltip={
                if let Some(shortcut) = props.shortcut {
                    format!("{} ({})", props.label, shortcut.to_uppercase())
                } else {
                    props.label.to_owned()
                }
            }
            style={&format!("{}", props.visibility)}
        >
            <Icon name={props.icon} size={IconSize::Icon24} />
        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct SidebarDrawerProps {
    pub class: &'static str,
    pub title: &'static str,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(SidebarDrawer)]
pub fn sidebar_drawer(props: &SidebarDrawerProps) -> Html {
    html! {
        <aside class={format!("{} drawer", props.class)}>
            <div class="drawer__header">
                <span class="drawer__title">
                    {props.title}
                </span>
            </div>
            <div class="drawer__content">
                { for props.children.iter() }
            </div>
        </aside>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct SidebarProps {
    pub proof: Proof,
    pub dispatch: Callback<model::Action>,
}

#[derive(Clone, PartialEq)]
pub enum SidebarMsg {
    Toggle(drawers::NavDrawer),
    Dispatch(model::Action),
}

pub struct Sidebar {
    props: SidebarProps,
    link: ComponentLink<Self>,
    open: Option<drawers::NavDrawer>,
    // Hold onto bindings so that they are dropped when the app is destroyed
    bindings: Option<Closure<dyn FnMut(KeyboardEvent)>>,
}

impl Component for Sidebar {
    type Properties = SidebarProps;
    type Message = SidebarMsg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut sidebar = Self {
            props,
            link,
            open: None,
            bindings: None,
        };
        sidebar.install_keyboard_shortcuts();
        sidebar
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
                if let model::Action::Proof(proof::Action::CreateGeneratorZero) = action {
                    if self.open.is_none() {
                        self.link
                            .send_message(SidebarMsg::Toggle(drawers::NavDrawer::DRAWER_SIGNATURE));
                    }
                }
                self.props.dispatch.emit(action);
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props == props {
            false
        } else {
            self.props = props;
            true
        }
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
                <SidebarDrawer class="attach" title="Attach">
                    <AttachView
                        dispatch={dispatch.reform(model::Action::Proof)}
                        options={attach_options}
                        signature={self.props.proof.signature()}
                    />
                </SidebarDrawer>
            };
        }

        self.open
            .map(|drawer| drawer.view(dispatch, &self.props.proof))
            .unwrap_or_default()
    }

    fn install_keyboard_shortcuts(&mut self) {
        use homotopy_core::Direction;

        let dispatch = self.link.callback(SidebarMsg::Dispatch);
        let bindings = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            let key = event.key().to_ascii_lowercase();
            let button = TOOL_BUTTONS.iter().find(|button| match button.shortcut() {
                Some(shortcut) => shortcut.to_string() == key,
                None => false,
            });

            if let Some(button) = button {
                dispatch.emit(button.action());
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
            .add_event_listener_with_callback("keyup", bindings.as_ref().unchecked_ref())
            .unwrap();

        self.bindings = Some(bindings);
    }
}
