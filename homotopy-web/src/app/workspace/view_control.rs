use yew::prelude::*;

use crate::{
    app::{diagram_gl::GlViewControl, Icon, IconSize},
    components::panzoom::PanZoom,
};

pub struct ViewControl {}

pub enum ViewMessage {
    ZoomIn,
    ZoomOut,
    Reset,
}

impl Component for ViewControl {
    type Message = ViewMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ViewMessage::ZoomIn => {
                PanZoom::zoom_in();
                GlViewControl::zoom_in();
            }
            ViewMessage::ZoomOut => {
                PanZoom::zoom_out();
                GlViewControl::zoom_out();
            }
            ViewMessage::Reset => {
                PanZoom::reset();
                GlViewControl::reset();
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
