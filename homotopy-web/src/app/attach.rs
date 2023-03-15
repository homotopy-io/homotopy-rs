use homotopy_core::{common::Generator, signature::Signature as S};
use yew::prelude::*;

use crate::{
    app::tex::TexSpan,
    model::{
        proof::{self, AttachOption, Signature},
        Action, Selectables,
    },
};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    pub dispatch: Callback<Action>,
    pub options: Selectables,
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
        match &ctx.props().options {
            Selectables::Attach(att) => html! {
                { for att.iter().map(|option| Self::view_attach_option(ctx, option)) }
            },
            Selectables::Merge(from, tos) => html! {
                { for tos.iter().map(|&to| Self::view_merge_option(ctx, *from, to)) }
            },
        }
    }
}

impl AttachView {
    pub fn view_attach_option(ctx: &Context<Self>, option: &AttachOption) -> Html {
        let info = ctx
            .props()
            .signature
            .generator_info(option.generator)
            .unwrap();

        let onclick = ctx.props().dispatch.reform({
            let option = option.clone();
            move |_| Action::Proof(proof::Action::Attach(option.clone()))
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
                <TexSpan
                    class="attach__option-name"
                    error_color="#c004"
                    raw_tex={
                        format!("{}{}", info.name, option.tag.as_ref().map_or(Default::default(), |t| format!(" ({t})")))
                    }
                />
            </li>
        }
    }

    pub fn view_merge_option(ctx: &Context<Self>, from: Generator, to: Generator) -> Html {
        let info = ctx.props().signature.generator_info(to).unwrap();

        let onclick = ctx
            .props()
            .dispatch
            .reform(move |_| Action::Proof(proof::Action::Merge(from, to)));

        html! {
            <li
                class="attach__option"
                onclick={onclick}
            >
                <span
                    class="attach__option-color"
                    style={format!("background: {}", info.color)}
                />
                <TexSpan
                    class="attach__option-name"
                    error_color="#c004"
                    raw_tex={
                        format!("{}", info.name)
                    }
                />
            </li>
        }
    }
}
