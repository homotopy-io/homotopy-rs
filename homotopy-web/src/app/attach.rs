use homotopy_core::{common::Generator, signature::Signature as _};
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

#[function_component]
pub fn AttachView(props: &Props) -> Html {
    match &props.options {
        Selectables::Attach(att) => html! {
            { for att.iter().map(|option| view_attach_option(props, option)) }
        },
        Selectables::Merge(from, tos) => html! {
            { for tos.iter().map(|to| view_merge_option(props, *from, *to)) }
        },
    }
}

fn view_attach_option(props: &Props, option: &AttachOption) -> Html {
    let info = props.signature.generator_info(option.generator).unwrap();

    let onclick = props.dispatch.reform({
        let option = option.clone();
        move |_| Action::Proof(proof::Action::Attach(option.clone()))
    });

    let onmouseenter = props.dispatch.reform({
        let option = option.clone();
        move |_| Action::HighlightAttachment(Some(option.clone()))
    });

    let onmouseleave = props.dispatch.reform(|_| Action::HighlightAttachment(None));

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
                    format!("{}{}", info.name, option.tag.as_ref().map_or_else(Default::default, |t| format!(" ({t})")))
                }
            />
        </li>
    }
}

fn view_merge_option(props: &Props, from: Generator, to: Generator) -> Html {
    let info = props.signature.generator_info(to).unwrap();

    let onclick = props
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
