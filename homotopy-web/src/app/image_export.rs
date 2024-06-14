use homotopy_graphics::{manim::ManimOptions, stl::StlOptions, tikz::TikzOptions};
use yew::prelude::*;

use super::settings::AppSettings;
use crate::{
    components::delta::CallbackIdx,
    declare_settings,
    model::{self, ImageFormat, ImageOption},
};

declare_settings! {
    pub struct ImageExportSettings {
        tikz_left_to_right: bool = false,
        tikz_show_braidings: bool = true,
        manim_use_opengl: bool = false,
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    pub dispatch: Callback<model::Action>,
    pub view_dimension: u8,
    pub dimension: usize,
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
        let tikz = Self::view_tikz(ctx);
        let svg = Self::view_svg(ctx);
        let png = Self::view_png(ctx);
        let manim = Self::view_manim(ctx);
        let stl = Self::view_stl(ctx);
        html! {
            <div class="settings">
                {tikz}
                {svg}
                {png}
                {manim}
                {stl}
            </div>
        }
    }
}

impl ImageExportView {
    fn view_tikz(ctx: &Context<Self>) -> Html {
        let options = TikzOptions {
            left_to_right: ImageExportSettings::get_tikz_left_to_right(),
            show_braidings: ImageExportSettings::get_tikz_show_braidings(),
        };
        let export_tikz = |option| {
            ctx.props()
                .dispatch
                .reform(move |_| model::Action::ExportImage(ImageFormat::Tikz(options), option))
        };
        if ctx.props().view_dimension <= 2 {
            html! {
                <>
                    <h3>{"Export to TikZ"}</h3>
                    <div class="settings__segment">
                        {
                            Self::view_checkbox(
                                "Left-to-right",
                                ImageExportSettings::get_tikz_left_to_right(),
                                ImageExportSettings::set_tikz_left_to_right,
                            )
                        }
                        {
                            Self::view_checkbox(
                                "Show braidings",
                                ImageExportSettings::get_tikz_show_braidings(),
                                ImageExportSettings::set_tikz_show_braidings,
                            )
                        }
                        <button onclick={export_tikz(ImageOption::Single)}>{"Export"}</button>

                        if ctx.props().dimension > 0 {
                            <button onclick={export_tikz(ImageOption::Multiple)}>{"Export slices"}</button>
                        }
                    </div>
                </>
            }
        } else {
            Default::default()
        }
    }

    fn view_svg(ctx: &Context<Self>) -> Html {
        let export_svg = ctx
            .props()
            .dispatch
            .reform(move |_| model::Action::ExportImage(ImageFormat::Svg, ImageOption::Single));
        if ctx.props().view_dimension <= 2 {
            html! {
                <>
                    <h3>{"Export to SVG"}</h3>
                    <div class="settings__segment">
                        <button onclick={export_svg}>{"Export"}</button>
                    </div>
                </>
            }
        } else {
            Default::default()
        }
    }

    fn view_png(ctx: &Context<Self>) -> Html {
        let export_png = ctx
            .props()
            .dispatch
            .reform(move |_| model::Action::ExportImage(ImageFormat::Png, ImageOption::Single));
        if ctx.props().view_dimension >= 3 {
            html! {
                <>
                    <h3>{"Export to PNG"}</h3>
                    <div class="settings__segment">
                        <button onclick={export_png}>{"Export"}</button>
                    </div>
                </>
            }
        } else {
            Default::default()
        }
    }

    fn view_manim(ctx: &Context<Self>) -> Html {
        let options = ManimOptions {
            use_opengl: ImageExportSettings::get_manim_use_opengl(),
        };
        let export_manim = |option| {
            ctx.props()
                .dispatch
                .reform(move |_| model::Action::ExportImage(ImageFormat::Manim(options), option))
        };
        if ctx.props().view_dimension <= 2 {
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
                        <button onclick={export_manim(ImageOption::Single)}>{"Export"}</button>

                        if ctx.props().dimension > 0 {
                            <button onclick={export_manim(ImageOption::Multiple)}>{"Export slices"}</button>
                        }
                    </div>
                </>
            }
        } else {
            Default::default()
        }
    }

    fn view_stl(ctx: &Context<Self>) -> Html {
        let options = StlOptions {
            geometry_samples: AppSettings::get_geometry_samples() as u8,
            subdivision_depth: AppSettings::get_subdivision_depth() as u8,
        };
        let export_stl = |option| {
            ctx.props()
                .dispatch
                .reform(move |_| model::Action::ExportImage(ImageFormat::Stl(options), option))
        };
        if ctx.props().view_dimension == 3 {
            html! {
                <>
                    <h3>{"Export to STL"}</h3>
                    <div class="settings__segment">
                        <button onclick={export_stl(ImageOption::Single)}>{"Export"}</button>

                        if ctx.props().dimension > 3 {
                            <button onclick={export_stl(ImageOption::Multiple)}>{"Export slices"}</button>
                        }
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
