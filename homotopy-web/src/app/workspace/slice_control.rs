use homotopy_core::{Boundary, Height, SliceIndex};
use web_sys::Element;
use yew::prelude::*;

use crate::{
    app::{Icon, IconSize},
    components::{
        bounding_rect,
        panzoom::{PanZoom, PanZoomAgent},
    },
};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct SliceControlProps {
    pub number_slices: usize,
    pub descend_slice: Callback<SliceIndex>,
    pub diagram_ref: NodeRef,
    pub on_hover: Callback<Option<SliceIndex>>,
}

pub enum SliceControlMsg {
    Delta(f64, f64),
}

pub struct SliceControl {
    props: SliceControlProps,
    _panzoom: PanZoom,
    translate: f64,
    scale: f64,
    node_ref: NodeRef,
}

impl Component for SliceControl {
    type Message = SliceControlMsg;
    type Properties = SliceControlProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let panzoom = PanZoom::new();
        panzoom.register(Box::new(move |agent: &PanZoomAgent, _| {
            let state = agent.state();
            link.send_message(SliceControlMsg::Delta(state.translate.y, state.scale));
        }));

        Self {
            props,
            _panzoom: panzoom,
            translate: 0.0,
            scale: 1.0,
            node_ref: Default::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        let SliceControlMsg::Delta(translate, scale) = msg;
        self.translate = translate;
        self.scale = scale;
        true
    }

    fn rendered(&mut self, _first_render: bool) {
        let height = bounding_rect(&self.props.diagram_ref).unwrap().height();

        let style = format!(
            r#"
                transform: translate(0px, calc({y}px - 50%));
                height: {height}px;
                min-height: {min_height}px;
            "#,
            y = self.translate,
            height = height,
            min_height = 24 * (self.props.number_slices * 2 + 3),
        );

        self.node_ref
            .cast::<Element>()
            .unwrap()
            .set_attribute("style", &style)
            .unwrap();
    }

    fn view(&self) -> Html {
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
                    onmouseenter={self.props.on_hover.reform(move |_| Some(index))}
                    onclick={self.props.descend_slice.reform(move |_| index)}
                >
                    <Icon name="arrow_right" size={IconSize::Icon24} />
                </div>
            }
        };

        let buttons: Html = SliceIndex::for_size(self.props.number_slices as usize)
            .map(slice_button)
            .rev()
            .collect();

        html! {
            <div
                class="workspace__slice-buttons"
                onmouseleave={self.props.on_hover.reform(move |_| None)}
                ref={self.node_ref.clone()}
            >
                {buttons}
            </div>
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props != props && {
            self.props = props;
            true
        }
    }
}
