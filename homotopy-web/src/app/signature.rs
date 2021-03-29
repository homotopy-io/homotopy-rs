use crate::model::proof::{Action, GeneratorInfo};
use homotopy_core::Generator;
use im::HashMap;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub signature: HashMap<Generator, GeneratorInfo>,
    pub dispatch: Callback<Action>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {}

pub struct SignatureView {
    props: Props,
}

impl Component for SignatureView {
    type Message = Message;
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        SignatureView { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
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
        let generators: Html = self
            .props
            .signature
            .iter()
            .map(|(generator, info)| self.view_generator(*generator, info))
            .collect();

        // TODO: Order?
        // TODO: Extract drawer component
        // TODO: Add search
        // TODO: Drag to reorder
        // TODO: Folders/groups
        // TODO: On mobile, drag to the side to delete
        html! {
            <aside class="signature drawer">
                <div class="drawer__header">
                    <span class="drawer__title">
                        {"Signature"}
                    </span>
                </div>
                <div class="drawer__content">
                    <ul class="signature__generators">
                        {generators}
                    </ul>
                </div>
            </aside>
        }
    }
}

impl SignatureView {
    fn view_generator(&self, generator: Generator, info: &GeneratorInfo) -> Html {
        let dispatch = &self.props.dispatch;

        html! {
            <li
                class="signature__generator"
                onclick={dispatch.reform(move |_| Action::SelectGenerator(generator))}
            >
                <span
                    class="signature__generator-color"
                    style={format!("background: {}", info.color)}
                />
                <span class="signature__generator-name">
                    {&info.name}
                </span>
                <span class="signature__generator-dimension">
                    {info.diagram.dimension()}
                </span>
            </li>
        }
    }
}
