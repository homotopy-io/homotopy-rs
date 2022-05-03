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

pub struct AttachView;

impl Component for AttachView {
    type Message = Message;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            { for ctx.props().options.iter().map(|option| Self::view_option(ctx, option)) }
        }
    }
}

impl AttachView {
    pub fn view_option(ctx: &Context<Self>, option: &AttachOption) -> Html {
        let info = ctx
            .props()
            .signature
            .generator_info(option.generator)
            .unwrap();

        let onclick = ctx.props().dispatch.reform({
            let option = option.clone();
            move |_| Action::Attach(option.clone())
        });

        let onmouseenter = ctx.props().dispatch.reform({
            let option = option.clone();
            move |_| Action::HighlightAttachment(Some(option.clone()))
        });

        let onmouseleave = ctx
            .props()
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
                    {
                        if option.inverse {
                            format!("{} (inverse)", info.name)
                        } else {
                            info.name.clone()
                        }
                    }
                </span>
            </li>
        }
    }
}
