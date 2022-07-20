use yew::prelude::*;

#[derive(Properties, Debug, Clone, PartialEq)]
pub struct GeneratorPreferenceCheckboxProps {
    pub name: &'static str,
    pub tooltip: Option<&'static str>,
    pub checked: bool,
    pub oninput: Callback<InputEvent>,
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
            <div class={"signature__generator-preference"}>
                <p>{ctx.props().name}</p>
                <input
                    type={"checkbox"}
                    checked={ctx.props().checked}
                    oninput={ctx.props().oninput.clone()}
                />
            </div>
        }
    }
}
