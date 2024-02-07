use yew::prelude::*;

#[derive(Properties, Debug, Clone, PartialEq)]
pub struct GeneratorPreferenceCheckboxProps {
    pub checked: bool,
    #[prop_or(false)]
    pub disabled: bool,
    pub left: &'static str,
    pub right: &'static str,
    #[prop_or_default]
    pub tooltip: Option<&'static str>,
    pub color: String,
    pub onclick: Callback<MouseEvent>,
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
        let color_disabled = "var(--drawer-foreground-dimmed)";
        let color_text_on = "var(--drawer-background)";
        let color_text_off = "var(--drawer-foreground)";
        let color_text_disabled = "var(--drawer-foreground-dimmed-text)";

        let color = if ctx.props().disabled {
            color_disabled
        } else {
            color_main
        };

        let border_style = format!("border: 1px solid {color};");

        let slider_style = format!(
            "transform: translateX({}); background-color: {color};",
            if ctx.props().checked { "100%" } else { "0" },
        );

        let (left_color, right_color) = if ctx.props().disabled {
            (color_text_disabled, color_text_disabled)
        } else if ctx.props().checked {
            (color_text_off, color_text_on)
        } else {
            (color_text_on, color_text_off)
        };
        let (left_style, right_style) = (
            format!("color: {left_color};"),
            format!("color: {right_color};"),
        );

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
