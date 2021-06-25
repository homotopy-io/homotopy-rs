// https://github.com/yewstack/yew/issues/1281#issuecomment-637508696

use web_sys::Element;
use yew::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Properties)]
pub struct RawHtmlProps {
    pub inner_html: String,
}

pub struct RawHtml {
    props: RawHtmlProps,
    node_ref: NodeRef,
}

impl Component for RawHtml {
    type Message = ();
    type Properties = RawHtmlProps;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {
            props,
            node_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props == props {
            false
        } else {
            self.props = props;
            true
        }
    }

    fn view(&self) -> Html {
        // create the parent element and store a reference to it
        html! {
            <div ref=self.node_ref.clone()/>
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        let el = self.node_ref.cast::<Element>().unwrap();
        el.set_inner_html(&self.props.inner_html);
    }
}
