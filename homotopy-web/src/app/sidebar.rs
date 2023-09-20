use serde::{Deserialize, Serialize};
use wasm_bindgen::{closure::Closure, JsCast};
use yew::prelude::*;
use yew_macro::function_component;

use crate::{
    app::{account::RemoteProjectMetadata, attach::AttachView, keybindings::Keybindings},
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DrawerViewSize {
    TemporarilyHidden,
    Regular,
    Expanded,
}

impl Default for DrawerViewSize {
    fn default() -> Self {
        Self::Regular
    }
}

// from pixel width of drawer to size
impl From<i32> for DrawerViewSize {
    fn from(px: i32) -> Self {
        if px < 100 {
            Self::TemporarilyHidden
        } else if px < 300 {
            Self::Regular
        } else {
            Self::Expanded
        }
    }
}

impl DrawerViewSize {
    // Some drawer view sizes (eg. compact) will snap the drawer to a certain width (px).
    fn snap_width(self) -> Option<i32> {
        match self {
            Self::TemporarilyHidden => Some(0),
            Self::Regular | Self::Expanded => None,
        }
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct SidebarDrawerProps {
    pub class: &'static str,
    pub title: &'static str,
    pub model_dispatch: Callback<model::Action>,
    pub sidebar_dispatch: Callback<SidebarMsg>,
    pub remote_project_metadata: Option<RemoteProjectMetadata>,
    pub initial_width: i32,
    #[prop_or(0)]
    pub min_width: i32,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub icon: Option<&'static str>,
    #[prop_or_default]
    pub on_click: Option<model::Action>,
}

// Messages corresponding to mouse events on resize bar (right border of drawer)
pub enum SidebarDrawerMsg {
    ResizeStart,
    Resize(i32),
    ResizeDone,
}

pub struct SidebarDrawer {
    width: i32,
    drawer_view_size: DrawerViewSize,
    resize_closure: Closure<dyn FnMut(MouseEvent)>,
    resize_done_closure: Closure<dyn FnMut(MouseEvent)>,
}

impl SidebarDrawer {
    const MAX_WIDTH: i32 = 400; // px
    const DEFAULT_WIDTH: i32 = 250; // px
    const RESIZE_OFFSET: i32 = 2; // px, useful for recentering cursor while mouse button held
}

impl Component for SidebarDrawer {
    type Message = SidebarDrawerMsg;
    type Properties = SidebarDrawerProps;

    fn create(ctx: &Context<Self>) -> Self {
        let resize_msg = ctx.link().callback(SidebarDrawerMsg::Resize);
        let done_msg = ctx.link().callback(|()| SidebarDrawerMsg::ResizeDone);

        let mouse_move_handler = move |e: MouseEvent| {
            let width = e.client_x() - Sidebar::WIDTH + Self::RESIZE_OFFSET;
            resize_msg.emit(width);
        };
        let mouse_up_handler = move |_| {
            done_msg.emit(());
        };

        let resize_closure = Closure::wrap(Box::new(mouse_move_handler) as Box<dyn FnMut(_)>);
        let resize_done_closure = Closure::wrap(Box::new(mouse_up_handler) as Box<dyn FnMut(_)>);

        let drawer_view_size = DrawerViewSize::from(ctx.props().initial_width);

        Self {
            width: ctx.props().initial_width,
            drawer_view_size,
            resize_closure,
            resize_done_closure,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use SidebarDrawerMsg::{Resize, ResizeDone, ResizeStart};

        match msg {
            ResizeStart => {
                let window = web_sys::window().unwrap();
                window
                    .add_event_listener_with_callback(
                        "mousemove",
                        self.resize_closure.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                window
                    .add_event_listener_with_callback(
                        "mouseup",
                        self.resize_done_closure.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                false
            }
            Resize(width) => {
                let new_dvs = DrawerViewSize::from(width);
                if self.drawer_view_size != new_dvs {
                    ctx.props()
                        .sidebar_dispatch
                        .emit(SidebarMsg::ResizeDrawerView(new_dvs));
                    self.drawer_view_size = new_dvs;
                }

                if let Some(snap_width) = self.drawer_view_size.snap_width() {
                    self.width = snap_width;
                } else {
                    self.width = width.min(Self::MAX_WIDTH);
                }
                true
            }
            ResizeDone => {
                let window = web_sys::window().unwrap();
                window
                    .remove_event_listener_with_callback(
                        "mousemove",
                        self.resize_closure.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                window
                    .remove_event_listener_with_callback(
                        "mouseup",
                        self.resize_done_closure.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                if self.drawer_view_size == DrawerViewSize::TemporarilyHidden {
                    ctx.props().sidebar_dispatch.emit(SidebarMsg::Toggle(None));
                } else {
                    ctx.props()
                        .sidebar_dispatch
                        .emit(SidebarMsg::SaveDrawerWidth(self.width));
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let width = if self.width > 0 {
            self.width.max(ctx.props().min_width)
        } else {
            0
        };
        let size_class = match self.drawer_view_size {
            DrawerViewSize::TemporarilyHidden => "temporarily-hidden",
            DrawerViewSize::Regular => "regular",
            DrawerViewSize::Expanded => "expanded",
        };

        html! {
            <aside
                class={format!("{} drawer drawer-{size_class}", ctx.props().class)}
                style={format!("width: {width}px;")}
                >
                <div class="drawer__inner">
                    <div class="drawer__header">
                        <span class="drawer__title">
                            {ctx.props().title}
                        </span>
                        if let (Some(icon), Some(action)) = (ctx.props().icon, ctx.props().on_click.as_ref().cloned()) {
                            <span
                                class="drawer__icon"
                                onclick={ctx.props().model_dispatch.reform(move |_| action.clone())}
                            >
                                <Icon name={icon} size={IconSize::Icon18} />
                            </span>
                        }
                    </div>
                    <div class="drawer__content">
                        { for ctx.props().children.iter() }
                    </div>
                </div>
                <div
                    class="drawer__resize-bar"
                    draggable="false"
                    onmousedown={ctx.link().callback(|_| SidebarDrawerMsg::ResizeStart)}
                />
            </aside>
        }
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct SidebarProps {
    pub proof: Proof,
    pub options: Option<model::Selectables>,
    pub dispatch: Callback<model::Action>,
    pub remote_project_metadata: Option<RemoteProjectMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidebarMsg {
    Dispatch(model::Action),
    SaveDrawerWidth(i32),
    ResizeDrawerView(DrawerViewSize),
    Toggle(Option<drawers::NavDrawer>),
}

pub struct Sidebar {
    last_drawer_width: i32,
    drawer_view_size: DrawerViewSize,
    open: Option<drawers::NavDrawer>,
    // Hold onto bindings so that they are dropped when the app is destroyed
    keybindings: Option<Closure<dyn FnMut(KeyboardEvent)>>,
}

impl Default for Sidebar {
    fn default() -> Self {
        Sidebar {
            last_drawer_width: SidebarDrawer::DEFAULT_WIDTH,
            drawer_view_size: DrawerViewSize::from(SidebarDrawer::DEFAULT_WIDTH),
            open: Default::default(),
            keybindings: Default::default(),
        }
    }
}

impl Component for Sidebar {
    type Message = SidebarMsg;
    type Properties = SidebarProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut sidebar = Sidebar::default();
        sidebar.install_keyboard_shortcuts(ctx);
        sidebar
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SidebarMsg::SaveDrawerWidth(width) => {
                self.last_drawer_width = width;
                self.drawer_view_size = width.into();
                false
            }
            SidebarMsg::ResizeDrawerView(size) => {
                self.drawer_view_size = size;
                true
            }
            SidebarMsg::Toggle(None) => {
                self.drawer_view_size = self.last_drawer_width.into();
                self.open = None;
                true
            }
            SidebarMsg::Toggle(drawer) if drawer == self.open => {
                self.open = None;
                true
            }
            SidebarMsg::Toggle(drawer) => {
                self.open = drawer;
                true
            }
            SidebarMsg::Dispatch(action) => {
                if action == model::Action::Proof(proof::Action::CreateGeneratorZero)
                    && !matches!(self.open, Some(drawers::NavDrawer::DRAWER_SIGNATURE))
                {
                    ctx.link().send_message(SidebarMsg::Toggle(Some(
                        drawers::NavDrawer::DRAWER_SIGNATURE,
                    )));
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
    const WIDTH: i32 = 48; // px

    #[allow(clippy::let_underscore_untyped)]
    fn drawer(&self, ctx: &Context<Self>) -> Html {
        let model_dispatch = &ctx.props().dispatch;
        let sidebar_dispatch = ctx.link().callback(|x| x);

        if let Some(options) = ctx.props().options.as_ref() {
            return html! {
                <SidebarDrawer
                    class="attach"
                    title={options.name()}
                    model_dispatch={model_dispatch}
                    sidebar_dispatch={sidebar_dispatch}
                    initial_width={self.last_drawer_width}
                    icon="close"
                    on_click={model::Action::ClearSelections}
                >
                    <AttachView
                        dispatch={model_dispatch.clone()}
                        options={options.clone()}
                        signature={ctx.props().proof.signature.clone()}
                    />
                </SidebarDrawer>
            };
        }

        self.open
            .map(|drawer| {
                drawer.view(
                    model_dispatch,
                    &sidebar_dispatch,
                    &ctx.props().proof,
                    &ctx.props().remote_project_metadata,
                    self.last_drawer_width,
                    self.drawer_view_size,
                )
            })
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
