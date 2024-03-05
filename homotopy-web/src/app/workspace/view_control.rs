use yew::prelude::*;

use crate::{
    app::{diagram_gl::GlViewControl, Icon, IconSize},
    components::panzoom::PanZoom,
};

#[function_component]
pub fn ViewControl() -> Html {
    let zoom_in = Callback::from(|_| {
        PanZoom::zoom_in();
        GlViewControl::zoom_in();
    });
    let zoom_out = Callback::from(|_| {
        PanZoom::zoom_out();
        GlViewControl::zoom_out();
    });
    let reset = Callback::from(|_| {
        PanZoom::reset();
        GlViewControl::reset();
    });

    html! {
        <div class="workspace__toolbar__segment">
            <span
                class="workspace__toolbar__button"
                onclick={zoom_in}
            >
                <Icon name="zoom_in" size={IconSize::Icon24} />
            </span>
            <span
                class="workspace__toolbar__button"
                onclick={zoom_out}
            >
                <Icon name="zoom_out" size={IconSize::Icon24} />
            </span>
            <span
                class="workspace__toolbar__button"
                onclick={reset}
            >
                <Icon name="filter_center_focus" size={IconSize::Icon24} />
            </span>
        </div>
    }
}
