use std::str::FromStr;

use homotopy_common::tree::Node;
use homotopy_core::Diagram;
use homotopy_graphics::style::{Color, VertexShape};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlInputElement};
use yew::prelude::*;
use yew_macro::function_component;

use crate::{
    app::{diagram_svg::DiagramSvg, sidebar::DrawerViewSize, AppSettings, AppSettingsKey},
    components::{
        icon::{Icon, IconSize},
        settings::{KeyStore, Settings, Store},
    },
    model::proof::{
        generators::GeneratorInfo, Action, Signature, SignatureEdit, SignatureItem,
        SignatureItemEdit, COLORS, VERTEX_SHAPES,
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
    #[prop_or_default]
    light: bool,
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
            <Icon name={props.icon.clone()} size={IconSize::Icon18} light={props.light} />
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
    pub signature: Signature,
    pub drawer_view_size: DrawerViewSize,

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
    Hovering,
    Editing,
}

impl Default for ItemViewMode {
    fn default() -> Self {
        Self::Viewing
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Preview {
    pub signature: Signature,
    pub html: Html,
}

#[derive(Debug, Clone)]
pub enum ItemViewMessage {
    SwitchTo(ItemViewMode),
    Edit(SignatureItemEdit),
    CachePreview(bool, Diagram),
    Setting(<Store<AppSettings> as KeyStore>::Message),
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
}

#[function_component(CustomRecolorButton)]
fn custom_recolor_button(props: &CustomRecolorButtonProps) -> Html {
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
                        value={props.value.to_string()}
                    />
                </div>
            </div>
            <div class="signature__generator-picker-custom-inner">
                <Icon name={"palette"} size={IconSize::Icon18} light={!props.value.is_light()} />
            </div>
        </div>
    }
}

pub struct ItemView {
    local: Store<AppSettings>,
    mode: ItemViewMode,
    name: String,
    preview_cache: Option<Preview>,
    _settings: AppSettings,
}

impl Component for ItemView {
    type Message = ItemViewMessage;
    type Properties = ItemViewProps;

    fn create(ctx: &Context<Self>) -> Self {
        const ITEM_SUBSCRIPTIONS: &[AppSettingsKey] = &[AppSettingsKey::show_previews];

        let mut settings = AppSettings::connect(ctx.link().callback(ItemViewMessage::Setting));
        settings.subscribe(ITEM_SUBSCRIPTIONS);

        let name = match &ctx.props().item {
            SignatureItem::Item(info) => info.name.clone(),
            SignatureItem::Folder(info) => info.name.clone(),
        };
        Self {
            local: Default::default(),
            mode: Default::default(),
            name,
            preview_cache: Default::default(),
            _settings: settings,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ItemViewMessage::SwitchTo(mode) => return self.switch_to(ctx, mode),
            ItemViewMessage::Edit(edit) => {
                // In order to avoid generating multiple history events for a single rename, we
                // don't dispatch renames until the user is done editing.
                if let SignatureItemEdit::Rename(name) = edit {
                    self.name = name;
                    return true;
                };

                return apply_edit(&ctx.props().dispatch, ctx.props().node, edit);
            }
            ItemViewMessage::CachePreview(show_single_preview, diagram) => {
                self.cache_preview(ctx, show_single_preview, &diagram);
                return true;
            }
            ItemViewMessage::Setting(msg) => {
                self.local.set(&msg);
                return true;
            }
            ItemViewMessage::Noop => {}
        }

        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let item = &ctx.props().item;

        let class = format!(
            "signature__item signature__{item} signature__{item}-{mode} {generator_dimension}",
            item = match item {
                SignatureItem::Item(_) => "generator",
                SignatureItem::Folder(_) => "folder",
            },
            mode = match self.mode {
                ItemViewMode::Editing => "editing",
                ItemViewMode::Hovering => "hovering",
                ItemViewMode::Viewing => "viewing",
            },
            generator_dimension = match item {
                SignatureItem::Item(info) =>
                    format!("signature__generator-{}d", info.generator.dimension),
                SignatureItem::Folder(_) => "".to_owned(),
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

        let on_mouse_over = if self.mode == ItemViewMode::Viewing {
            ctx.link()
                .callback(|_| ItemViewMessage::SwitchTo(ItemViewMode::Hovering))
        } else {
            ctx.link().callback(|_| ItemViewMessage::Noop)
        };
        let on_mouse_out = if self.mode == ItemViewMode::Hovering {
            ctx.link()
                .callback(|_| ItemViewMessage::SwitchTo(ItemViewMode::Viewing))
        } else {
            ctx.link().callback(|_| ItemViewMessage::Noop)
        };

        let select_generator = match (self.mode, &ctx.props().item) {
            (ItemViewMode::Viewing | ItemViewMode::Hovering, SignatureItem::Item(info)) => {
                let generator = info.generator;
                ctx.props()
                    .dispatch
                    .reform(move |_| Action::SelectGenerator(generator))
            }
            _ => Callback::noop(),
        };

        html! {
            <div
                class={class}
                draggable={(self.mode != ItemViewMode::Editing).to_string()}
                ondragover={ctx.props().on_drag_over.clone()}
                ondragenter={ctx.props().on_drag_enter.clone()}
                ondrop={ctx.props().on_drop.clone()}
                ondragstart={ctx.props().on_drag_start.clone()}
                onmouseover={on_mouse_over}
                onmouseout={on_mouse_out}
            >
                {Self::view_sliver(ctx)}
                <div class="signature__item-contents" onclick={select_generator}>
                    <div class="signature__item-info">
                        {self.view_left_buttons(ctx)}
                        {self.view_info(ctx)}
                        {self.view_property_indicators(ctx)}
                        {self.view_preview(ctx)}
                        {self.view_right_buttons(ctx)}
                    </div>
                    {picker_and_prefs}
                </div>
            </div>
        }
    }
}

impl ItemView {
    fn switch_to(&mut self, ctx: &Context<Self>, mode: ItemViewMode) -> bool {
        if mode == self.mode {
            return false;
        }

        // Apply rename if name has changed.
        if mode == ItemViewMode::Viewing {
            let prev_name = match &ctx.props().item {
                SignatureItem::Item(info) => &info.name,
                SignatureItem::Folder(info) => &info.name,
            };
            if &self.name != prev_name {
                apply_edit(
                    &ctx.props().dispatch,
                    ctx.props().node,
                    SignatureItemEdit::Rename(self.name.clone()),
                );
            }
        }

        self.mode = mode;
        true
    }

    fn view_name(&self, ctx: &Context<Self>) -> Html {
        let name = match &ctx.props().item {
            SignatureItem::Item(info) => &info.name,
            SignatureItem::Folder(info) => &info.name,
        };

        if self.mode == ItemViewMode::Editing {
            html! {
                <input
                    type="text"
                    class="signature__item-name-input"
                    value={self.name.clone()}
                    oninput={ctx.link().callback(|e: InputEvent| {
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
        } else {
            html! {
                <div class="signature__item-child signature__item-name">
                    <span class="signature__item-name">
                        {name}
                    </span>
                </div>
            }
        }
    }

    fn view_sliver(ctx: &Context<Self>) -> Html {
        if let SignatureItem::Item(ref info) = ctx.props().item {
            let style = format!("background-color: {}", info.color.hex());
            let class = format!(
                "signature__generator-color-sliver {}",
                if info.color.is_light() {
                    "signature__generator-color-sliver-light"
                } else {
                    ""
                }
            );

            html! {
                <div class={class} style={style}/>
            }
        } else {
            html! {}
        }
    }

    fn cache_preview(&mut self, ctx: &Context<Self>, single_preview: bool, diagram: &Diagram) {
        let svg_of = |diagram: Diagram, id: String| match diagram.dimension() {
            0 => Self::view_diagram_svg::<0>(ctx, diagram, id),
            1 => Self::view_diagram_svg::<1>(ctx, diagram, id),
            _ => Self::view_diagram_svg::<2>(ctx, diagram, id),
        };

        let diagrams = match &diagram {
            Diagram::Diagram0(_) => {
                svg_of(diagram.clone(), "signature__generator-preview".to_owned())
            }
            Diagram::DiagramN(diagram_n) => {
                if single_preview {
                    svg_of(diagram.clone(), "signature__generator-preview".to_owned())
                } else {
                    html! {
                        <>
                        {svg_of(diagram_n.source(), "signature__generator-preview-source".to_owned())}
                        <div class="signature__generator-preview-spacer" />
                        {svg_of(diagram_n.target(), "signature__generator-preview-source".to_owned())}
                        </>
                    }
                }
            }
        };

        let preview_html = html! {
            <div class="signature__generator-previews-wrapper">
            {diagrams}
            </div>
        };

        self.preview_cache = Some(Preview {
            signature: ctx.props().signature.clone(),
            html: preview_html,
        });
    }

    fn view_preview(&self, ctx: &Context<Self>) -> Html {
        if !self.local.get_show_previews() {
            return html! {};
        }

        if let SignatureItem::Item(ref info) = ctx.props().item {
            if let Some(cache) = &self.preview_cache {
                // Note that the following is executed on every change in `ItemViewMode`, ie. if
                // the user hovers over a signature item, then this requires an entire diff on
                // signatures. I can't see an easy and always-correct way of getting around this.
                // It may well also be the case that preview caching is slower than no caching for
                // small diagrams.
                if ctx.props().signature == cache.signature {
                    return cache.html.clone();
                }
            }
            ctx.link().send_message(ItemViewMessage::CachePreview(
                info.single_preview,
                info.diagram.clone(),
            ));
        }

        html! {}
    }

    fn view_diagram_svg<const N: usize>(ctx: &Context<Self>, diagram: Diagram, id: String) -> Html {
        html! {
            <DiagramSvg<N>
                    diagram={diagram}
                    id={id}
                    signature={ctx.props().signature.clone()}
                    max_width={Some(42.0)}
                    max_height={Some(30.0)}
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
                ItemViewMessage::Edit(SignatureItemEdit::Recolor(Color::from_str(color).unwrap()))
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
            if let Ok(color) = Color::from_str(&input.value()) {
                ItemViewMessage::Edit(SignatureItemEdit::Recolor(color))
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
            let icon_class = if *shape == selected_shape {
                ""
            } else {
                "md-inactive"
            };

            html! {
                <div class="signature__generator-picker-preset" onclick={reshape}>
                    <Icon name={icon_name} size={IconSize::Icon18} class={icon_class} />
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
                            if e.key().to_ascii_lowercase() == "enter" {
                                ItemViewMessage::SwitchTo(ItemViewMode::Viewing)
                            } else {
                                ItemViewMessage::Noop
                            }
                        })}
                    />
                </div>
            </>
        }
    }

    fn view_info(&self, ctx: &Context<Self>) -> Html {
        match &ctx.props().item {
            SignatureItem::Item(_info) => self.view_name(ctx),
            SignatureItem::Folder(info) => {
                let icon = if info.open { "folder_open" } else { "folder" };
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

    fn view_left_buttons(&self, ctx: &Context<Self>) -> Html {
        use ItemViewMode::{Editing, Hovering, Viewing};
        if let SignatureItem::Item(ref info) = ctx.props().item {
            let icon_light = !info.color.is_light();
            let style = format!("background-color: {};", info.color.hex());
            let class = format!(
                "signature__generator-color {}",
                match self.mode {
                    Viewing => "",
                    Hovering => "signature__generator-color-hover",
                    Editing => "signature__generator-color-edit",
                }
            );
            let dimension_class = format!(
                "signature__item-child signature__generator-dimension {}",
                if info.color.is_light() {
                    ""
                } else {
                    "signature__generator-dimension-light"
                }
            );

            let inner = match self.mode {
                Viewing | Hovering => html! {
                    <>
                        <span class={dimension_class}>
                            {info.diagram.dimension()}
                        </span>
                        <ItemViewButton
                            icon={"settings"}
                            light={icon_light}
                            class="signature__generator-settings-btn"
                            on_click={
                                ctx.link().callback(move |e: MouseEvent| {
                                    e.stop_propagation();
                                    ItemViewMessage::SwitchTo(Editing)
                                })
                            } />
                    </>
                },
                Editing => {
                    let node = ctx.props().node;

                    html! {
                        <>
                            <ItemViewButton icon={"done"} light={icon_light} on_click={
                                ctx.link().callback(move |_| {
                                    ItemViewMessage::SwitchTo(Hovering)
                                })
                            } />
                        <ItemViewButton icon={"delete"} light={icon_light} on_click={
                            ctx.props().dispatch.reform(
                                move |_| Action::EditSignature(SignatureEdit::Remove(node))
                                )
                        } />
                        </>
                    }
                }
            };

            html! {
                <div class={class} style={style}>
                {inner}
                </div>
            }
        } else {
            html! {}
        }
    }

    fn view_right_buttons(&self, ctx: &Context<Self>) -> Html {
        if let SignatureItem::Folder(info) = &ctx.props().item {
            let node = ctx.props().node;
            let class = format!(
                "signature__folder-right {}",
                match self.mode {
                    ItemViewMode::Viewing => "",
                    ItemViewMode::Hovering => "signature__folder-right-hover",
                    ItemViewMode::Editing => "signature__folder-right-editing",
                },
            );

            let buttons = match self.mode {
                ItemViewMode::Viewing | ItemViewMode::Hovering => {
                    let new_folder = if info.open {
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

                    let settings = html! {
                        <ItemViewButton icon={"settings"} on_click={
                            ctx.link().callback(move |_| {
                                ItemViewMessage::SwitchTo(ItemViewMode::Editing)
                            })
                        } />
                    };

                    html! {
                        <>
                            {new_folder}
                            {settings}
                        </>
                    }
                }
                ItemViewMode::Editing => html! {
                    <>
                        <ItemViewButton icon={"delete"} on_click={
                            ctx.props().dispatch.reform(
                                move |_| Action::EditSignature(SignatureEdit::Remove(node))
                            )
                        } />
                        <ItemViewButton icon={"done"} on_click={
                            ctx.link().callback(move |_| {
                                ItemViewMessage::SwitchTo(ItemViewMode::Hovering)
                            })
                        } />
                    </>
                },
            };

            html! {
                <div class={class}>
                    {buttons}
                </div>
            }
        } else {
            html! {}
        }
    }

    fn view_property_indicators(&self, ctx: &Context<Self>) -> Html {
        if self.mode == ItemViewMode::Editing {
            return html! {};
        }

        if let SignatureItem::Item(ref info) = ctx.props().item {
            // To avoid unnecessary String operations, we define all classes beforehand
            let invertible_class =
                "signature__generator-indicator signature__generator-indicator-invertible";
            let oriented_class =
                "signature__generator-indicator signature__generator-indicator-oriented";

            let invertible = if info.invertible {
                html! {
                    <span class={invertible_class}>{"I"}</span>
                }
            } else {
                html! {}
            };

            let oriented = if info.oriented {
                html! {
                    <span class={oriented_class}>{"O"}</span>
                }
            } else {
                html! {}
            };

            html! {
                <div class="signature__generator-indicators-wrapper">
                    {invertible}
                    {oriented}
                </div>
            }
        } else {
            html! {}
        }
    }

    fn view_preferences(&self, ctx: &Context<Self>, info: &GeneratorInfo) -> Html {
        if self.mode != ItemViewMode::Editing {
            return html! {};
        }

        // The below macro (for convenience) dispatches a message with a `SignatureItemEdit` by
        // getting the status of the <input> within the outer <div> of a preference.
        macro_rules! toggle_or_noop {
            ($edit_type:ident) => {
                |e: MouseEvent| {
                    let input = e
                        .target_unchecked_into::<Element>()
                        .last_child()
                        .unwrap()
                        .unchecked_into::<HtmlInputElement>();

                    if input.disabled() {
                        ItemViewMessage::Noop
                    } else {
                        ItemViewMessage::Edit(SignatureItemEdit::$edit_type(!input.checked()))
                    }
                }
            };
        }

        let toggle_single_preview = ctx.link().callback(toggle_or_noop!(ShowSourceTarget));
        let toggle_invertible = ctx.link().callback(toggle_or_noop!(MakeInvertible));
        let toggle_framed = ctx.link().callback(toggle_or_noop!(MakeOriented));

        let color = if info.color.is_light() {
            "var(--drawer-foreground)".to_owned()
        } else {
            info.color.hex()
        };

        match info.generator.dimension {
            0 => Html::default(),
            _ => html! {
                <div class="signature__generator-preferences-wrapper">
                    <GeneratorPreferenceCheckbox
                        left="Single Preview"
                        right="Source-Target"
                        color={color.clone()}
                        onclick={toggle_single_preview}
                        checked={!info.single_preview}
                        disabled={!self.local.get_show_previews()}
                    />
                    <GeneratorPreferenceCheckbox
                        left="Directed"
                        right="Invertible"
                        color={color.clone()}
                        onclick={toggle_invertible}
                        checked={info.invertible}
                        disabled={info.invertible}
                    />
                    <GeneratorPreferenceCheckbox
                        left="Framed"
                        right="Oriented"
                        color={color.clone()}
                        onclick={toggle_framed}
                        checked={info.oriented}
                        disabled={info.oriented}
                    />
                </div>
            },
        }
    }
}
