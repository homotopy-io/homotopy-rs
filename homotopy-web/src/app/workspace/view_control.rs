use yew::prelude::*;

use crate::{
    app::{diagram_gl::OrbitControl, Icon, IconSize},
    components::panzoom::PanZoom,
};

pub struct ViewControl {
    panzoom: PanZoom,
    orbit_control: OrbitControl,
}

pub enum ViewMessage {
    ZoomIn,
    ZoomOut,
    Reset,
}

impl Component for ViewControl {
    type Message = ViewMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            panzoom: PanZoom::new(),
            orbit_control: OrbitControl::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ViewMessage::ZoomIn => {
                self.panzoom.zoom_in();
                self.orbit_control.zoom_in();
            },
            ViewMessage::ZoomOut => {
                self.panzoom.zoom_out();
                self.orbit_control.zoom_out();
            }
            ViewMessage::Reset => {
                self.panzoom.reset();
                self.orbit_control.reset();
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
