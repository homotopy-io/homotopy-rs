use yew::agent::Dispatcher;
use yew::prelude::*;
use yew_functional::function_component;

use crate::components::drawer::Drawer;
use crate::components::settings::{Settings, SettingsAgent};

use super::{AppSettings, Setting, SettingPayload};

#[derive(Properties, Clone, PartialEq)]
pub struct SettingsProps {}

pub struct SettingsView {
    props: SettingsProps,
    settings: Dispatcher<AppSettings>,
}

impl Component for SettingsView {
    type Properties = SettingsProps;
    type Message = ();

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {
            props,
            settings: AppSettings::dispatcher(),
        }
    }

    fn view(&self) -> Html {
        html! {
            <Drawer title="Settings" class="settings">
                {"Hello, World!"}
            </Drawer>
        }
    }

    fn rendered(&mut self, _: bool) {}

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        false
    }
}

