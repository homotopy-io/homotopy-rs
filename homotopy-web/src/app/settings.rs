use yew::agent::Dispatcher;
use yew::prelude::*;
use web_sys::Element;

use crate::declare_settings;
use crate::components::toast::{Toast, ToastAgent};
use crate::components::drawer::Drawer;
use crate::components::settings::{Settings, SettingsAgent};

declare_settings! {
    pub struct GlobalSettings {
        type Key = Global;
        type Message = GlobalMsg;

        example_toggle: bool,
    }
}

pub type AppSettings = SettingsAgent<GlobalSettings>;

pub enum SettingsMsg {
    Click,
    Setting(GlobalMsg),
}

#[derive(Properties, Clone, PartialEq)]
pub struct SettingsProps {}

pub struct SettingsView {
    link: ComponentLink<Self>,
    props: SettingsProps,
    toaster: Dispatcher<ToastAgent>,
    dispatcher: Dispatcher<AppSettings>,
    _settings: Box<dyn Bridge<AppSettings>>,
    toggle_ref: NodeRef,
}

impl Component for SettingsView {
    type Properties = SettingsProps;
    type Message = SettingsMsg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut settings = AppSettings::bridge(link.callback(SettingsMsg::Setting));
        settings.send(Settings::Subscribe(Global::example_toggle));

        Self {
            link,
            props,
            dispatcher: AppSettings::dispatcher(),
            _settings: settings,
            toaster: ToastAgent::dispatcher(),
            toggle_ref: NodeRef::default(),
        }
    }

    fn view(&self) -> Html {
        html! {
            <Drawer
                title="Settings"
                class="settings"
            >
                <input
                    type="checkbox"
                    ref=self.toggle_ref.clone()
                    onclick=self.link.callback(|e: MouseEvent| {
                        e.prevent_default();
                        SettingsMsg::Click
                    })
                />
            </Drawer>
        }
    }

    fn rendered(&mut self, _: bool) {}

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Self::Message::Setting(GlobalMsg::example_toggle(b)) => {
                self.set(b);
                if b {
                    self.toaster.send(Toast::success("Setting on"));
                } else {
                    self.toaster.send(Toast::error("Setting off"));
                }
            }
            Self::Message::Click => {
                self.toggle();
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        false
    }
}

impl SettingsView {
    fn toggle(&mut self) {
        let toggle = self.toggle_ref.cast::<Element>().unwrap();
        let updated = !toggle.has_attribute("checked");
        self.dispatcher
            .send(Settings::Update(GlobalMsg::example_toggle(updated)));
    }

    fn set(&mut self, b: bool) {
        let toggle = self.toggle_ref.cast::<Element>().unwrap();
        if b {
            toggle.set_attribute("checked", "").unwrap();
        } else {
            toggle.remove_attribute("checked").unwrap();
        }
    }
}
