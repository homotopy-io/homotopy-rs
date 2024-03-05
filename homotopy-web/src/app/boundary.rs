/*
    Display the selected source/target (boundary) at the left-bottom corner of the workspace.
*/

use homotopy_core::common::Boundary;
use yew::prelude::*;

use crate::{
    app::diagram_svg::DiagramSvg,
    components::icon::{Icon, IconSize},
    model::proof::{Action, SelectedBoundary, Signature},
};

#[derive(Clone, PartialEq, Properties)]
pub struct BoundaryPreviewProps {
    pub boundary: SelectedBoundary,
    pub dispatch: Callback<Action>,
    pub signature: Signature,
}

#[function_component]
pub fn BoundaryPreview(props: &BoundaryPreviewProps) -> Html {
    let bound = match props.boundary.boundary {
        Boundary::Source => "Source",
        Boundary::Target => "Target",
    };

    let dim = props.boundary.diagram.dimension();

    let preview = match dim {
        0 => view_diagram_svg::<0>(props),
        1 => view_diagram_svg::<1>(props),
        _ => view_diagram_svg::<2>(props),
    };

    let onclick = props.dispatch.reform(move |_| Action::RecoverBoundary);
    let preview = html! {
        <div
            class="boundary__element boundary__preview"
            onclick={onclick}
        >
            {preview}
        </div>
    };

    html! {
        <div class="boundary">
            <div
                class="boundary__element boundary__name"
                onclick={props.dispatch.reform(move |_| Action::FlipBoundary)}
            >
                <span>{bound}</span>
            </div>
            <div
                class="boundary__element boundary__button"
                onclick={props.dispatch.reform(move |_| Action::ClearBoundary)}
            >
                <Icon name="close" size={IconSize::Icon18} />
            </div>
            {preview}
        </div>
    }
}

fn view_diagram_svg<const N: usize>(props: &BoundaryPreviewProps) -> Html {
    html! {
        <DiagramSvg<N>
                diagram={props.boundary.diagram.clone()}
                id="boundary__preview"
                signature={props.signature.clone()}
                max_width={Some(160.0)}
                max_height={Some(160.0)}
        />
    }
}
