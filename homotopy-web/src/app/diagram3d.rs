use yew::prelude::*;
use homotopy_core::{DiagramN};

pub struct Diagram3D {
    props: Props3D,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props3D {
    pub diagram: DiagramN,
}

#[allow(clippy::pub_enum_variant_names)]
#[derive(Debug)]
pub enum Message3D {
}

impl Component for Diagram3D {
    type Message = Message3D;
    type Properties = Props3D;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        log::info!("Hello, rendering a workspace component {:?}", &props.diagram);
        log::info!("Hello, this is link {:?}", link);
        Self {
            props
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        log::info!("Hello, this is msg {:?}", msg);
        false
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
        log::info!("Hello, view was called");

        html! {
            <div>{"todo: 3-dimensional diagram"}</div>
        }
    }
}

impl Diagram3D {

}