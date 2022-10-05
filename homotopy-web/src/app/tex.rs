use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TexSpanMode {
    Editing,
    Viewing,
}

#[derive(Default, Clone, Debug, Properties, PartialEq)]
pub struct TexSpanProps {
    pub raw_tex: String,
    pub class: &'static str,

    #[prop_or(false)]
    pub is_editable_as_textarea: bool,
    #[prop_or_default]
    pub placeholder: &'static str,
    #[prop_or("#cc0000")]
    pub error_color: &'static str,
    #[prop_or_default]
    pub on_input: Callback<String>,
    #[prop_or_default]
    pub on_focus_out: Callback<String>,
    #[prop_or_default]
    pub on_key_down: Callback<KeyboardEvent>,
}

pub enum TexSpanMsg {
    EditTex(String),
    SwitchMode(TexSpanMode),
    Noop,
}

pub struct TexSpan {
    span: web_sys::Element,
    textarea_ref: NodeRef,
    render_opts: katex::Opts,
    raw_tex: String,
    mode: TexSpanMode,
}

impl Component for TexSpan {
    type Message = TexSpanMsg;
    type Properties = TexSpanProps;

    fn create(ctx: &Context<Self>) -> Self {
        let span = gloo::utils::document().create_element("span").unwrap();

        let render_opts = katex::Opts::builder()
            .display_mode(false)
            .trust(false)
            .throw_on_error(false)
            .error_color(ctx.props().error_color)
            .build()
            .unwrap();

        Self {
            span,
            textarea_ref: NodeRef::default(),
            render_opts,
            raw_tex: ctx.props().raw_tex.clone(),
            mode: TexSpanMode::Viewing,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TexSpanMsg::EditTex(tex) => {
                self.raw_tex = tex;
                false
            }
            TexSpanMsg::SwitchMode(mode) => {
                if mode == self.mode {
                    return false;
                }

                self.mode = mode;
                true
            }
            TexSpanMsg::Noop => false,
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if self.mode == TexSpanMode::Editing {
            if let Some(textarea) = self.textarea_ref.cast::<HtmlTextAreaElement>() {
                textarea.focus().expect("tex textarea to be focusable");
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let inner = if ctx.props().is_editable_as_textarea && self.mode == TexSpanMode::Editing {
            let class = format!("tex__textarea {}", ctx.props().class,);

            let super_on_input = ctx.props().on_input.clone();
            let on_input = ctx.link().callback(move |e: InputEvent| {
                e.stop_propagation();
                let input: HtmlTextAreaElement = e.target_unchecked_into();
                let val = input.value();
                super_on_input.emit(val.clone());
                TexSpanMsg::EditTex(val)
            });

            let super_on_focus_out = ctx.props().on_focus_out.clone();
            let on_focus_out = ctx.link().callback(move |e: FocusEvent| {
                let input: HtmlTextAreaElement = e.target_unchecked_into();
                super_on_focus_out.emit(input.value());
                TexSpanMsg::SwitchMode(TexSpanMode::Viewing)
            });

            html! {
                <textarea
                    class={class}
                    ref={self.textarea_ref.clone()}
                    placeholder={ctx.props().placeholder}
                    value={ctx.props().raw_tex.clone()}
                    oninput={on_input}
                    onfocusout={on_focus_out}
                    onkeydown={ctx.props().on_key_down.clone()}
                    onkeyup={ctx.link().callback(|e: KeyboardEvent| {
                        e.stop_propagation();
                        TexSpanMsg::Noop
                    })}
                />
            }
        } else {
            let raw_tex = if ctx.props().raw_tex.is_empty() {
                ctx.props().placeholder
            } else {
                &ctx.props().raw_tex
            };
            self.span
                .set_inner_html(&render_tex(raw_tex, &self.render_opts));
            self.span.set_class_name(&format!(
                "tex__span {} {}",
                if ctx.props().raw_tex.is_empty() {
                    "tex__span-placeholder"
                } else {
                    ""
                },
                ctx.props().class,
            ));

            Html::VRef(self.span.clone().into())
        };

        let editable = if ctx.props().is_editable_as_textarea {
            "tex__editable"
        } else {
            "tex__non-editable"
        };

        html! {
            <div
                class={format!("tex__wrapper {0} {0}-wrapper {1}", ctx.props().class, editable)}
                onclick={ctx.link().callback(|_| TexSpanMsg::SwitchMode(TexSpanMode::Editing))}
                >
                {inner}
            </div>
        }
    }
}

fn render_tex(raw_tex: &str, render_opts: &katex::Opts) -> String {
    raw_tex
        .split('$')
        .enumerate()
        .map(|(i, tex)| {
            if i % 2 == 1 {
                katex::render_with_opts(tex, render_opts).unwrap()
            } else {
                html_escape::encode_text(&tex).into_owned()
            }
        })
        .collect()
}
