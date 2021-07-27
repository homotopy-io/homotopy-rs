use crate::app::{Icon, IconSize};
use homotopy_core::{Boundary, Height, SliceIndex};
use yew::prelude::*;
use yew_functional::function_component;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct SliceControlProps {
    pub number_slices: usize,
    pub scale: f64,
    pub translate: f64,
    pub descend_slice: Callback<SliceIndex>,
}

#[function_component(SliceControl)]
pub fn slice_control(props: &SliceControlProps) -> Html {
    let slice_button = |index: SliceIndex| -> Html {
        let label = match index {
            SliceIndex::Boundary(Boundary::Source) => "Source".to_owned(),
            SliceIndex::Boundary(Boundary::Target) => "Target".to_owned(),
            SliceIndex::Interior(Height::Regular(i)) => format!("Regular {}", i),
            SliceIndex::Interior(Height::Singular(i)) => format!("Singular {}", i),
        };
        let style = format!("height: {height}px;", height = props.scale * 40.0);

        html! {
            <div
                class="workspace__slice-button tooltip tooltip--left"
                data-tooltip={label}
                style={&style}
                onclick={props.descend_slice.reform(move |_| index)}
            >
                <Icon name="arrow_right" size={IconSize::Icon24} />
            </div>
        }
    };

    let buttons: Html = SliceIndex::for_size(props.number_slices)
        .map(slice_button)
        .rev()
        .collect();

    let style = format!(
        "transform: translate(0px, calc({y}px - 50%))",
        y = props.translate - (0.5 * 40.0 * props.scale)
    );

    html! {
        <div class="workspace__slice-buttons" style={style}>
            {buttons}
        </div>
    }
}
