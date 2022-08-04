use yew::prelude::*;

#[derive(Properties, Debug, Clone, PartialEq)]
pub struct GeneratorPreferenceCheckboxProps {
    pub left: &'static str,
    pub right: &'static str,
    pub tooltip: Option<&'static str>,
    pub color: String,
    pub checked: bool,
    pub onclick: Callback<MouseEvent>,
    #[prop_or(false)]
    pub disabled: bool,
}

pub struct GeneratorPreferenceCheckbox;

impl Component for GeneratorPreferenceCheckbox {
    type Message = ();
    type Properties = GeneratorPreferenceCheckboxProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let color_main = &ctx.props().color;
        let color_text_on = "var(--drawer-background)";
        let color_text_off = "var(--drawer-foreground)";

        let border_style = format!("border: 1px solid {};", color_main);

        let slider_style = format!(
            "transform: translateX({}); background-color: {};",
            if ctx.props().checked { "100%" } else { "0" },
            color_main,
        );

        let (left_style, right_style) = if ctx.props().checked {
            (
                format!("color: {};", color_text_off),
                format!("color: {};", color_text_on),
            )
        } else {
            (
                format!("color: {};", color_text_on),
                format!("color: {};", color_text_off),
            )
        };

        html! {
            <div
                class={"signature__generator-preference"}
                onclick={ctx.props().onclick.clone()}
                style={border_style}
            >
                <div class="signature__generator-preference-options-wrapper">
                    <div class="signature__generator-preference-option" style={left_style}>
                        {ctx.props().left}
                    </div>
                    <div class="signature__generator-preference-option" style={right_style}>
                        {ctx.props().right}
                    </div>
                </div>
                <div class="signature__generator-preference-slider" style={slider_style} />
                <input
                    type="checkbox"
                    checked={ctx.props().checked}
                    disabled={ctx.props().disabled}
                />
            </div>
        }
    }
}
