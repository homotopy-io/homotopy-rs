use yew::prelude::*;

use crate::components::settings::KeyStore;
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
    settings: AppSettings,
    // Maintain a local copy of the global app settings in order to display the current settings
    // state correctly.
    local: AppSettingsKeyStore,
}

impl Component for SettingsView {
    type Properties = SettingsProps;
    type Message = SettingsMsg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut settings = AppSettings::connect(link.callback(SettingsMsg::Setting));
        // So that we can keep our local copy of the global settings up to date,
        // we're going to need to subscribe to all changes in the global settings state.
        settings.subscribe(AppSettings::ALL);

        Self {
            link,
            props,
            settings,
            local: Default::default(),
        }
    }

    fn view(&self) -> Html {
        html! {
            <input
                type="checkbox"
                checked={*self.local.get_example_toggle()}
                onclick={self.link.callback(|_| SettingsMsg::Click)}
            />
        }
    }

    fn rendered(&mut self, _: bool) {}

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            // If we're notified about a setting change, just thread that through
            // to our local working copy.
            Self::Message::Setting(msg) => self.local.set(&msg),
            Self::Message::Click => {
                self.settings
                    .set_example_toggle(!self.local.get_example_toggle());
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        false
    }
}
