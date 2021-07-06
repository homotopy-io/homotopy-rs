use crate::components::icon::{Icon, IconSize};

use crate::model::proof::{Action, Color, GeneratorEdit, GeneratorInfo, Signature};
use homotopy_core::Generator;
use im::HashMap;
use palette::Srgb;
use web_sys::Element;
use yew::prelude::*;

const COLOR_RGB: [[[u8; 3]; 4]; 2] = [
    [
        [41, 128, 185],
        [192, 57, 43],
        [243, 156, 18],
        [142, 68, 173],
    ],
    [[39, 174, 96], [241, 196, 15], [255, 255, 255], [0, 0, 0]],
];

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub signature: Signature,
    pub dispatch: Callback<Action>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Done(Generator),
    DoneColor(Generator),
    Edit(Generator),
    Color(Generator),
    Rename(Generator, String),
    Recolor(Generator, Color),
    Noop,
}

pub struct SignatureView {
    link: ComponentLink<Self>,
    props: Props,
    editing: Vec<Generator>,
    coloring: Vec<Generator>,
    renames: HashMap<Generator, String>,
    recolors: HashMap<Generator, Color>,
}

impl Component for SignatureView {
    type Message = Message;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
            editing: Default::default(),
            coloring: Default::default(),
            renames: Default::default(),
            recolors: Default::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::Edit(generator) => self.editing.push(generator),
            Message::Color(generator) => self.coloring.push(generator),
            Message::Done(generator) => {
                let dispatch = &self.props.dispatch;
                for (g, n) in self.renames.iter() {
                    if n != &self.props.signature[g].name {
                        dispatch.emit(Action::EditGenerator(*g, GeneratorEdit::Rename(n.clone())));
                    }
                }
                for (g, n) in self.recolors.iter() {
                    if n != &self.props.signature[g].color {
                        dispatch.emit(Action::EditGenerator(*g, GeneratorEdit::Recolor(n.clone())));
                    }
                }
                self.renames.retain(|g, _| g != &generator);
                self.recolors.retain(|g, _| g != &generator);
                self.editing.retain(|g| g != &generator);
                self.coloring.retain(|g| g != &generator);
            }
            Message::DoneColor(generator) => {
                let dispatch = &self.props.dispatch;
                for (g, n) in self.recolors.iter() {
                    if n != &self.props.signature[g].color {
                        dispatch.emit(Action::EditGenerator(*g, GeneratorEdit::Recolor(n.clone())));
                    }
                }
                self.coloring.retain(|g| g != &generator);
            }
            Message::Rename(generator, name) => {
                self.renames.insert(generator, name);
                return false;
            }
            Message::Recolor(generator, color) => {
                self.recolors.insert(generator, color);
                return false;
            }
            Message::Noop => {
                return false;
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props == props {
            false
        } else {
            self.renames.retain(|g, _| props.signature.contains_key(g));
            self.recolors.retain(|g, _| props.signature.contains_key(g));
            self.editing.retain(|g| props.signature.contains_key(g));
            self.props = props;
            true
        }
    }

    fn view(&self) -> Html {
        let mut generators: Vec<_> = self.props.signature.iter().collect();
        generators.sort_by_key(|(generator, info)| (generator.dimension, &info.name));
        let generators: Html = self
            .props
            .signature
            .iter()
            .map(|(generator, info)| {
                if self.coloring.contains(generator) {
                    self.color_generator(*generator, info)
                } else if self.editing.contains(generator) {
                    self.edit_generator(*generator, info)
                } else {
                    self.view_generator(*generator, info)
                }
            })
            .collect();

        // TODO: Order?
        // TODO: Extract drawer component
        // TODO: Add search
        // TODO: Drag to reorder
        // TODO: Folders/groups
        // TODO: On mobile, drag to the side to delete
        html! {
            <ul class="signature__generators">{generators}</ul>
        }
    }
}

