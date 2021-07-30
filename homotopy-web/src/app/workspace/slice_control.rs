use yew::prelude::*;

use homotopy_core::{Boundary, Height, SliceIndex};

use crate::app::{Icon, IconSize};
use crate::components::panzoom::{PanZoom, PanZoomAgent};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct SliceControlProps {
    pub number_slices: usize,
    pub descend_slice: Callback<SliceIndex>,
}

pub enum SliceControlMsg {
    Delta(f64, f64),
}

pub struct SliceControl {
    props: SliceControlProps,
    _panzoom: PanZoom,
    translate: f64,
    scale: f64,
}

impl Component for SliceControl {
    type Properties = SliceControlProps;
    type Message = SliceControlMsg;

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
            scale: 0.0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        let SliceControlMsg::Delta(translate, scale) = msg;
        self.translate = translate;
        self.scale = scale;
        true
    }

    fn view(&self) -> Html {
        let slice_button = |index: SliceIndex| -> Html {
            let label = match index {
                SliceIndex::Boundary(Boundary::Source) => "Source".to_owned(),
                SliceIndex::Boundary(Boundary::Target) => "Target".to_owned(),
                SliceIndex::Interior(Height::Regular(i)) => format!("Regular {}", i),
                SliceIndex::Interior(Height::Singular(i)) => format!("Singular {}", i),
            };
            let style = format!("height: {height}px;", height = self.scale * 40.0);

            html! {
                <div
                    class="workspace__slice-button tooltip tooltip--left"
                    data-tooltip={label}
                    style={style}
                    onclick={self.props.descend_slice.reform(move |_| index)}
                >
                    <Icon name="arrow_right" size={IconSize::Icon24} />
                </div>
            }
        };

        let buttons: Html = SliceIndex::for_size(self.props.number_slices)
            .map(slice_button)
            .rev()
            .collect();

        let style = format!(
            "transform: translate(0px, calc({y}px - 50%))",
            y = self.translate - (0.5 * 40.0 * self.scale)
        );

        html! {
            <div class="workspace__slice-buttons" style={style}>
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
