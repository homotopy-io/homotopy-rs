use yew::prelude::*;

use crate::{
    app::{Icon, IconSize},
    components::panzoom::PanZoom,
};

pub struct ViewControl {
    link: ComponentLink<Self>,
    panzoom: PanZoom,
}

pub enum ViewMessage {
    ZoomIn,
    ZoomOut,
    Reset,
}

impl Component for ViewControl {
    type Message = ViewMessage;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            panzoom: PanZoom::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            ViewMessage::ZoomIn => self.panzoom.zoom_in(),
            ViewMessage::ZoomOut => self.panzoom.zoom_out(),
            ViewMessage::Reset => self.panzoom.reset(),
        }
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="workspace__toolbar__segment">
                <span
                    class="workspace__toolbar__button"
                    onclick={self.link.callback(|_| ViewMessage::ZoomIn)}
                >
                    <Icon name="zoom_in" size={IconSize::Icon24} />
                </span>
                <span
                    class="workspace__toolbar__button"
                    onclick={self.link.callback(|_| ViewMessage::ZoomOut)}
                >
                    <Icon name="zoom_out" size={IconSize::Icon24} />
                </span>
                <span
                    class="workspace__toolbar__button"
                    onclick={self.link.callback(|_| ViewMessage::Reset)}
                >
                    <Icon name="filter_center_focus" size={IconSize::Icon24} />
                </span>

            </div>
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
}
