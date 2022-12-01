use homotopy_core::{Boundary, Height, SliceIndex};
use web_sys::Element;
use yew::prelude::*;

use crate::{
    app::{Icon, IconSize},
    components::{bounding_rect, panzoom::PanZoomState},
};

#[derive(Clone, PartialEq, Properties)]
pub struct SliceControlProps {
    pub number_slices: usize,
    pub descend_slice: Callback<SliceIndex>,
    pub diagram_ref: NodeRef,
    pub on_hover: Callback<Option<SliceIndex>>,
    pub panzoom: PanZoomState,
}

pub struct SliceControl {
    node_ref: NodeRef,
}

impl Component for SliceControl {
    type Message = ();
    type Properties = SliceControlProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            node_ref: Default::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        let height = bounding_rect(&ctx.props().diagram_ref)
            .map(|rect| rect.height())
            .unwrap_or_default();

        let style = format!(
            r#"
                transform: translate(0px, calc({y}px - 50%));
                height: {height}px;
                min-height: {min_height}px;
            "#,
            y = ctx.props().panzoom.translate.y,
            height = height,
            min_height = 24 * (ctx.props().number_slices * 2 + 3),
        );

        self.node_ref
            .cast::<Element>()
            .unwrap()
            .set_attribute("style", &style)
            .unwrap();
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let slice_button = |index: SliceIndex| -> Html {
            let label = match index {
                SliceIndex::Boundary(Boundary::Source) => "Source".to_owned(),
                SliceIndex::Boundary(Boundary::Target) => "Target".to_owned(),
                SliceIndex::Interior(Height::Regular(i)) => format!("Regular {}", i),
                SliceIndex::Interior(Height::Singular(i)) => format!("Singular {}", i),
            };

            html! {
                <div
                    class="workspace__slice-button tooltip tooltip--left"
                    data-tooltip={label}
                    onmouseenter={ctx.props().on_hover.reform(move |_| Some(index))}
                    onclick={ctx.props().descend_slice.reform(move |_| index)}
                >
                    <Icon name="arrow_right" size={IconSize::Icon24} />
                </div>
            }
        };

        let buttons: Html = SliceIndex::for_size(ctx.props().number_slices as usize)
            .map(slice_button)
            .rev()
            .collect();

        html! {
            <div
                class="workspace__slice-buttons"
                onmouseleave={ctx.props().on_hover.reform(move |_| None)}
                ref={self.node_ref.clone()}
            >
                {buttons}
            </div>
        }
    }
}
