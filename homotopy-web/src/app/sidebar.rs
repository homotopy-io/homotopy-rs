use wasm_bindgen::{closure::Closure, JsCast};
use yew::prelude::*;
use yew_macro::function_component;

use crate::{
    app::{attach::AttachView, keybindings::Keybindings},
    components::{
        icon::{Icon, IconSize},
        Visibility,
    },
    model::{self, proof, Proof},
};

mod buttons;
mod drawers;

#[derive(Properties, Clone, PartialEq)]
pub struct SidebarButtonProps {
    pub label: &'static str,
    pub icon: &'static str,
    pub action: SidebarMsg,
    #[prop_or_default]
    pub shortcut: Option<&'static str>,
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
            style={format!("{}", props.visibility)}
        >
            <Icon name={props.icon} size={IconSize::Icon24} />
        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct SidebarDrawerProps {
    pub class: &'static str,
    pub title: &'static str,
    pub dispatch: Callback<model::Action>,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub icon: Option<&'static str>,
    #[prop_or_default]
    pub on_click: Option<model::Action>,
}

#[function_component(SidebarDrawer)]
pub fn sidebar_drawer(props: &SidebarDrawerProps) -> Html {
    html! {
        <aside class={format!("{} drawer", props.class)}>
            <div class="drawer__header">
                <span class="drawer__title">
                    {props.title}
                </span>
                if let (Some(icon), Some(action)) = (props.icon, props.on_click.as_ref().cloned()) {
                    <span
                        class="drawer__icon"
                        onclick={props.dispatch.reform(move |_| action.clone())}
                    >
                        <Icon name={icon} size={IconSize::Icon18} />
                    </span>
                }
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

#[derive(Clone, PartialEq, Eq)]
pub enum SidebarMsg {
    Toggle(drawers::NavDrawer),
    Dispatch(model::Action),
}

#[derive(Default)]
pub struct Sidebar {
    open: Option<drawers::NavDrawer>,
    // Hold onto bindings so that they are dropped when the app is destroyed
    keybindings: Option<Closure<dyn FnMut(KeyboardEvent)>>,
}

impl Component for Sidebar {
    type Message = SidebarMsg;
    type Properties = SidebarProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut sidebar = Self::default();
        sidebar.install_keyboard_shortcuts(ctx);
        sidebar
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
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
                        ctx.link()
                            .send_message(SidebarMsg::Toggle(drawers::NavDrawer::DRAWER_SIGNATURE));
                    }
                }
                ctx.props().dispatch.emit(action);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <aside class="sidebar">
                    <a href="#about">
                        <img src="/logo.svg" alt="Homotopy.io logo" class="sidebar__logo" />
                    </a>
                    {self.nav(ctx)}
                    {self.tools(ctx)}
                </aside>
                {self.drawer(ctx)}
            </>
        }
    }
}

impl Sidebar {
    fn drawer(&self, ctx: &Context<Self>) -> Html {
        let dispatch = &ctx.props().dispatch;
        let proof = &ctx.props().proof;        

        if proof.image_export {
            return html! {
                <SidebarDrawer
                    // Re-using the attach class here, will give a new class later (perhaps)
                    class="attach"
                    title="Image export"
                    dispatch={dispatch}
                    icon="close"
                    on_click={model::Action::ExportImage}
                >
                </SidebarDrawer>
            };
        }

        let attach_options = proof.workspace().and_then(|workspace| workspace.attach.clone());

        if let Some(attach_options) = attach_options {
            return html! {
                <SidebarDrawer
                    class="attach"
                    title="Attach"
                    dispatch={dispatch}
                    icon="close"
                    on_click={model::Action::from(proof::Action::ClearAttach)}
                >
                    <AttachView
                        dispatch={dispatch.reform(model::Action::Proof)}
                        options={attach_options}
                        signature={ctx.props().proof.signature().clone()}
                    />
                </SidebarDrawer>
            };
        }

        self.open
            .map(|drawer| drawer.view(dispatch, &ctx.props().proof))
            .unwrap_or_default()
    }

    fn install_keyboard_shortcuts(&mut self, ctx: &Context<Self>) {
        let dispatch = ctx.link().callback(SidebarMsg::Dispatch);
        let keybindings = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            let key = event.key().to_ascii_lowercase();
            if let Some(action) = Keybindings::get_action(&key) {
                dispatch.emit(action);
            }
        }) as Box<dyn FnMut(_)>);

        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("keyup", keybindings.as_ref().unchecked_ref())
            .unwrap();

        self.keybindings = Some(keybindings);
    }
}
