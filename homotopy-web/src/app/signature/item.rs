use std::str::FromStr;

use homotopy_common::tree::Node;
use palette::Srgb;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement};
use yew::prelude::*;
use yew_macro::function_component;

use crate::{
    components::icon::{Icon, IconSize},
    model::proof::{
        generators::{Color, GeneratorInfo, VertexShape},
        Action, SignatureEdit, SignatureItem, SignatureItemEdit, COLORS, VERTEX_SHAPES,
    },
};

mod preference;
use preference::GeneratorPreferenceCheckbox;

// FIXME(@doctorn)
//
// When deleting signature items, at the moment, `ItemView` components
// retain their state.
//
// In order to fix this, I think it is necessary to maintain a map from nodes
// in the signature to their current `ItemView` state, but this is a big change
// and I can't see any urgency.
//
// An alternative solution would be to prevent more than one signature item being
// edited concurrently and simply reset all `ItemView` states to `Viewing` after
// an edit.

#[derive(Properties, Debug, Clone, PartialEq)]
struct ItemViewButtonProps {
    icon: String,
    on_click: Callback<MouseEvent>,
    #[prop_or_default]
    fill: bool,
    #[prop_or_default]
    class: String,
    #[prop_or_default]
    style: String,
}

#[function_component(ItemViewButton)]
fn item_view_button(props: &ItemViewButtonProps) -> Html {
    let class = format!(
        "signature__item-child {} {}",
        if props.fill {
            "signature__item-fill"
        } else {
            ""
        },
        props.class,
    );

    html! {
        <span
            class={class}
            style={props.style.clone()}
            onclick={props.on_click.clone()}
        >
            <Icon name={props.icon.clone()} size={IconSize::Icon18} />
        </span>
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NewFolderKind {
    Root,
    Inline,
}

#[derive(Properties, Debug, Clone, PartialEq)]
pub struct NewFolderButtonProps {
    pub dispatch: Callback<Action>,
    pub kind: NewFolderKind,
    pub node: Node,
}

#[function_component(NewFolderButton)]
pub fn new_folder_button(props: &NewFolderButtonProps) -> Html {
    let node = props.node;
    let on_click = props
        .dispatch
        .reform(move |_| Action::EditSignature(SignatureEdit::NewFolder(node)));

    html! {
        <ItemViewButton
            icon={"create_new_folder"}
            on_click={on_click}
            fill={props.kind == NewFolderKind::Root}
        />
    }
}

#[derive(Properties, Debug, Clone, PartialEq)]
pub struct ItemViewProps {
    pub dispatch: Callback<Action>,
    pub node: Node,
    pub item: SignatureItem,

    #[prop_or_default]
    pub on_drag_over: Callback<DragEvent>,
    #[prop_or_default]
    pub on_drag_enter: Callback<DragEvent>,
    #[prop_or_default]
    pub on_drop: Callback<DragEvent>,
    #[prop_or_default]
    pub on_drag_start: Callback<DragEvent>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ItemViewMode {
    Viewing,
    Editing,
}

impl Default for ItemViewMode {
    fn default() -> Self {
        Self::Viewing
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemViewMessage {
    SwitchTo(ItemViewMode),
    Edit(SignatureItemEdit),
    Noop,
}

fn apply_edit(dispatch: &Callback<Action>, node: Node, edit: SignatureItemEdit) -> bool {
    dispatch.emit(Action::EditSignature(SignatureEdit::Edit(node, edit)));
    true
}

#[derive(Properties, Debug, Clone, PartialEq)]
struct CustomRecolorButtonProps {
    oninput: Callback<InputEvent>,
    onkeyup: Callback<KeyboardEvent>,
    value: Color,
    #[prop_or_default]
    class: String,
}

#[function_component(CustomRecolorButton)]
fn custom_recolor_button(props: &CustomRecolorButtonProps) -> Html {
    let hex = format!("#{:X}", props.value.0);
    html! {
        <div class={"signature__generator-picker-custom-wrapper"}>
            <div class={"signature__generator-picker-custom-flex"}>
                <input
                    class={"signature__generator-picker-custom"}
                    type={"color"}
                    oninput={props.oninput.clone()}
                    value={props.value.to_string()}
                />
                <div class={"signature__generator-picker-custom-hex"}>
                    <input
                        class={"signature__generator-picker-custom-hex-input"}
                        type="text"
                        oninput={props.oninput.clone()}
                        onkeyup={props.onkeyup.clone()}
                        value={hex}
                    />
                </div>
            </div>
            <div class={"signature__generator-picker-custom-inner"}>
                <Icon name={"palette"} size={IconSize::Icon18} />
            </div>
        </div>
    }
}

#[derive(Debug)]
pub struct ItemView {
    mode: ItemViewMode,
}

impl Component for ItemView {
    type Message = ItemViewMessage;
    type Properties = ItemViewProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            mode: Default::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ItemViewMessage::SwitchTo(mode) => return self.switch_to(mode),
            ItemViewMessage::Edit(edit) => {
                return apply_edit(&ctx.props().dispatch, ctx.props().node, edit)
            }
            ItemViewMessage::Noop => {}
        }

        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let item = &ctx.props().item;

        let class = format!(
            "signature__item {}",
            match (self.mode, item) {
                (ItemViewMode::Editing, SignatureItem::Item(info)) => format!(
                    "signature__item-editing signature__item-generator-{}",
                    info.generator.dimension
                ),
                (ItemViewMode::Editing, _) => "signature__item-editing".to_owned(),
                (_, _) => "".to_owned(),
            }
        );

        let picker_and_prefs = if let SignatureItem::Item(info) = item {
            html! {
                <>
                    {self.view_preferences(ctx, info)}
                    {self.view_picker(ctx, info)}
                </>
            }
        } else {
            html! {}
        };

        html! {
            <div
                class={class}
                draggable={(self.mode == ItemViewMode::Viewing).to_string()}
                ondragover={ctx.props().on_drag_over.clone()}
                ondragenter={ctx.props().on_drag_enter.clone()}
                ondrop={ctx.props().on_drop.clone()}
                ondragstart={ctx.props().on_drag_start.clone()}
            >
                <div class="signature__item-info">
                    {self.view_info(ctx)}
                    {self.view_buttons(ctx)}
                </div>
                {picker_and_prefs}
            </div>
        }
    }
}

impl ItemView {
    fn switch_to(&mut self, mode: ItemViewMode) -> bool {
        if mode == self.mode {
            return false;
        }

        self.mode = mode;
        true
    }

    fn view_name(&self, ctx: &Context<Self>) -> Html {
        let name = match &ctx.props().item {
            SignatureItem::Item(info) => &info.name,
            SignatureItem::Folder(name, _) => name,
        };
        let on_click = match &ctx.props().item {
            SignatureItem::Item(info) => {
                let generator = info.generator;
                ctx.props()
                    .dispatch
                    .reform(move |_| Action::SelectGenerator(generator))
            }
            SignatureItem::Folder(_, _) => Callback::noop(),
        };

        if self.mode == ItemViewMode::Viewing {
            html! {
                <span
                    class="signature__item-child signature__item-name"
                    onclick={on_click}
                >
                    {name}
                </span>
            }
        } else {
            html! {
                <input
                    type="text"
                    class="signature__item-name-input"
                    value={name.clone()}
                    oninput={ctx.link().callback(move |e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        ItemViewMessage::Edit(SignatureItemEdit::Rename(input.value()))
                    })}
                    onkeyup={ctx.link().callback(move |e: KeyboardEvent| {
                        e.stop_propagation();
                        if e.key().to_ascii_lowercase() == "enter" {
                            ItemViewMessage::SwitchTo(ItemViewMode::Viewing)
                        } else {
                            ItemViewMessage::Noop
                        }
                    })}
                />
            }
        }
    }

    fn view_color(color: &Color) -> Html {
        let style = format!("background: {}", color.clone());

        html! {
            <span
                class={"signature__item-child signature__generator-color"}
                style={style}
            />
        }
    }

    fn view_picker(&self, ctx: &Context<Self>, info: &GeneratorInfo) -> Html {
        if self.mode != ItemViewMode::Editing {
            return html! {};
        }

        let selected_color = info.color.clone();
        let color_preset_buttons = COLORS.iter().map(|color| {
            let recolor = ctx.link().callback(move |_| {
                ItemViewMessage::Edit(SignatureItemEdit::Recolor(Color(
                    Srgb::<u8>::from_str(color).unwrap(),
                )))
            });

            html! {
                <div
                    class="signature__generator-picker-preset"
                    style={format!("background: {}", color)}
                    onclick={recolor}
                />
            }
        });
        let custom_recolor = ctx.link().callback(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            e.stop_propagation();
            // The below ensures that the user's typing isn't overwritten by reactive code.
            if input.type_() == "text" && input.value().len() < 6 {
                return ItemViewMessage::Noop;
            }
            if let Ok(srgb) = Srgb::<u8>::from_str(&input.value()) {
                ItemViewMessage::Edit(SignatureItemEdit::Recolor(Color(srgb)))
            } else {
                ItemViewMessage::Noop
            }
        });

        let selected_shape = info.shape.clone();
        let shape_preset_buttons = VERTEX_SHAPES.iter().map(|shape| {
            let reshape = ctx.link().callback(move |_| {
                ItemViewMessage::Edit(SignatureItemEdit::Reshape(shape.clone()))
            });
            let icon_name = match shape {
                VertexShape::Circle => "circle",
                VertexShape::Square => "square",
            };
            let mut class = "signature__generator-picker-preset".to_owned();
            if *shape == selected_shape {
                class += " signature__generator-picker-preset-shape-selected";
            }

            html! {
                <div {class} onclick={reshape}>
                    <Icon name={icon_name} size={IconSize::Icon18} />
                </div>
            }
        });

        html! {
            <>
                <div class="signature__generator-picker signature__generator-picker-shape">
                    {for shape_preset_buttons}
                </div>
                <div class="signature__generator-picker signature__generator-picker-color">
                    {for color_preset_buttons}
                    <CustomRecolorButton
                        value={selected_color}
                        oninput={custom_recolor}
                        onkeyup={ctx.link().callback(move |e: KeyboardEvent| {
                            e.stop_propagation();
                            ItemViewMessage::Noop
                        })}
                    />
                </div>
            </>
        }
    }

    fn view_info(&self, ctx: &Context<Self>) -> Html {
        match &ctx.props().item {
            SignatureItem::Item(info) => {
                html! {
                    <>
                        {Self::view_color(&info.color)}
                        {self.view_name(ctx)}
                        <span class="signature__item-child">
                            {info.diagram.dimension()}
                        </span>
                    </>
                }
            }
            SignatureItem::Folder(_, open) => {
                let icon = if *open { "folder_open" } else { "folder" };
                let node = ctx.props().node;
                let toggle = ctx
                    .props()
                    .dispatch
                    .reform(move |_| Action::EditSignature(SignatureEdit::ToggleFolder(node)));

                html! {
                    <>
                        <ItemViewButton icon={icon} on_click={toggle} />
                        {self.view_name(ctx)}
                    </>
                }
            }
        }
    }

    fn view_buttons(&self, ctx: &Context<Self>) -> Html {
        if self.mode == ItemViewMode::Viewing {
            let new_folder = if let SignatureItem::Folder(_, true) = ctx.props().item {
                html! {
                    <NewFolderButton
                        dispatch={ctx.props().dispatch.clone()}
                        node={ctx.props().node}
                        kind={NewFolderKind::Inline}
                    />
                }
            } else {
                html! {}
            };

            html! {
                <>
                    {new_folder}
                    <ItemViewButton icon={"settings"} on_click={
                        ctx.link().callback(move |_| {
                            ItemViewMessage::SwitchTo(ItemViewMode::Editing)
                        })
                    } />
                </>
            }
        } else {
            let node = ctx.props().node;

            html! {
                <>
                    <ItemViewButton icon={"delete"} on_click={
                        ctx.props().dispatch.reform(
                            move |_| Action::EditSignature(SignatureEdit::Remove(node))
                        )
                    } />
                    <ItemViewButton icon={"done"} on_click={
                        ctx.link().callback(move |_| {
                            ItemViewMessage::SwitchTo(ItemViewMode::Viewing)
                        })
                    } />
                </>
            }
        }
    }

    fn view_preferences(&self, ctx: &Context<Self>, info: &GeneratorInfo) -> Html {
        if self.mode != ItemViewMode::Editing {
            return html! {};
        }

        // The following is required to allow the div to respond to onclick events appropriately.
        let toggle = ctx.link().callback(move |e: MouseEvent| {
            let div: HtmlElement = e.target_unchecked_into();
            let input: HtmlInputElement = div.last_element_child().unwrap().unchecked_into();
            let invertible = !input.checked();
            ItemViewMessage::Edit(SignatureItemEdit::Invertibility(invertible))
        });

        let dimension = info.generator.dimension;

        let invertible_checkbox = if dimension > 0 {
            html! {
                <GeneratorPreferenceCheckbox
                    name={"Invertible:"}
                    onclick={toggle}
                    checked={info.invertible}
                />
            }
        } else {
            html! {}
        };

        html! {
            <>
                {invertible_checkbox}
            </>
        }
    }
}
