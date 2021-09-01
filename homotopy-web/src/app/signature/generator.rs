use homotopy_core::Generator;

use crate::components::icon::{Icon, IconSize};
use crate::model::proof::{Action, Color, GeneratorInfo};

use palette::Srgb;
use web_sys::Element;
use yew::prelude::*;

pub struct GeneratorView {
    props: Props,
    link: ComponentLink<Self>,
    editing: Option<Edit>,
    coloring: bool,
}

#[derive(Default)]
struct Edit {
    name: Option<String>,
    color: Option<Color>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub dispatch: Callback<Action>,
    pub generator: Generator,
    pub info: GeneratorInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Done,
    DoneColor,
    Edit,
    Color,
    Rename(String),
    Recolor(Color),
    Noop,
}

impl Component for GeneratorView {
    type Message = Message;

    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            editing: Default::default(),
            coloring: Default::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::Edit => self.editing = Some(Default::default()),
            Message::Color => self.coloring = true,
            Message::Done => {
                if let Some(mut edit) = self.editing.take() {
                    if let Some(name) = edit.name.take() {
                        self.props.info.name = name;
                    }
                    if let Some(color) = edit.color.take() {
                        self.props.info.color = color;
                    }
                }
                self.coloring = false;
            }
            Message::DoneColor => {
                if let Some(edit) = self.editing.as_mut() {
                    if let Some(color) = edit.color.take() {
                        self.props.info.color = color;
                    }
                }
                self.coloring = false;
            }
            Message::Rename(name) => {
                if let Some(edit) = self.editing.as_mut() {
                    edit.name = Some(name);
                } else {
                    self.editing = Some(Edit {
                        name: Some(name),
                        ..Default::default()
                    });
                }
                return false;
            }
            Message::Recolor(color) => {
                if let Some(edit) = self.editing.as_mut() {
                    edit.color = Some(color);
                } else {
                    self.editing = Some(Edit {
                        color: Some(color),
                        ..Default::default()
                    });
                }
                return false;
            }
            Message::Noop => return false,
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props == props {
            false
        } else {
            self.props = props;
            true
        }
    }

    fn view(&self) -> Html {
        match (self.coloring, &self.editing) {
            (true, _) => self.color_generator(),
            (_, Some(_)) => self.edit_generator(),
            _ => self.view_generator(),
        }
    }
}

impl GeneratorView {
    fn color_generator(&self) -> Html {
        let color_ref = NodeRef::default();
        let color_clone = color_ref.clone();
        let picker_ref = NodeRef::default();
        let buttons = super::COLOR_RGB
            .iter()
            .map(|colors| {
                let b = colors.iter().map(|c| {
                    let (icon, picker) = (color_ref.clone(), picker_ref.clone());
                    html! {
                        <span style={format!("color:{}", Color(Srgb::new(c[0], c[1], c[2])))}
                            class="signature__item-icon"
                            onclick={self.link.callback(move |_| {
                                let color = Color(Srgb::new(c[0], c[1], c[2]));
                                let s = format!("background-color:{}; height:32px",color);
                                icon.cast::<Element>().unwrap().set_attribute("style",&s).unwrap();
                                let p = picker.cast::<Element>().unwrap();
                                p.set_attribute("type", "button").unwrap();
                                p.set_attribute("value", &color.to_string()).unwrap();
                                p.set_attribute("type", "color").unwrap();
                                Message::Recolor(Color(Srgb::new(c[0], c[1], c[2])))
                            })}
                        >
                            <Icon name={"circle"} size={IconSize::Icon18} />
                        </span>
                    }
                }
            ).collect::<Html>();
                html! {
                    <div
                        style="display: flex; flex-direction: row; justify-content: center; background-color:#f1f1f1"
                    >
                    {b}
                    </div>
                }
            })
            .collect::<Html>();

        html! {
            <div style="display: flex; flex-direction: column;">
                <div style="display: flex; flex-direction: row; flex: 1;">
                    <span
                        style={format!(
                            "color:black; background-color:{};",
                            self.editing.as_ref().unwrap().color.as_ref().map_or(&self.props.info.color, |color| color),
                        )}
                        class="signature__generator-edit"
                        onclick={self.link.callback(move |_| Message::DoneColor)}
                        ref={color_ref.clone()}
                    >
                        <Icon name={"done"} size={IconSize::Icon18} />
                    </span>
                    {self.edit_generator_name()}
                </div>
                {buttons}
                <div style="background-color:#f1f1f1">
                    <input
                        style="border:none"
                        ref={picker_ref.clone()}
                        type="color"
                        value={self.editing.as_ref().unwrap().color.as_ref().map_or(self.props.info.color.to_string(), std::string::ToString::to_string)}
                        oninput={self.link.callback(move |e: InputData| {
                            let str = e.value;
                            let r = u8::from_str_radix(&str[1..3], 16).unwrap();
                            let g = u8::from_str_radix(&str[3..5], 16).unwrap();
                            let b = u8::from_str_radix(&str[5..], 16).unwrap();
                            let s = format!("background-color:{}", str.as_str());
                            color_clone.cast::<Element>().unwrap().set_attribute("style",&s).unwrap();
                            Message::Recolor(Color(Srgb::new(r, g, b)))
                        })}
                    />
                </div>
            </div>
        }
    }

    fn edit_generator(&self) -> Html {
        html! {
            <>
                <span style={format!("color:{}", &self.props.info.color)}
                    class="signature__item-icon"
                    onclick={self.link.callback(move |_| Message::Color)}
                >
                    <Icon name={"palette"} size={IconSize::Icon18} />
                </span>
                {self.edit_generator_name()}
            </>
        }
    }

    fn edit_generator_name(&self) -> Html {
        let generator = self.props.generator;
        html! {
            <>
                <input
                    type="text"
                    class="signature__generator-name-input"
                    value={self.editing.as_ref().unwrap().name.as_ref().map_or(self.props.info.name.clone(), std::clone::Clone::clone)}
                    oninput={self.link.callback(move |e: InputData| {
                        Message::Rename(e.value)
                    })}
                    onkeyup={self.link.callback(move |e: KeyboardEvent| {
                        e.stop_propagation();
                        if e.key().to_ascii_lowercase() == "enter" {
                            Message::Done
                        } else {
                            Message::Noop
                        }
                    })}
                />
                <span class="signature__item-child">
                    {self.props.info.diagram.dimension()}
                </span>
                <span
                    class="signature__item-child"
                    onclick={self.props.dispatch.reform(move |_| Action::RemoveGenerator(generator))}
                >
                    <Icon name={"delete"} size={IconSize::Icon18} />
                </span>
                <span
                    class="signature__item-child"
                    onclick={self.link.callback(move |_| {
                        Message::Done
                    })}
                >
                    <Icon name={"done"} size={IconSize::Icon18} />
                </span>
            </>
        }
    }

    fn view_generator(&self) -> Html {
        let generator = self.props.generator;
        html! {
            <>
                <span
                    class="signature__generator-color"
                    style={format!("background: {}", self.props.info.color)}
                />
                <span
                    class="signature__item-child signature__item-name"
                    onclick={self.props.dispatch.reform(move |_| Action::SelectGenerator(generator))}
                >
                    {&self.props.info.name}
                </span>
                <span class="signature__item-child">
                    {self.props.info.diagram.dimension()}
                </span>
                <span
                    class="signature__item-child"
                    onclick={self.link.callback(|_| Message::Edit)}
                >
                    <Icon name={"edit"} size={IconSize::Icon18} />
                </span>
            </>
        }
    }
}
