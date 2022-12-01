use yew::prelude::*;

use crate::{
    app::{diagram_gl::GlViewControl, Icon, IconSize},
    components::panzoom::{PanZoomDispatch, PanZoomState, Zoomable},
};

#[derive(Clone, PartialEq, Properties)]
pub struct ViewProps {
    pub panzoom: PanZoomState,
    pub dispatch: PanZoomDispatch,
}

pub struct ViewControl {
    orbit_control: GlViewControl,
}

pub enum ViewMessage {
    ZoomIn,
    ZoomOut,
    Reset,
}

impl Component for ViewControl {
    type Message = ViewMessage;
    type Properties = ViewProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            orbit_control: GlViewControl::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ViewMessage::ZoomIn => {
                self.orbit_control.zoom_in();
                ctx.props().dispatch.zoom_in();
            }
            ViewMessage::ZoomOut => {
                self.orbit_control.zoom_out();
                ctx.props().dispatch.zoom_out();
            }
            ViewMessage::Reset => {
                self.orbit_control.reset();
                ctx.props().dispatch.reset();
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="workspace__toolbar__segment">
                <span
                    class="workspace__toolbar__button"
                    onclick={ctx.link().callback(|_| ViewMessage::ZoomIn)}
                >
                    <Icon name="zoom_in" size={IconSize::Icon24} />
                </span>
                <span
                    class="workspace__toolbar__button"
                    onclick={ctx.link().callback(|_| ViewMessage::ZoomOut)}
                >
                    <Icon name="zoom_out" size={IconSize::Icon24} />
                </span>
                <span
                    class="workspace__toolbar__button"
                    onclick={ctx.link().callback(|_| ViewMessage::Reset)}
                >
                    <Icon name="filter_center_focus" size={IconSize::Icon24} />
                </span>

            </div>
        }
    }
}
