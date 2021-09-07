use homotopy_core::{Boundary, Height, SliceIndex};
use im::Vector;
use yew::prelude::*;
use yew_macro::function_component;

use crate::app::{Icon, IconSize};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct PathControlProps {
    pub path: Vector<SliceIndex>,
    pub dimension: usize,
    pub ascend_slice: Callback<usize>,
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
                SliceIndex::Interior(Height::Singular(h)) => format!("S{}", h),
                SliceIndex::Interior(Height::Regular(h)) => format!("R{}", h),
            },
            Step::View => "V".to_owned(),
            Step::Projection => "P".to_owned(),
        };

        let ascend_slice = props.ascend_slice.clone();

        let onclick = Callback::from(move |_| {
            if index < path_len {
                ascend_slice.emit(path_len - index - 1);
            }
        });

        html! {
            <span
                class="workspace__toolbar__button"
                onclick={onclick}
            >
                {label}
            </span>
        }
    };

    let path = {
        let mut path = Vec::with_capacity(props.dimension);
        path.extend(props.path.iter().map(|slice| Step::SliceIndex(*slice)));
        path.extend(
            (path.len()..std::cmp::min(path.len() + 2, props.dimension)).map(|_| Step::View),
        );
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
