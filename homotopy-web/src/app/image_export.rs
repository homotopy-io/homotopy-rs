use yew::prelude::*;

use crate::{components::delta::CallbackIdx, declare_settings, model};

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
    callback_idxs: Vec<CallbackIdx>,
}

impl Component for ImageExportView {
    type Message = ImageExportSettingsMsg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        // So that we can keep our local copy of the global settings up to date,
        // we're going to need to subscribe to all changes in the global settings state.
        let callback_idxs =
            ImageExportSettings::subscribe(ImageExportSettings::ALL, ctx.link().callback(|x| x));
        Self { callback_idxs }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        ImageExportSettings::unsubscribe(ImageExportSettings::ALL, &self.callback_idxs);
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
        let tikz = Self::view_tikz(ctx);
        let svg = Self::view_svg(ctx);
        let manim = Self::view_manim(ctx);
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
    fn view_tikz(ctx: &Context<Self>) -> Html {
        let show_braidings = ImageExportSettings::get_tikz_show_braidings();
        let leftright_mode = ImageExportSettings::get_tikz_leftright_mode();
        if ctx.props().view_dim == 2 {
            html! {
                <>
                    <h3>{"Export to TikZ"}</h3>
                    <div class="settings__segment">
                        {
                            Self::view_checkbox(
                                "Left-right mode",
                                ImageExportSettings::get_tikz_leftright_mode(),
                                ImageExportSettings::set_tikz_leftright_mode,
                            )
                        }
                        {
                            Self::view_checkbox(
                                "Show braidings",
                                ImageExportSettings::get_tikz_show_braidings(),
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

    fn view_manim(ctx: &Context<Self>) -> Html {
        let use_opengl = ImageExportSettings::get_manim_use_opengl();
        if ctx.props().view_dim == 2 {
            html! {
                <>
                    <h3>{"Export to Manim"}</h3>
                    <div class="settings__segment">
                        {
                            Self::view_checkbox(
                                "OpenGL renderer",
                                ImageExportSettings::get_manim_use_opengl(),
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
}
