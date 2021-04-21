use yew::prelude::*;
use yew_functional::function_component;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    pub name: String,
}

#[function_component(Icon)]
pub fn icon(props: &Props) -> Html {
    html! {
        <i class="material-icons md-light">{&props.name}</i>
    }
}
