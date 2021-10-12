// https://github.com/yewstack/yew/issues/1281#issuecomment-637508696

use web_sys::Element;
use yew::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Properties)]
pub struct RawHtmlProps {
    pub inner_html: String,
}

pub struct RawHtml {
    node_ref: NodeRef,
}

impl Component for RawHtml {
    type Message = ();
    type Properties = RawHtmlProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _: Self::Message) -> bool {
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        // create the parent element and store a reference to it
        html! {
            <div ref={self.node_ref.clone()}/>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        let el = self.node_ref.cast::<Element>().unwrap();
        el.set_inner_html(&ctx.props().inner_html);
    }
}
