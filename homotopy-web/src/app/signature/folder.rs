use homotopy_common::idx::Idx;
use homotopy_common::tree::{Tree, Node};

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
            f(e)
        }
    })
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
    let root = props.contents.root();
    let inner: Html = props.contents.with(root, |r| {
        r.children().map(|c| render_tree(props, c)).collect()
    });
    let ondragenter_end = Callback::from(|e: DragEvent| e.prevent_default());
    let ondragover_end = Callback::from(|e: DragEvent| {
        e.prevent_default();
        if let Some(dt) = e.data_transfer() {
            dt.set_drop_effect("move");
        }
    });
    let ondrop_end = props
        .dispatch
        .reform(into_action(move |from| SignatureEdit::MoveInto(from, root)));

    html! {
        <>
            <ul class="signature__generators">
                {inner}
                <li
                    class="signature__dropzone"
                    ondragenter={ondragenter_end}
                    ondragover={ondragover_end}
                    ondrop={ondrop_end}
                />
            </ul>
            <span onclick={props.dispatch.reform(|_| Action::EditSignature(SignatureEdit::NewFolder))}>
                <Icon name={"create_new_folder"} size={IconSize::Icon18} />
            </span>
        </>
    }
}

fn render_item(props: &Props, node: Node) -> Html {
    props.contents.with(node, |item| {
        let ondragenter = on_valid_callback(props, node, |_| {});
        let ondragenter_before = ondragenter.clone();
        let ondragover = on_valid_callback(props, node, |e| {
            if let Some(dt) = e.data_transfer() {
                dt.set_drop_effect("move");
            }
        });
        let ondragover_before = ondragover.clone();
        let ondrop = |n: Node| props.dispatch.reform(into_action(move |from| SignatureEdit::MoveInto(from, n)));
        let ondrop_before = |n: Node| props.dispatch.reform(into_action(move |from| SignatureEdit::MoveBefore(from, n)));
        let ondragstart = |n: Node| {
            Callback::from(move |e: DragEvent| {
                if let Some(dt) = e.data_transfer() {
                    dt.set_effect_allowed("move");
                    let _result = dt.set_data("text/plain", &n.index().to_string());
                }
            })
        };

        let element = match item.inner() {
            SignatureItem::Folder(name) => {
                html! {
                    <li
                        class="signature__folder"
                        draggable={true.to_string()}
                        ondragover={ondragover}
                        ondragenter={ondragenter}
                        ondrop={ondrop(node)}
                        ondragstart={ondragstart(node)}
                    >
                        <Icon name={"folder"} size={IconSize::Icon18} />
                        {name}
                    </li>
                }
            }
            SignatureItem::Item(info) => {
                html! {
                    <li
                        class="signature__generator"
                        draggable={true.to_string()}
                        ondragstart={ondragstart(node)}
                    >
                        <GeneratorView
                            dispatch={props.dispatch.clone()}
                            generator={info.generator}
                            info={info.clone()}
                        />
                    </li>
                }
            }
        };

        html! {
            <>
                <li
                    class="signature__dropzone"
                    ondragenter={ondragenter_before}
                    ondragover={ondragover_before}
                    ondrop={ondrop_before(node)}
                />
                {element}
            </>
        }
    })
}

fn render_tree(props: &Props, node: Node) -> Html {
    props.contents.with(node, move |n| {
        let ondragenter_end = on_valid_callback(props, node, |_| {});
        let ondragover_end = on_valid_callback(props, node, |e| {
            if let Some(dt) = e.data_transfer() {
                dt.set_drop_effect("move");
            }
        });
        let ondrop_end = props
            .dispatch
            .reform(into_action(move |from| SignatureEdit::MoveInto(from, node)));

        let children: Html = n
            .children()
            .map(|child| {
                html! {
                    <ul>
                        {render_tree(props, child)}
                        <li
                            class="signature__dropzone"
                            ondragenter={ondragenter_end.clone()}
                            ondragover={ondragover_end.clone()}
                            ondrop={ondrop_end.clone()}
                        />
                    </ul>
                }
            })
            .collect();

        html! {
            <>
                {render_item(props, node)}
                {children}
            </>
        }
    })
}
