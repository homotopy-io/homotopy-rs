use yew::prelude::*;

use crate::{
    components::settings::{KeyStore, Settings},
    declare_settings,
};

declare_settings! {
    pub struct AppSettings {
        wireframe_3d: bool,
        orthographic_3d: bool,
        lighting_disable: bool,
        subdivision_depth: usize,
    }
}

#[derive(Clone)]
pub enum SettingsMsg {
    ToggleWireframe,
    ToggleOrtho,
    ToggleLighting,
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
    type Message = SettingsMsg;
    type Properties = SettingsProps;

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
            <div class="settings__segment">
                <h4>{"3D renderer"}</h4>
                {
                    self.view_checkbox(
                        "Wireframe 3D",
                        |local| *local.get_wireframe_3d(),
                        &SettingsMsg::ToggleWireframe,
                    )
                }
                {
                    self.view_checkbox(
                        "Orthographic perspective",
                        |local| *local.get_orthographic_3d(),
                        &SettingsMsg::ToggleOrtho,
                    )
                }
                {
                    self.view_checkbox(
                        "Disable lighting",
                        |local| *local.get_lighting_disable(),
                        &SettingsMsg::ToggleLighting,
                    )
                }
            </div>
        }
    }

    fn rendered(&mut self, _: bool) {}

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            // If we're notified about a setting change, just thread that through
            // to our local working copy.
            Self::Message::Setting(msg) => self.local.set(&msg),
            Self::Message::ToggleWireframe => {
                self.settings
                    .set_wireframe_3d(!self.local.get_wireframe_3d());
            }
            Self::Message::ToggleOrtho => {
                self.settings
                    .set_orthographic_3d(!self.local.get_orthographic_3d());
            }
            Self::Message::ToggleLighting => {
                self.settings
                    .set_lighting_disable(!self.local.get_lighting_disable());
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
    fn view_checkbox<F>(&self, name: &str, getter: F, on_click: &'static SettingsMsg) -> Html
    where
        F: Fn(&AppSettingsKeyStore) -> bool,
    {
        html! {
            <div class="settings__toggle-setting">
                <input
                    type="checkbox"
                    checked={getter(&self.local)}
                    onclick={self.link.callback(move |_| on_click.clone())}
                />
                {name}
            </div>
        }
    }
}
