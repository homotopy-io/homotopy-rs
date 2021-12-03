use homotopy_common::{
    idx::Idx,
    tree::{Node, Tree},
};
use yew::prelude::*;

use super::item::{ItemView, NewFolderButton, NewFolderKind};
use crate::{
    components::{add_class, remove_class},
    model::proof::{Action, SignatureEdit, SignatureItem},
};

#[derive(Copy, Clone, PartialEq, Eq)]
enum DropPosition {
    Before,
    After,
}

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
        e.prevent_default();
        if e.data_transfer()
            .and_then(|dt| dt.get_data("text").ok())
            .and_then(|data| data.parse().ok())
            .map_or(false, |from| !ancestors.contains(&Node::new(from)))
        {
            f(e);
        }
    })
}

fn on_drag_start(node: Node) -> Callback<DragEvent> {
    Callback::from(move |e: DragEvent| {
        if let Some(dt) = e.data_transfer() {
            dt.set_effect_allowed("move");
            dt.set_data("text", &node.index().to_string()).unwrap();
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

// NOTE this looks like it could be a function component but it can't.
// Making this a function component means that it doesn't re-render
// as the signature is incrementally modified.
pub struct FolderView {}

impl Component for FolderView {
    type Message = ();
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        Self {}
    }

    fn changed(&mut self, _: &Context<Self>) -> bool {
        // This is the critical line.
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        render_children(ctx.props(), ctx.props().contents.root())
    }
}

fn render_drop_zone(props: &Props, node: Node, position: DropPosition) -> Html {
    let drop_zone_ref = NodeRef::default();
    let on_drag_enter = {
        let drop_zone_ref = drop_zone_ref.clone();
        on_drag_enter(props, node).reform(move |e| {
            add_class(&drop_zone_ref, "drag-over");
            e
        })
    };
    let on_drag_leave = {
        let drop_zone_ref = drop_zone_ref.clone();
        Callback::from(move |_| {
            remove_class(&drop_zone_ref, "drag-over");
        })
    };
    let on_drop = {
        let drop_zone_ref = drop_zone_ref.clone();
        on_drop(props, node, position).reform(move |e| {
            remove_class(&drop_zone_ref, "drag-over");
            e
        })
    };

    html! {
        <li
            ref={drop_zone_ref}
            class="signature__dropzone"
            ondragenter={on_drag_enter}
            ondragleave={on_drag_leave}
            ondragover={on_drag_over(props, node)}
            ondrop={on_drop}
        />
    }
}

fn render_item(props: &Props, node: Node) -> Html {
    props.contents.with(node, |item| match item.inner() {
        SignatureItem::Folder(_, _) => {
            html! {
                <ItemView
                    dispatch={props.dispatch.clone()}
                    node={node}
                    item={item.inner().clone()}
                    on_drag_over={on_drag_over(props, node)}
                    on_drag_enter={on_drag_enter(props, node)}
                    on_drop={on_drop(props, node, DropPosition::After)}
                    on_drag_start={on_drag_start(node)}
                />
            }
        }
        SignatureItem::Item(_) => {
            html! {
                <ItemView
                    dispatch={props.dispatch.clone()}
                    node={node}
                    item={item.inner().clone()}
                    on_drag_start={on_drag_start(node)}
                />
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
                    <NewFolderButton
                        dispatch={props.dispatch.clone()}
                        node={props.contents.root()}
                        kind={NewFolderKind::Root}
                    />
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
