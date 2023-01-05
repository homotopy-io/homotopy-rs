use std::cmp;

use homotopy_core::{Boundary, Height, SliceIndex};
use im::Vector;
use yew::prelude::*;
use yew_macro::function_component;

use crate::{
    app::{Icon, IconSize},
    model::proof::View,
};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct PathControlProps {
    pub path: Vector<SliceIndex>,
    pub view: View,
    pub dimension: usize,
    pub ascend_slice: Callback<usize>,
    pub increase_view: Callback<u8>,
    pub decrease_view: Callback<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    SliceIndex(SliceIndex),
    View,
    Projection,
}

#[function_component(PathControl)]
pub fn path_control(props: &PathControlProps) -> Html {
    let path_len = props.path.len();

    let step_button = |index: usize, step: Step| -> Html {
        let label = match step {
            Step::SliceIndex(slice) => match slice {
                SliceIndex::Boundary(Boundary::Source) => "S".to_owned(),
                SliceIndex::Boundary(Boundary::Target) => "T".to_owned(),
                SliceIndex::Interior(Height::Singular(h)) => format!("S{h}"),
                SliceIndex::Interior(Height::Regular(h)) => format!("R{h}"),
            },
            Step::View => "V".to_owned(),
            Step::Projection => "P".to_owned(),
        };

        let ascend_slice = props.ascend_slice.clone();
        let increase_view = props.increase_view.clone();
        let decrease_view = props.decrease_view.clone();

        let onclick = Callback::from(move |_| {
            if index < path_len {
                ascend_slice.emit(path_len - index - 1);
            }

            if step == Step::View {
                decrease_view.emit(1);
            }

            if step == Step::Projection {
                increase_view.emit(1);
            }
        });

        html! {
            <span
                class="workspace__toolbar__button workspace__path-segment"
                onclick={onclick}
            >
                {label}
            </span>
        }
    };

    let path = {
        let mut path = Vec::with_capacity(props.dimension);
        path.extend(props.path.iter().map(|slice| Step::SliceIndex(*slice)));

        let viewing_range = path.len()
            ..cmp::min(
                path.len() + props.view.dimension() as usize,
                props.dimension,
            );

        path.extend(viewing_range.map(|_| Step::View));
        path.extend((path.len()..props.dimension).map(|_| Step::Projection));
        path
    };

    let step_buttons: Html = path
        .into_iter()
        .enumerate()
        .map(|(index, step)| step_button(index, step))
        .collect();

    html! {
        <div class="workspace__toolbar__segment">
            <span
                class="workspace__toolbar__button"
                onclick={props.ascend_slice.reform(move |_| path_len)}
            >
                <Icon name="star" size={IconSize::Icon24} />
            </span>
            {step_buttons}
        </div>
    }
}
