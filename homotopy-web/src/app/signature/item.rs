use std::str::FromStr;

use yew::prelude::*;
use yew_macro::function_component;

use palette::Srgb;

use homotopy_common::tree::Node;

use crate::components::icon::{Icon, IconSize};
use crate::model::proof::{Action, Color, SignatureEdit, SignatureItem, SignatureItemEdit, COLORS};

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
    Coloring,
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
    Dispatch(Action),
    Noop,
}

#[derive(Debug, Default)]
pub struct EditState {
    name: Option<String>,
    color: Option<Color>,
}

impl EditState {
    fn apply(&mut self, edit: SignatureItemEdit) -> bool {
        match edit {
            SignatureItemEdit::Rename(name) => self.name = Some(name),
            SignatureItemEdit::Recolor(color) => self.color = Some(color),
        }

        true
    }

    fn dispatch(&mut self, dispatch: &Callback<Action>, node: Node) {
        if let Some(name) = self.name.take() {
            dispatch.emit(Action::EditSignature(SignatureEdit::Edit(
                node,
                SignatureItemEdit::Rename(name),
            )));
        }

        if let Some(color) = self.color.take() {
            dispatch.emit(Action::EditSignature(SignatureEdit::Edit(
                node,
                SignatureItemEdit::Recolor(color),
            )));
        }
    }
}

#[derive(Debug)]
pub struct ItemView {
    props: ItemViewProps,
    link: ComponentLink<Self>,
    mode: ItemViewMode,
    edit: EditState,
}

impl Component for ItemView {
    type Properties = ItemViewProps;
    type Message = ItemViewMessage;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            mode: Default::default(),
            edit: Default::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            ItemViewMessage::SwitchTo(mode) => return self.switch_to(mode),
            ItemViewMessage::Edit(edit) => return self.edit.apply(edit),
            ItemViewMessage::Dispatch(dispatch) => self.props.dispatch.emit(dispatch),
            ItemViewMessage::Noop => {}
        }

        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        props != self.props && {
            self.props = props;
            true
        }
    }

    fn view(&self) -> Html {
        let class = format!(
            "signature__item {}",
            if self.mode == ItemViewMode::Coloring {
                "signature__item-coloring"
            } else {
                ""
            },
        );

        let picker = if let SignatureItem::Item(info) = &self.props.item {
            self.view_picker(&info.color)
        } else {
            html! {}
        };

        html! {
            <div
                class={class}
                draggable={(self.mode == ItemViewMode::Viewing).to_string()}
                ondragover={self.props.on_drag_over.clone()}
                ondragenter={self.props.on_drag_enter.clone()}
                ondrop={self.props.on_drop.clone()}
                ondragstart={self.props.on_drag_start.clone()}
            >
                <div class="signature__item-info">
                    {self.view_info()}
                    {self.view_buttons()}
                </div>
                {picker}
            </div>
        }
    }
}

impl ItemView {
    fn switch_to(&mut self, mode: ItemViewMode) -> bool {
        if mode == self.mode {
            return false;
        }

        if ItemViewMode::Viewing == mode {
            self.edit.dispatch(&self.props.dispatch, self.props.node);
        }

        self.mode = mode;
        true
    }

    fn view_name(&self) -> Html {
        let name = match &self.props.item {
            SignatureItem::Item(info) => &info.name,
            SignatureItem::Folder(name, _) => name,
        };
        let on_click = match &self.props.item {
            SignatureItem::Item(info) => {
                let generator = info.generator;
                self.props
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
                    value={self.edit.name.clone().unwrap_or_else(|| name.clone())}
                    oninput={self.link.callback(move |e: InputData| {
                        ItemViewMessage::Edit(SignatureItemEdit::Rename(e.value))
                    })}
                    onkeyup={self.link.callback(move |e: KeyboardEvent| {
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

    fn view_color(&self, color: &Color) -> Html {
        let style = format!(
            "background: {}",
            self.edit.color.clone().unwrap_or_else(|| color.clone())
        );

        match self.mode {
            ItemViewMode::Viewing => html! {
                <span
                    class={"signature__item-child signature__generator-color"}
                    style={style}
                />
            },
            ItemViewMode::Editing => {
                let recolor = self
                    .link
                    .callback(|_| ItemViewMessage::SwitchTo(ItemViewMode::Coloring));

                html! {
                    <ItemViewButton
                        class={"signature__generator-color"}
                        icon={"palette"}
                        on_click={recolor}
                        style={style}
                    />
                }
            }
            ItemViewMode::Coloring => {
                let apply = self
                    .link
                    .callback(|_| ItemViewMessage::SwitchTo(ItemViewMode::Editing));

                html! {
                    <ItemViewButton
                        class={"signature__generator-color"}
                        icon={"done"}
                        on_click={apply}
                        style={style}
                    />
                }
            }
        }
    }

    fn view_picker(&self, color: &Color) -> Html {
        if self.mode != ItemViewMode::Coloring {
            return html! {};
        }

        let buttons = COLORS.iter().map(|color| {
            let recolor = self.link.callback(move |_| {
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
        let color = self.edit.color.clone().unwrap_or_else(|| color.clone());
        let custom_recolor = self.link.callback(move |e: InputData| {
            ItemViewMessage::Edit(SignatureItemEdit::Recolor(Color(
                Srgb::<u8>::from_str(&e.value).unwrap(),
            )))
        });

        html! {
            <div class="signature__generator-picker">
                {for buttons}
                <input
                    class="signature__generator-picker-custom"
                    value={color.to_string()}
                    type="color"
                    oninput={custom_recolor}
                />
            </div>
        }
    }

    fn view_info(&self) -> Html {
        match &self.props.item {
            SignatureItem::Item(info) => {
                html! {
                    <>
                        {self.view_color(&info.color)}
                        {self.view_name()}
                        <span class="signature__item-child">
                            {info.diagram.dimension()}
                        </span>
                    </>
                }
            }
            SignatureItem::Folder(_, open) => {
                let icon = if *open { "folder_open" } else { "folder" };
                let node = self.props.node;
                let toggle = self.link.callback(move |_| {
                    ItemViewMessage::Dispatch(Action::EditSignature(SignatureEdit::ToggleFolder(
                        node,
                    )))
                });

                html! {
                    <>
                        <ItemViewButton icon={icon} on_click={toggle} />
                        {self.view_name()}
                    </>
                }
            }
        }
    }

    fn view_buttons(&self) -> Html {
        if self.mode == ItemViewMode::Viewing {
            let new_folder = if let SignatureItem::Folder(_, true) = self.props.item {
                html! {
                    <NewFolderButton
                        dispatch={self.props.dispatch.clone()}
                        node={self.props.node}
                        kind={NewFolderKind::Inline}
                    />
                }
            } else {
                html! {}
            };

            html! {
                <>
                    {new_folder}
                    <ItemViewButton icon={"edit"} on_click={
                        self.link.callback(move |_| {
                            ItemViewMessage::SwitchTo(ItemViewMode::Editing)
                        })
                    } />
                </>
            }
        } else {
            let node = self.props.node;

            html! {
                <>
                    <ItemViewButton icon={"delete"} on_click={
                        self.props.dispatch.reform(
                            move |_| Action::EditSignature(SignatureEdit::Remove(node))
                        )
                    } />
                    <ItemViewButton icon={"done"} on_click={
                        self.link.callback(move |_| {
                            ItemViewMessage::SwitchTo(ItemViewMode::Viewing)
                        })
                    } />
                </>
            }
        }
    }
}
