use homotopy_common::idx::Idx;
use homotopy_common::tree::{Node, Tree};

use crate::components::icon::{Icon, IconSize};
use crate::model::proof::{Action, SignatureEdit, SignatureItem};

use super::generator::GeneratorView;

use yew::prelude::*;
use yew_macro::function_component;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub dispatch: Callback<Action>,
    pub contents: Tree<SignatureItem>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum DropPosition {
    Before,
    After,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum NewFolderKind {
    Root,
    Inline,
}

fn on_valid_callback<F>(props: &Props, node: Node, f: F) -> Callback<DragEvent>
where
    F: Fn(DragEvent) + 'static,
{
    let ancestors: Vec<_> = props.contents.ancestors_of(node).collect();
    Callback::from(move |e: DragEvent| {
        if e.data_transfer()
            .and_then(|dt| dt.get_data("text/plain").ok())
            .and_then(|data| data.parse().ok())
            .map_or(false, |from| !ancestors.contains(&Node::new(from)))
        {
            e.prevent_default();
            f(e);
        }
    })
}

fn on_drag_start(node: Node) -> Callback<DragEvent> {
    Callback::from(move |e: DragEvent| {
        if let Some(dt) = e.data_transfer() {
            dt.set_effect_allowed("move");
            let _result = dt.set_data("text/plain", &node.index().to_string());
        }
    })
}

fn on_drag_enter(props: &Props, node: Node) -> Callback<DragEvent> {
    on_valid_callback(props, node, |_| {})
}

fn on_drag_over(props: &Props, node: Node) -> Callback<DragEvent> {
    on_valid_callback(props, node, |e| {
        if let Some(dt) = e.data_transfer() {
            dt.set_drop_effect("move");
        }
    })
}

fn on_drop(props: &Props, node: Node, position: DropPosition) -> Callback<DragEvent> {
    if position == DropPosition::After {
        props
            .dispatch
            .reform(into_action(move |from| SignatureEdit::MoveInto(from, node)))
    } else {
        props.dispatch.reform(into_action(move |from| {
            SignatureEdit::MoveBefore(from, node)
        }))
    }
}

fn into_action<F>(f: F) -> impl Fn(DragEvent) -> Action
where
    F: Fn(Node) -> SignatureEdit,
{
    move |e| {
        e.prevent_default();
        e.data_transfer()
            .and_then(|dt| dt.get_data("text/plain").ok())
            .and_then(|data| data.parse().ok())
            .map_or(Action::Nothing, |from| {
                Action::EditSignature(f(Node::new(from)))
            })
    }
}

#[function_component(FolderView)]
pub fn folder_view(props: &Props) -> Html {
    render_children(props, props.contents.root())
}

fn render_drop_zone(props: &Props, node: Node, position: DropPosition) -> Html {
    html! {
        <li
            class="signature__dropzone"
            ondragenter={on_drag_enter(props, node)}
            ondragover={on_drag_over(props, node)}
            ondrop={on_drop(props, node, position)}
        />
    }
}

fn render_new_folder(props: &Props, node: Node, kind: NewFolderKind) -> Html {
    let new_folder = props
        .dispatch
        .reform(move |_| Action::EditSignature(SignatureEdit::NewFolder(node)));

    if kind == NewFolderKind::Inline {
        html! {
            <span
                class="signature__item-child"
                onclick={new_folder}
            >
                <Icon
                    name={"create_new_folder"}
                    size={IconSize::Icon18}
                />
            </span>
        }
    } else {
        html! {
            <span
                class="signature__item-child signature__item-fill"
                onclick={new_folder}
            >
                <Icon
                    name={"create_new_folder"}
                    size={IconSize::Icon18}
                />
            </span>
        }
    }
}

fn render_item(props: &Props, node: Node) -> Html {
    props.contents.with(node, |item| match item.inner() {
        SignatureItem::Folder(name, open) => {
            let icon = if *open { "folder_open" } else { "folder" };
            let toggle = props
                .dispatch
                .reform(move |_| Action::EditSignature(SignatureEdit::ToggleFolder(node)));

            html! {
                <div
                    class="signature__item signature__folder"
                    draggable={true.to_string()}
                    ondragover={on_drag_over(props, node)}
                    ondragenter={on_drag_enter(props, node)}
                    ondrop={on_drop(props, node, DropPosition::After)}
                    ondragstart={on_drag_start(node)}
                >
                    <span
                        class="signature__item-child"
                        onclick={toggle}
                    >
                        <Icon name={icon} size={IconSize::Icon18} />
                    </span>
                    <span class="signature__item-child signature__item-name">
                        {name}
                    </span>
                    {render_new_folder(props, node, NewFolderKind::Inline)}
                </div>
            }
        }
        SignatureItem::Item(info) => {
            html! {
                <div
                    class="signature__item"
                    draggable={true.to_string()}
                    ondragstart={on_drag_start(node)}
                >
                    <GeneratorView
                        dispatch={props.dispatch.clone()}
                        generator={info.generator}
                        info={info.clone()}
                    />
                </div>
            }
        }
    })
}

fn render_children(props: &Props, node: Node) -> Html {
    props.contents.with(node, move |n| match n.inner() {
        SignatureItem::Folder(_, true) => {
            let children = n.children().map(|child| render_tree(props, child));
            let footer = if node == props.contents.root() {
                html! {
                    <div class="signature__item">
                        {render_new_folder(props, node, NewFolderKind::Root)}
                    </div>
                }
            } else {
                html! {}
            };
            let class = format!(
                "signature__branch {}",
                if n.is_empty() {
                    "signature__branch-empty"
                } else {
                    ""
                }
            );

            html! {
                <ul class={class}>
                    {for children}
                    {render_drop_zone(props, node, DropPosition::After)}
                    {footer}
                </ul>
            }
        }
        _ => html! {},
    })
}

fn render_tree(props: &Props, node: Node) -> Html {
    html! {
        <>
            {render_drop_zone(props, node, DropPosition::Before)}
            <li>
                {render_item(props, node)}
                {render_children(props, node)}
            </li>
        </>
    }
}
