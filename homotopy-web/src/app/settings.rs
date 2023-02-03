use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{components::delta::CallbackIdx, declare_settings};

declare_settings! {
    pub struct AppSettings {
        show_previews: bool = true,
        weak_units: bool = false,
        animated_3d: bool = false,

        cubical_subdivision: bool = true,
        dpr_scale: bool = true,
        smooth_time: bool = true,
        subdivision_depth: u32 = 2,
        geometry_samples: u32 = 10,

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
    callback_idxs: Vec<CallbackIdx>,
}

impl Component for SettingsView {
    type Message = AppSettingsMsg;
    type Properties = SettingsProps;

    fn create(ctx: &Context<Self>) -> Self {
        let callback_idxs = AppSettings::subscribe(AppSettings::ALL, ctx.link().callback(|x| x));

        Self { callback_idxs }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        AppSettings::unsubscribe(AppSettings::ALL, &self.callback_idxs);
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="settings">
                <h3>{"General"}</h3>
                <div class="settings__segment">
                    {
                        Self::view_checkbox(
                            "Show previews in signature",
                            AppSettings::get_show_previews(),
                            AppSettings::set_show_previews,
                        )
                    }
                    {
                        Self::view_checkbox(
                            "Allow attaching weak units",
                            AppSettings::get_weak_units(),
                            AppSettings::set_weak_units,
                        )
                    }
                    {
                        Self::view_checkbox(
                            "Render 3D diagrams as movies",
                            AppSettings::get_animated_3d(),
                            AppSettings::set_animated_3d,
                        )
                    }
                </div>
                <h3>{"3D renderer"}</h3>
                <div class="settings__segment">
                    <h4>{"Quality"}</h4>
                    {
                        Self::view_checkbox(
                            "Cubical subdivision",
                            AppSettings::get_cubical_subdivision(),
                            AppSettings::set_cubical_subdivision,
                        )
                    }
                    {
                        Self::view_checkbox(
                            "Scale by device pixel ratio",
                            AppSettings::get_dpr_scale(),
                            AppSettings::set_dpr_scale,
                        )
                    }
                    {
                        Self::view_checkbox(
                            "Smooth in the time axis",
                            AppSettings::get_smooth_time(),
                            AppSettings::set_smooth_time,
                        )
                    }
                    {
                        Self::view_slider(
                            "Subdivision depth",
                            AppSettings::get_subdivision_depth(),
                            AppSettings::set_subdivision_depth,
                            0,
                            6,
                        )
                    }
                    {
                        Self::view_slider(
                            "Geometry samples",
                            AppSettings::get_geometry_samples(),
                            AppSettings::set_geometry_samples,
                            3,
                            15,
                        )
                    }
                </div>
                <div class="settings__segment">
                    <h4>{"Style"}</h4>
                    {
                        Self::view_checkbox(
                            "Orthographic projection",
                            AppSettings::get_orthographic_3d(),
                            AppSettings::set_orthographic_3d,
                        )
                    }
                    {
                        Self::view_slider(
                            "Specularity",
                            AppSettings::get_specularity(),
                            AppSettings::set_specularity,
                            0,
                            100,
                        )
                    }
                    {
                        Self::view_slider(
                            "Shininess",
                            AppSettings::get_shininess(),
                            AppSettings::set_shininess,
                            20,
                            80,
                        )
                    }
                    {
                        Self::view_checkbox(
                            "Animate singularities",
                            AppSettings::get_animate_singularities(),
                            AppSettings::set_animate_singularities,
                        )
                    }
                    {
                        Self::view_slider(
                            "Singularity duration",
                            AppSettings::get_singularity_duration(),
                            AppSettings::set_singularity_duration,
                            1,
                            9,
                        )
                    }
                    {
                        Self::view_slider(
                            "4D geometry scale",
                            AppSettings::get_geometry_scale(),
                            AppSettings::set_geometry_scale,
                            5,
                            20,
                        )
                    }
                </div>
                <div class="settings__segment">
                    <h4>{"Debugging"}</h4>
                    {
                        Self::view_checkbox(
                            "Debug wireframe",
                            AppSettings::get_wireframe_3d(),
                            AppSettings::set_wireframe_3d,
                        )
                    }
                    {
                        Self::view_checkbox(
                            "Hide mesh",
                            AppSettings::get_mesh_hidden(),
                            AppSettings::set_mesh_hidden,
                        )
                    }
                    {
                        Self::view_checkbox(
                            "Debug normals",
                            AppSettings::get_debug_normals(),
                            AppSettings::set_debug_normals,
                        )
                    }
                    {
                        Self::view_checkbox(
                            "Disable lighting",
                            AppSettings::get_disable_lighting(),
                            AppSettings::set_disable_lighting,
                        )
                    }
                    {
                        Self::view_checkbox(
                            "Debug axes",
                            AppSettings::get_debug_axes(),
                            AppSettings::set_debug_axes,
                        )
                    }
                </div>
            </div>
        }
    }
}

impl SettingsView {
    fn view_checkbox<S>(name: &str, current: bool, setter: S) -> Html
    where
        S: Fn(bool) + 'static,
    {
        html! {
            <div class="settings__toggle-setting">
                <input
                    type="checkbox"
                    checked={current}
                    onclick={Callback::from(move |_| setter(!current))}
                />
                {name}
            </div>
        }
    }

    fn view_slider<S>(name: &str, current: u32, setter: S, min: u32, max: u32) -> Html
    where
        S: Fn(u32) + 'static,
    {
        html! {
            <div class="settings__slider-setting">
                {name}
                <input
                    type="range"
                    min={min.to_string()}
                    max={max.to_string()}
                    value={current.to_string()}
                    onchange={Callback::from(move |e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        let updated = input.value().parse::<u32>().unwrap_or(0);
                        setter(updated);
                    })}
                />
            </div>
        }
    }
}
