use yew::prelude::*;

use crate::{
    components::settings::{KeyStore, Settings},
    declare_settings, model,
};

declare_settings! {
    pub struct ImageExportSettings {
        tikz_leftright_mode: bool = false,
        tikz_show_braidings: bool = true,
        manim_use_opengl: bool = false,
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    pub dispatch: Callback<model::Action>,
    pub view_dim: u8,
}

pub struct ImageExportView {
    // Maintain a local copy of the global app settings in order to display the current settings
    // state correctly.
    local: ImageExportSettingsKeyStore,
}

impl Component for ImageExportView {
    type Message = ImageExportSettingsMsg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        // So that we can keep our local copy of the global settings up to date,
        // we're going to need to subscribe to all changes in the global settings state.
        ImageExportSettings::subscribe(ImageExportSettings::ALL, ctx.link().callback(|x| x));
        Self {
            local: Default::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.local.set(&msg);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let default_text = if ctx.props().view_dim == 2 || ctx.props().view_dim == 3 {
            Default::default()
        } else {
            html! {
                <p>{
                    "There is nothing to export. \n
                    Try creating a 2D/3D diagram or change to 2D/3D view with the view buttons
                    at the top-right corner."}
                </p>
            }
        };
        let tikz = self.view_tikz(ctx);
        let svg = Self::view_svg(ctx);
        let manim = self.view_manim(ctx);
        let stl = Self::view_stl(ctx);
        html! {
            <div class="settings">
                {default_text}
                {tikz}
                {svg}
                {manim}
                {stl}
            </div>

        }
    }
}

impl ImageExportView {
    fn view_tikz(&self, ctx: &Context<Self>) -> Html {
        let show_braidings = *self.local.get_tikz_show_braidings();
        let leftright_mode = *self.local.get_tikz_leftright_mode();
        if ctx.props().view_dim == 2 {
            html! {
                <>
                    <h3>{"Export to TikZ"}</h3>
                    <div class="settings__segment">
                        {
                            self.view_checkbox(
                                "Left-right mode",
                                |local| *local.get_tikz_leftright_mode(),
                                ImageExportSettings::set_tikz_leftright_mode,
                            )
                        }
                        {
                            self.view_checkbox(
                                "Show braidings",
                                |local| *local.get_tikz_show_braidings(),
                                ImageExportSettings::set_tikz_show_braidings,
                            )
                        }
                        <button onclick={ctx.props().dispatch.reform(move |_| model::Action::ExportTikz(leftright_mode,show_braidings))}>{"Export"}</button>
                    </div>
                </>
            }
        } else {
            Default::default()
        }
    }

    fn view_svg(ctx: &Context<Self>) -> Html {
        if ctx.props().view_dim == 2 {
            html! {
                <>
                    <h3>{"Export to SVG"}</h3>
                    <div class="settings__segment">
                        <button onclick={ctx.props().dispatch.reform(move |_| model::Action::ExportSvg)}>{"Export"}</button>
                    </div>
                </>
            }
        } else {
            Default::default()
        }
    }

    fn view_manim(&self, ctx: &Context<Self>) -> Html {
        let use_opengl = *self.local.get_manim_use_opengl();
        if ctx.props().view_dim == 2 {
            html! {
                <>
                    <h3>{"Export to Manim"}</h3>
                    <div class="settings__segment">
                        {
                            self.view_checkbox(
                                "OpenGL renderer",
                                |local| *local.get_manim_use_opengl(),
                                ImageExportSettings::set_manim_use_opengl,
                            )
                        }
                        <button onclick={ctx.props().dispatch.reform(move |_| model::Action::ExportManim(use_opengl))}>{"Export"}</button>
                    </div>
                </>
            }
        } else {
            Default::default()
        }
    }

    fn view_stl(ctx: &Context<Self>) -> Html {
        if ctx.props().view_dim == 3 {
            html! {
                <>
                    <h3>{"Export to STL"}</h3>
                    <div class="settings__segment">
                        <button onclick={ctx.props().dispatch.reform(move |_| model::Action::ExportStl)}>{"Export"}</button>
                    </div>
                </>
            }
        } else {
            Default::default()
        }
    }

    fn view_checkbox<G, S>(&self, name: &str, getter: G, setter: S) -> Html
    where
        G: Fn(&ImageExportSettingsKeyStore) -> bool,
        S: Fn(&ImageExportSettings, bool) + 'static,
    {
        let checked = getter(&self.local);
        let dispatch = ImageExportSettings::new();

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
}
