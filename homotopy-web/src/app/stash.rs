use homotopy_core::Diagram;
use homotopy_model::proof::{self, Signature, Workspace};
use im::Vector;
use yew::prelude::*;
use yew_macro::function_component;

use crate::{app::diagram_svg::DiagramSvg, model::Action};

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub stash: Vector<Workspace>,
    pub dispatch: Callback<Action>,
    pub signature: Signature,
}

#[function_component]
pub fn StashView(props: &Props) -> Html {
    let stash = props.dispatch.reform(|_| proof::Action::Stash.into());
    let stash_drop = props.dispatch.reform(|_| proof::Action::StashDrop.into());
    let stash_pop = props.dispatch.reform(|_| proof::Action::StashPop.into());
    let stash_apply = props.dispatch.reform(|_| proof::Action::StashApply.into());

    let diagrams = props.stash.iter().map(|ws| {
        let diagram = match ws.view.dimension() {
            0 => view_diagram::<0>(ws.visible_diagram(), &props.signature),
            1 => view_diagram::<1>(ws.visible_diagram(), &props.signature),
            _ => view_diagram::<2>(ws.visible_diagram(), &props.signature),
        };
        html! {
            <div class={"stash__element stash__diagram"}>
                {diagram}
            </div>
        }
    });

    html! {
        <>
            <div>
                <button onclick={stash}>{"Stash"}</button>
                <button onclick={stash_drop}>{"Drop"}</button>
                <button onclick={stash_pop}>{"Pop"}</button>
                <button onclick={stash_apply}>{"Apply"}</button>
            </div>
            <>
                {for diagrams}
            </>
        </>
    }
}

fn view_diagram<const N: usize>(diagram: Diagram, signature: &Signature) -> Html {
    html! {
            <DiagramSvg<N>
                id={"stash__diagram"}
                diagram={diagram}
                signature={signature.clone()}
                max_width={Some(200.)}
                max_height={Some(200.)}
            />
    }
}
