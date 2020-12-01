// https://github.com/yewstack/yew/issues/1281#issuecomment-637508696

use web_sys::Element;
use yew::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Properties)]
pub struct RawHTMLProps {
  pub inner_html: String,
}

pub struct RawHTML {
    props: RawHTMLProps,
    node_ref: NodeRef,
}

impl Component for RawHTML {
    type Message = ();
    type Properties = RawHTMLProps;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self { props, node_ref: NodeRef::default() }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
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
