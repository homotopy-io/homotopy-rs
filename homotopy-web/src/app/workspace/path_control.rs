use crate::app::{Icon, IconSize};
use homotopy_core::{Boundary, Height, SliceIndex};
use im::Vector;
use yew::prelude::*;
use yew_functional::function_component;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct PathControlProps {
    pub path: Vector<SliceIndex>,
    pub ascend_slice: Callback<usize>,
}

#[function_component(PathControl)]
pub fn path_control(props: &PathControlProps) -> Html {
    let path_len = props.path.len();

    let step_button = |index: usize, slice: SliceIndex| -> Html {
        let label = match slice {
            SliceIndex::Boundary(Boundary::Source) => "S".to_owned(),
            SliceIndex::Boundary(Boundary::Target) => "T".to_owned(),
            SliceIndex::Interior(Height::Singular(h)) => format!("s{}", h),
            SliceIndex::Interior(Height::Regular(h)) => format!("r{}", h),
        };

        html! {
            <span
                class="workspace__path-step"
                onclick={props.ascend_slice.reform(move |_| path_len - index - 1)}
            >
                {label}
            </span>
        }
    };

    let step_buttons: Html = props
        .path
        .iter()
        .enumerate()
        .map(|(index, slice)| step_button(index, *slice))
        .collect();

    let class = if props.path.is_empty() {
        "workspace__path workspace__path--empty"
    } else {
        "workspace__path"
    };

    html! {
        <div class={class}>
            <span
                class="workspace__path-home"
                onclick={props.ascend_slice.reform(move |_| path_len)}
            >
                <Icon name="menu" size={IconSize::Icon24} />
            </span>
            {step_buttons}
        </div>
    }
}
