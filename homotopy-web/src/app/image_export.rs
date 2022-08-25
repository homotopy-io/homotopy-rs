use yew::prelude::*;

use crate::model;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    pub dispatch: Callback<model::Action>,
    pub view_dim: u8,
}

#[derive(Debug, Clone)]
pub enum Message {}

pub struct ImageExportView;

impl Component for ImageExportView {
    type Message = Message;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let tikz = Self::view_tikz(ctx);
        let svg = Self::view_svg(ctx);
        let manim = Self::view_manim(ctx);
        let stl = Self::view_stl(ctx);
        html! {
            <>
                {tikz}
                {svg}
                {manim}
                {stl}
            </>

        }
    }
}

impl ImageExportView {
    fn view_tikz(ctx: &Context<Self>) -> Html {
        if ctx.props().view_dim == 2 {
            html! {
                <>
                    <h3>{"Export to TikZ"}</h3>
                    <button onclick={ctx.props().dispatch.reform(move |_| model::Action::ExportTikz)}>{"Export"}</button>
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
                    <button onclick={ctx.props().dispatch.reform(move |_| model::Action::ExportSvg)}>{"Export"}</button>
                </>
            }
        } else {
            Default::default()
        }
    }

    fn view_manim(ctx: &Context<Self>) -> Html {
        if ctx.props().view_dim == 2 {
            html! {
                <>
                    <h3>{"Export to Manim"}</h3>
                    <button onclick={ctx.props().dispatch.reform(move |_| model::Action::ExportManim)}>{"Export"}</button>
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
                    <button onclick={ctx.props().dispatch.reform(move |_| model::Action::ExportStl)}>{"Export"}</button>
                </>
            }
        } else {
            Default::default()
        }
    }
}
