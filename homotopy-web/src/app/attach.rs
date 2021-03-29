use crate::model::proof::{Action, AttachOption, GeneratorInfo};
use homotopy_core::common::*;
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    pub dispatch: Callback<Action>,
    pub options: im::Vector<AttachOption>,
    pub signature: im::HashMap<Generator, GeneratorInfo>,
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
        AttachView { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let options: Html = self
            .props
            .options
            .iter()
            .map(|option| self.view_option(option))
            .collect();

        html! {
            <aside class="attach drawer">
                <div class="drawer__header">
                    <span class="drawer__title">
                        {"Attach"}
                    </span>
                </div>
                <div class="drawer__content">
                    <ul class="attach__options">
                        {options}
                    </ul>
                </div>
            </aside>
        }
    }
}

impl AttachView {
    pub fn view_option(&self, option: &AttachOption) -> Html {
        let info = self.props.signature.get(&option.generator).unwrap();

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
                <span class="attach__name-name">
                    {&info.name}
                </span>
            </li>
        }
    }
}
