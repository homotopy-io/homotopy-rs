use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::declare_settings;

declare_settings! {
    pub struct AppSettings {
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

#[derive(Properties, Clone, PartialEq)]
pub struct SettingsProps {
    pub settings: AppSettingsDispatch,
}

pub struct SettingsView {}

impl Component for SettingsView {
    type Message = AppSettingsMsg;
    type Properties = SettingsProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="settings">
                <h3>{"General"}</h3>
                <div class="settings__segment">
                    {
                        SettingsView::view_checkbox(
                            ctx,
                            "Show previews in signature",
                            |local| *local.get_show_previews(),
                            AppSettingsDispatch::set_show_previews,
                        )
                    }
                </div>
                <h3>{"3D renderer"}</h3>
                <div class="settings__segment">
                    <h4>{"Quality"}</h4>
                    {
                        SettingsView::view_checkbox(
                            ctx,
                            "Cubical subdivision",
                            |local| *local.get_cubical_subdivision(),
                            AppSettingsDispatch::set_cubical_subdivision,
                        )
                    }
                    {
                        SettingsView::view_checkbox(
                            ctx,
                            "Scale by device pixel ratio",
                            |local| *local.get_dpr_scale(),
                            AppSettingsDispatch::set_dpr_scale,
                        )
                    }
                    {
                        SettingsView::view_checkbox(
                            ctx,
                            "Smooth in the time axis",
                            |local| *local.get_smooth_time(),
                            AppSettingsDispatch::set_smooth_time,
                        )
                    }
                    {
                        SettingsView::view_slider(
                            ctx,
                            "Subdivision depth",
                            |local| *local.get_subdivision_depth(),
                            AppSettingsDispatch::set_subdivision_depth,
                            0,
                            6,
                        )
                    }
                    {
                        SettingsView::view_slider(
                            ctx,
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
                        SettingsView::view_checkbox(
                            ctx,
                            "Orthographic projection",
                            |local| *local.get_orthographic_3d(),
                            AppSettingsDispatch::set_orthographic_3d,
                        )
                    }
                    {
                        SettingsView::view_slider(
                            ctx,
                            "Specularity",
                            |local| *local.get_specularity(),
                            AppSettingsDispatch::set_specularity,
                            0,
                            100,
                        )
                    }
                    {
                        SettingsView::view_slider(
                            ctx,
                            "Shininess",
                            |local| *local.get_shininess(),
                            AppSettingsDispatch::set_shininess,
                            20,
                            80,
                        )
                    }
                    {
                        SettingsView::view_checkbox(
                            ctx,
                            "Animate singularities",
                            |local| *local.get_animate_singularities(),
                            AppSettingsDispatch::set_animate_singularities,
                        )
                    }
                    {
                        SettingsView::view_slider(
                            ctx,
                            "Singularity duration",
                            |local| *local.get_singularity_duration(),
                            AppSettingsDispatch::set_singularity_duration,
                            1,
                            9,
                        )
                    }
                    {
                        SettingsView::view_slider(
                            ctx,
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
                        SettingsView::view_checkbox(
                            ctx,
                            "Debug wireframe",
                            |local| *local.get_wireframe_3d(),
                            AppSettingsDispatch::set_wireframe_3d,
                        )
                    }
                    {
                        SettingsView::view_checkbox(
                            ctx,
                            "Hide mesh",
                            |local| *local.get_mesh_hidden(),
                            AppSettingsDispatch::set_mesh_hidden,
                        )
                    }
                    {
                        SettingsView::view_checkbox(
                            ctx,
                            "Debug normals",
                            |local| *local.get_debug_normals(),
                            AppSettingsDispatch::set_debug_normals,
                        )
                    }
                    {
                        SettingsView::view_checkbox(
                            ctx,
                            "Disable lighting",
                            |local| *local.get_disable_lighting(),
                            AppSettingsDispatch::set_disable_lighting,
                        )
                    }
                    {
                        SettingsView::view_checkbox(
                            ctx,
                            "Debug axes",
                            |local| *local.get_debug_axes(),
                            AppSettingsDispatch::set_debug_axes,
                        )
                    }
                </div>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        ctx.props().settings.dispatch.emit(msg);
        true
    }
}

impl SettingsView {
    fn view_checkbox<G, S>(ctx: &Context<Self>, name: &str, getter: G, setter: S) -> Html
    where
        G: Fn(&AppSettings) -> bool,
        S: Fn(&AppSettingsDispatch, bool) + 'static,
    {
        let checked = getter(&ctx.props().settings.inner.clone());
        let dispatch = ctx.props().settings.clone();
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

    fn view_slider<G, S>(
        ctx: &Context<Self>,
        name: &str,
        getter: G,
        setter: S,
        min: u32,
        max: u32,
    ) -> Html
    where
        G: Fn(&AppSettings) -> u32,
        S: Fn(&AppSettingsDispatch, u32) + 'static,
    {
        let dispatch = ctx.props().settings.clone();
        html! {
            <div class="settings__slider-setting">
                {name}
                <input
                    type="range"
                    min={min.to_string()}
                    max={max.to_string()}
                    value={getter(&ctx.props().settings.inner.clone()).to_string()}
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
