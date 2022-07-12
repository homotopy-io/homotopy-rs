/*
    Display the selected source/target (boundary) at the left-bottom corner of the workspace.
*/

use homotopy_core::common::Boundary;
use yew::prelude::*;

use crate::{app::diagram_svg::DiagramSvg, model::proof::SelectedBoundary};

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
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let bound = match ctx.props().boundary.boundary {
            Boundary::Source => "Source",
            Boundary::Target => "Target",
        };

        let dim = ctx.props().boundary.diagram.dimension();

        let preview = match dim {
            0 => Self::view_diagram_svg::<0>(ctx),
            1 => Self::view_diagram_svg::<1>(ctx),
            _ => Self::view_diagram_svg::<2>(ctx),
        };

        let preview = match dim {
            // Display flex to center 0 & 1-dimensional diagrams.
            0 | 1 => html! {
                <div class="boundary__preview" style="display:flex; align-items:center; justify-content:center">
                    {preview}
                </div>
            },
            _ => html! {
                <div class="boundary__preview">
                    {preview}
                </div>
            },
        };

        html! {
            <div class="boundary">
                <div class="boundary__name">
                    <span>{bound}</span>
                </div>
                {preview}
            </div>
        }
    }
}

impl BoundaryPreview {
    fn view_diagram_svg<const N: usize>(ctx: &Context<Self>) -> Html {
        html! {
            <DiagramSvg<N>
                    diagram={ctx.props().boundary.diagram.clone()}
                    id="boundary__preview"
            />
        }
    }
}