impl SignatureView {
    fn color_generator(&self, generator: Generator, info: &GeneratorInfo) -> Html {
        let dispatch = &self.props.dispatch;
        let color_ref = NodeRef::default();
        let color_clone = color_ref.clone();
        let picker_ref = NodeRef::default();
        let buttons = COLOR_RGB
            .iter()
            .map(|colors| {
                let b = colors.iter().map(|c| {
                    let (icon, picker) = (color_ref.clone(), picker_ref.clone());
                    html! {
                        <span style=format!("color:{}", Color(Srgb::new(c[0], c[1], c[2])))
                            class="signature__generator-edit"
                            onclick=self.link.callback(move |_| {
                                let color = Color(Srgb::new(c[0], c[1], c[2]));
                                let s = format!("background-color:{}; height:32px",color);
                                icon.cast::<Element>().unwrap().set_attribute("style",&s).unwrap();
                                let p = picker.cast::<Element>().unwrap();
                                p.set_attribute("type", "button").unwrap();
                                p.set_attribute("value", &color.to_string()).unwrap();
                                p.set_attribute("type", "color").unwrap();
                                Message::Recolor(generator, Color(Srgb::new(c[0], c[1], c[2])))
                            })
                        >
                            <Icon name={"circle"} size={IconSize::Icon18} />
                        </span>
                    }
                }
            ).collect::<Html>();
                html! {
                    <li
                        class="signature__generator"
                        style="justify-content: center; background-color:#f1f1f1"
                    >
                    {b}
                    </li>
                }
            })
            .collect::<Html>();

        html! {
            <>
                <li
                    class="signature__generator"
                >
                    <span
                        style={format!(
                            "background-color:{}; height:32px",
                            self.recolors.get(&generator).map_or(&info.color, |color| color),
                        )}
                        ref=color_ref.clone()
                    >
                    <span style="color:black"
                        class="signature__generator-edit"
                        onclick=self.link.callback(move |_| Message::DoneColor(generator))
                    >
                        <Icon name={"done"} size={IconSize::Icon18} />
                    </span>
                    </span>
                    {self.edit_generator_name(generator, info)}
                </li>
                {buttons}
                <li
                    class="signature__generator"
                    style="background-color:#f1f1f1"
                >
                    <input
                        style="border:none"
                        ref=picker_ref.clone()
                        type="color"
                        value= {
                            self.recolors.get(&generator).map_or(&info.color, |color| color)
                        }
                        oninput=self.link.callback(move |e: InputData| {
                            let str = e.value;
                            let r = u8::from_str_radix(&str[1..3], 16).unwrap();
                            let g = u8::from_str_radix(&str[3..5], 16).unwrap();
                            let b = u8::from_str_radix(&str[5..], 16).unwrap();
                            let s = format!("background-color:{}", str.as_str());
                            color_clone.cast::<Element>().unwrap().set_attribute("style",&s).unwrap();
                            Message::Recolor(generator, Color(Srgb::new(r, g, b)))
                        })
                    />
                </li>
            </>
        }
    }

    fn edit_generator(&self, generator: Generator, info: &GeneratorInfo) -> Html {
        html! {
            <li
                class="signature__generator"
            >
                <span style={format!("color:{}", &info.color)}
                    class="signature__generator-edit"
                    onclick=self.link.callback(move |_| Message::Color(generator))
                >
                    <Icon name={"palette"} size={IconSize::Icon18} />
                </span>
                {self.edit_generator_name(generator, info)}
            </li>
        }
    }

    fn edit_generator_name(&self, generator: Generator, info: &GeneratorInfo) -> Html {
        let dispatch = &self.props.dispatch;

        html! {
            <>
                <input
                    type="text"
                    class="signature__generator-name-input"
                    value={
                        self.renames.get(&generator).map_or(&info.name, |name| name)
                    }
                    oninput=self.link.callback(move |e: InputData| {
                        Message::Rename(generator, e.value)
                    })
                    onkeyup=self.link.callback(move |e: KeyboardEvent| {
                        e.stop_propagation();
                        if e.key().to_ascii_lowercase() == "enter" {
                            Message::Done(generator)
                        } else {
                            Message::Noop
                        }
                    })
                />
                <span class="signature__generator-dimension">
                    {info.diagram.dimension()}
                </span>
                <span
                    class="signature__generator-edit"
                    onclick=dispatch.reform(move |_| Action::RemoveGenerator(generator))
                >
                    <Icon name={"delete"} size={IconSize::Icon18} />
                </span>
                <span
                    class="signature__generator-edit"
                    onclick=self.link.callback(move |_| {
                        Message::Done(generator)
                    })
                >
                    <Icon name={"done"} size={IconSize::Icon18} />
                </span>
            </>
        }
    }

    fn view_generator(&self, generator: Generator, info: &GeneratorInfo) -> Html {
        let dispatch = &self.props.dispatch;

        html! {
            <li
                class="signature__generator"
            >
                <span
                    class="signature__generator-color"
                    style={format!("background: {}", info.color)}
                />
                <span
                    class="signature__generator-name"
                    onclick=dispatch.reform(move |_| Action::SelectGenerator(generator))
                >
                    {&info.name}
                </span>
                <span class="signature__generator-dimension">
                    {info.diagram.dimension()}
                </span>
                <span
                    class="signature__generator-edit"
                    onclick=self.link.callback(move |_| Message::Edit(generator))
                >
                    <Icon name={"edit"} size={IconSize::Icon18} />
                </span>
            </li>
        }
    }
}
