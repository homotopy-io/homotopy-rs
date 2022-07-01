use yew::prelude::*;

use homotopy_core::common::Boundary;

use crate::{
    // app::{attach::AttachView, keybindings::Keybindings},
    // components::{
    //     icon::{Icon, IconSize},
    //     Visibility,
    // },
    app::diagram_svg::DiagramSvg,
    model::proof::{SelectedBoundary},
};

pub struct BoundaryPreview {}

pub enum BoundaryPreviewMessage {}

#[derive(Clone, PartialEq, Properties)]
pub struct BoundaryPreviewProps {
    pub boundary: SelectedBoundary,
}

impl Component for BoundaryPreview {
    type Message = BoundaryPreviewMessage;
    type Properties = BoundaryPreviewProps;
    
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let s = match ctx.props().boundary.boundary {
            Boundary::Source => "Source: ",
            Boundary::Target => "Target: ",
        };

        let preview = match ctx.props().boundary.diagram.dimension() {
            0 => Self::view_diagram_svg::<0>(ctx),
            1 => Self::view_diagram_svg::<1>(ctx),
            _ => Self::view_diagram_svg::<2>(ctx),
        };

        html!{ 
            <div class="boundary__preview">
                <span>{s}</span>
                {preview}
            </div>
        }
    }

}

impl BoundaryPreview {
    fn view_diagram_svg<const N: usize>(ctx: &Context<Self>) -> Html {
        html!{
            <DiagramSvg<N>
                    diagram={ctx.props().boundary.diagram.clone()}
                    id="boundary__preview"
            />
        }
    }
}

