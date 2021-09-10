use yew::prelude::*;

use crate::{
    components::settings::{KeyStore, Settings},
    declare_settings,
};

declare_settings! {
    pub struct AppSettings {
        wireframe_3d: bool = false,
        orthographic_3d: bool = false,
        debug_normals: bool = false,
        debug_axes: bool = false,
        mesh_hidden: bool = false,
        subdivision_depth: u32 = 3,
    }
}

#[derive(Clone)]
pub enum SettingsMsg {
    ToggleWireframe,
    ToggleOrtho,
    ToggleDebugNormals,
    ToggleDebugAxes,
    ToggleMesh,
    SetSubdivisionDepth(u32),
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
                        "Debug wireframe",
                        |local| *local.get_wireframe_3d(),
                        self.link.callback(|_| SettingsMsg::ToggleWireframe),
                    )
                }
                {
                    self.view_checkbox(
                        "Orthographic projection",
                        |local| *local.get_orthographic_3d(),
                        self.link.callback(|_| SettingsMsg::ToggleOrtho),
                    )
                }
                {
                    self.view_checkbox(
                        "Hide mesh",
                        |local| *local.get_mesh_hidden(),
                        self.link.callback(|_| SettingsMsg::ToggleMesh),
                    )
                }
                {
                    self.view_checkbox(
                        "Debug normals",
                        |local| *local.get_debug_normals(),
                        self.link.callback(|_| SettingsMsg::ToggleDebugNormals),
                    )
                }
                {
                    self.view_checkbox(
                        "Debug axes",
                        |local| *local.get_debug_axes(),
                        self.link.callback(|_| SettingsMsg::ToggleDebugAxes),
                    )
                }
                {
                    self.view_slider(
                        "Subdivision depth",
                        |local| *local.get_subdivision_depth(),
                        0,
                        6,
                        &self.link.callback(SettingsMsg::SetSubdivisionDepth),
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
            Self::Message::ToggleMesh => {
                self.settings.set_mesh_hidden(!self.local.get_mesh_hidden());
            }
            Self::Message::ToggleDebugNormals => {
                self.settings
                    .set_debug_normals(!self.local.get_debug_normals());
            }
            Self::Message::ToggleDebugAxes => {
                self.settings.set_debug_axes(!self.local.get_debug_axes());
            }
            Self::Message::SetSubdivisionDepth(v) => {
                self.settings.set_subdivision_depth(v);
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
    fn view_checkbox<F>(&self, name: &str, getter: F, on_click: Callback<MouseEvent>) -> Html
    where
        F: Fn(&AppSettingsKeyStore) -> bool,
    {
        html! {
            <div class="settings__toggle-setting">
                <input
                    type="checkbox"
                    checked={getter(&self.local)}
                    onclick={on_click}
                />
                {name}
            </div>
        }
    }

    fn view_slider<F>(
        &self,
        name: &str,
        getter: F,
        min: u32,
        max: u32,
        on_change: &Callback<u32>,
    ) -> Html
    where
        F: Fn(&AppSettingsKeyStore) -> u32,
    {
        html! {
            <div class="settings__slider-setting">
                {name}
                <input
                    type="range"
                    min={min.to_string()}
                    max={max.to_string()}
                    value={getter(&self.local).to_string()}
                    onchange={on_change.reform(|c| {
                        if let ChangeData::Value(v) = c {
                            v.parse::<u32>().unwrap_or(0)
                        } else {
                            0
                        }
                    })}
                />
            </div>
        }
    }
}
