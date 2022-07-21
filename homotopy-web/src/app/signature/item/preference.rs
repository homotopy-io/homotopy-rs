use yew::prelude::*;

#[derive(Properties, Debug, Clone, PartialEq)]
pub struct GeneratorPreferenceCheckboxProps {
    pub name: &'static str,
    pub tooltip: Option<&'static str>,
    pub checked: bool,
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
        html! {
            <div
                class={"signature__generator-preference"}
                onclick={ctx.props().onclick.clone()}
            >
                <span>{ctx.props().name}</span>
                <input
                    type={"checkbox"}
                    checked={ctx.props().checked}
                />
            </div>
        }
    }
}
