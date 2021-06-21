use yew::agent::Dispatcher;
use yew::prelude::*;

use crate::components::drawer::Drawer;
use crate::components::toast::{Toast, ToastAgent};
use crate::declare_settings;

declare_settings! {
    pub struct AppSettings {
        example_toggle: bool,
    }
}

pub enum SettingsMsg {
    Click,
    Setting(AppSettingsMsg),
}

#[derive(Properties, Clone, PartialEq)]
pub struct SettingsProps {}

pub struct SettingsView {
    link: ComponentLink<Self>,
    props: SettingsProps,
    toaster: Dispatcher<ToastAgent>,
    settings: AppSettings,
    /// Example toggle setting
    example_toggle: bool,
}

impl Component for SettingsView {
    type Properties = SettingsProps;
    type Message = SettingsMsg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut settings = AppSettings::connect(link.callback(SettingsMsg::Setting));
        settings.subscribe(&[AppSettingsKey::example_toggle]);

        Self {
            link,
            props,
            toaster: ToastAgent::dispatcher(),
            settings,
            example_toggle: false,
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
                    checked=self.example_toggle
                    onclick=self.link.callback(|_| SettingsMsg::Click)
                />
            </Drawer>
        }
    }

    fn rendered(&mut self, _: bool) {}

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Self::Message::Setting(AppSettingsMsg::example_toggle(b)) => {
                self.example_toggle = b;
                if b {
                    self.toaster.send(Toast::success("Setting on"));
                } else {
                    self.toaster.send(Toast::error("Setting off"));
                }
            }
            Self::Message::Click => {
                self.example_toggle = !self.example_toggle;
                self.settings.set_example_toggle(self.example_toggle);
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        false
    }
}
