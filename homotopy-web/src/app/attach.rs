use std::convert::TryFrom;

use homotopy_core::DiagramN;
use yew::prelude::*;

use crate::model::proof::{Action, AttachOption, GeneratorInfo, Signature};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    pub dispatch: Callback<Action>,
    pub options: im::Vector<AttachOption>,
    pub signature: Signature,
}

#[derive(Debug, Clone)]
pub enum Message {}

pub struct AttachView {
    props: Props,
}

impl Component for AttachView {
    type Message = Message;
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            { for self.props.options.iter().map(|option| self.view_option(option)) }
        }
    }
}

impl AttachView {
    pub fn view_option(&self, option: &AttachOption) -> Html {
        let info = self
            .props
            .signature
            .generator_info(option.generator)
            .cloned()
            .unwrap_or_else(|| {
                let generator = option.generator.inverse();
                let i = self.props.signature.generator_info(generator).unwrap();
                let d = DiagramN::try_from(i.diagram.clone())
                    .expect("conversion to DiagramN failed")
                    .inverse()
                    .expect("inversion failed");
                GeneratorInfo {
                    generator,
                    name: format!("{} (inverse)", i.name),
                    diagram: d.into(),
                    color: i.color.clone(),
                }
            });

        let onclick = self.props.dispatch.reform({
            let option = option.clone();
            move |_| Action::Attach(option.clone())
        });

        let onmouseenter = self.props.dispatch.reform({
            let option = option.clone();
            move |_| Action::HighlightAttachment(Some(option.clone()))
        });

        let onmouseleave = self
            .props
            .dispatch
            .reform(|_| Action::HighlightAttachment(None));

        // TODO: Maybe extract a common component for this and the signature.
        // TODO: How should highlights work on touch devices?
        html! {
            <li
                class="attach__option"
                onclick={onclick}
                onmouseenter={onmouseenter}
                onmouseleave={onmouseleave}
            >
                <span
                    class="attach__option-color"
                    style={format!("background: {}", info.color)}
                />
                <span class="attach__option-name">
                    {&info.name}
                </span>
            </li>
        }
    }
}
