use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{
    components::settings::{KeyStore, Settings},
    declare_settings,
};

declare_settings! {
    pub struct AppSettings {
        animated_3d: bool = false,
        cubical_subdivision: bool = true,
        dpr_scale: bool = true,
        smooth_time: bool = true,
        subdivision_depth: u32 = 2,
        geometry_samples: u32 = 10,

        show_previews: bool = true,
        orthographic_3d: bool = false,
        specularity: u32 = 25,
        shininess: u32 = 64,
        gamma: u32 = 22,
        animate_singularities: bool = true,
        singularity_duration: u32 = 5,
        geometry_scale: u32 = 10,

        wireframe_3d: bool = false,
        mesh_hidden: bool = false,
        debug_normals: bool = false,
        disable_lighting: bool = false,
        debug_axes: bool = false,
    }
}

#[derive(Properties, Clone, PartialEq, Eq)]
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
            <div class="settings">
                <h3>{"General"}</h3>
                <div class="settings__segment">
                    {
                        self.view_checkbox(
                            "Show previews in signature",
                            |local| *local.get_show_previews(),
                            AppSettingsDispatch::set_show_previews,
                        )
                    }
                    {
                        self.view_checkbox(
                            "Render 3D diagrams as movies",
                            |local| *local.get_animated_3d(),
                            AppSettingsDispatch::set_animated_3d,
                        )
                    }
                </div>
                <h3>{"3D renderer"}</h3>
                <div class="settings__segment">
                    <h4>{"Quality"}</h4>
                    {
                        self.view_checkbox(
                            "Cubical subdivision",
                            |local| *local.get_cubical_subdivision(),
                            AppSettingsDispatch::set_cubical_subdivision,
                        )
                    }
                    {
                        self.view_checkbox(
                            "Scale by device pixel ratio",
                            |local| *local.get_dpr_scale(),
                            AppSettingsDispatch::set_dpr_scale,
                        )
                    }
                    {
                        self.view_checkbox(
                            "Smooth in the time axis",
                            |local| *local.get_smooth_time(),
                            AppSettingsDispatch::set_smooth_time,
                        )
                    }
                    {
                        self.view_slider(
                            "Subdivision depth",
                            |local| *local.get_subdivision_depth(),
                            AppSettingsDispatch::set_subdivision_depth,
                            0,
                            6,
                        )
                    }
                    {
                        self.view_slider(
                            "Geometry samples",
                            |local| *local.get_geometry_samples(),
                            AppSettingsDispatch::set_geometry_samples,
                            3,
                            15,
                        )
                    }
                </div>
                <div class="settings__segment">
                    <h4>{"Style"}</h4>
                    {
                        self.view_checkbox(
                            "Orthographic projection",
                            |local| *local.get_orthographic_3d(),
                            AppSettingsDispatch::set_orthographic_3d,
                        )
                    }
                    {
                        self.view_slider(
                            "Specularity",
                            |local| *local.get_specularity(),
                            AppSettingsDispatch::set_specularity,
                            0,
                            100,
                        )
                    }
                    {
                        self.view_slider(
                            "Shininess",
                            |local| *local.get_shininess(),
                            AppSettingsDispatch::set_shininess,
                            20,
                            80,
                        )
                    }
                    {
                        self.view_checkbox(
                            "Animate singularities",
                            |local| *local.get_animate_singularities(),
                            AppSettingsDispatch::set_animate_singularities,
                        )
                    }
                    {
                        self.view_slider(
                            "Singularity duration",
                            |local| *local.get_singularity_duration(),
                            AppSettingsDispatch::set_singularity_duration,
                            1,
                            9,
                        )
                    }
                    {
                        self.view_slider(
                            "4D geometry scale",
                            |local| *local.get_geometry_scale(),
                            AppSettingsDispatch::set_geometry_scale,
                            5,
                            20,
                        )
                    }
                </div>
                <div class="settings__segment">
                    <h4>{"Debugging"}</h4>
                    {
                        self.view_checkbox(
                            "Debug wireframe",
                            |local| *local.get_wireframe_3d(),
                            AppSettingsDispatch::set_wireframe_3d,
                        )
                    }
                    {
                        self.view_checkbox(
                            "Hide mesh",
                            |local| *local.get_mesh_hidden(),
                            AppSettingsDispatch::set_mesh_hidden,
                        )
                    }
                    {
                        self.view_checkbox(
                            "Debug normals",
                            |local| *local.get_debug_normals(),
                            AppSettingsDispatch::set_debug_normals,
                        )
                    }
                    {
                        self.view_checkbox(
                            "Disable lighting",
                            |local| *local.get_disable_lighting(),
                            AppSettingsDispatch::set_disable_lighting,
                        )
                    }
                    {
                        self.view_checkbox(
                            "Debug axes",
                            |local| *local.get_debug_axes(),
                            AppSettingsDispatch::set_debug_axes,
                        )
                    }
                </div>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.local.set(&msg);
        true
    }
}

impl SettingsView {
    fn view_checkbox<G, S>(&self, name: &str, getter: G, setter: S) -> Html
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
                    onclick={Callback::from(move |_| setter(&dispatch, !checked))}
                />
                {name}
            </div>
        }
    }

    fn view_slider<G, S>(&self, name: &str, getter: G, setter: S, min: u32, max: u32) -> Html
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
