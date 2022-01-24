use web_sys::HtmlInputElement;
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
        disable_lighting: bool = false,
        debug_axes: bool = false,
        mesh_hidden: bool = false,
        subdivision_depth: u32 = 3,
        geometry_samples: u32 = 10,
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct SettingsProps {}

pub struct SettingsView {
    _settings: AppSettings,
    // Maintain a local copy of the global app settings in order to display the current settings
    // state correctly.
    local: AppSettingsKeyStore,
}

impl Component for SettingsView {
    type Message = AppSettingsMsg;
    type Properties = SettingsProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut settings = AppSettings::connect(ctx.link().callback(|x| x));
        // So that we can keep our local copy of the global settings up to date,
        // we're going to need to subscribe to all changes in the global settings state.
        settings.subscribe(AppSettings::ALL);

        Self {
            _settings: settings,
            local: Default::default(),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="settings__segment">
                <h4>{"3D renderer"}</h4>
                {
                    self.view_checkbox(
                        "Debug wireframe",
                        |local| *local.get_wireframe_3d(),
                        |dispatch, checked| dispatch.set_wireframe_3d(!checked),
                    )
                }
                {
                    self.view_checkbox(
                        "Orthographic projection",
                        |local| *local.get_orthographic_3d(),
                        |dispatch, checked| dispatch.set_orthographic_3d(!checked),
                    )
                }
                {
                    self.view_checkbox(
                        "Hide mesh",
                        |local| *local.get_mesh_hidden(),
                        |dispatch, checked| dispatch.set_mesh_hidden(!checked),
                    )
                }
                {
                    self.view_checkbox(
                        "Debug normals",
                        |local| *local.get_debug_normals(),
                        |dispatch, checked| dispatch.set_debug_normals(!checked),
                    )
                }
                {
                    self.view_checkbox(
                        "Disable lighting",
                        |local| *local.get_disable_lighting(),
                        |dispatch, checked| dispatch.set_disable_lighting(!checked),
                    )
                }
                {
                    self.view_checkbox(
                        "Debug axes",
                        |local| *local.get_debug_axes(),
                        |dispatch, checked| dispatch.set_debug_axes(!checked),
                    )
                }
                {
                    self.view_slider(
                        "Subdivision depth",
                        |local| *local.get_subdivision_depth(),
                        |dispatch, depth| dispatch.set_subdivision_depth(depth),
                        0,
                        6,
                    )
                }
                {
                    self.view_slider(
                        "Geometry samples",
                        |local| *local.get_geometry_samples(),
                        |dispatch, samples| dispatch.set_geometry_samples(samples),
                        3,
                        15,
                    )
                }
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.local.set(&msg);
        true
    }
}

impl SettingsView {
    fn view_checkbox<G, S>(
        &self,
        name: &str,
        getter: G,
        setter: S,
    ) -> Html
    where
        G: Fn(&AppSettingsKeyStore) -> bool,
        S: Fn(&AppSettingsDispatch, bool) + 'static,
    {
        let checked = getter(&self.local);
        let dispatch = AppSettingsDispatch::new();

        html! {
            <div class="settings__toggle-setting">
                <input
                    type="checkbox"
                    checked={checked}
                    onclick={Callback::from(move |_| setter(&dispatch, checked))}
                />
                {name}
            </div>
        }
    }

    fn view_slider<G, S>(
        &self,
        name: &str,
        getter: G,
        setter: S,
        min: u32,
        max: u32,
    ) -> Html
    where
        G: Fn(&AppSettingsKeyStore) -> u32,
        S: Fn(&AppSettingsDispatch, u32) + 'static,
    {
        let dispatch = AppSettingsDispatch::new();

        html! {
            <div class="settings__slider-setting">
                {name}
                <input
                    type="range"
                    min={min.to_string()}
                    max={max.to_string()}
                    value={getter(&self.local).to_string()}
                    onchange={Callback::from(move |e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        let updated = input.value().parse::<u32>().unwrap_or(0);
                        setter(&dispatch, updated);
                    })}
                />
            </div>
        }
    }
}
