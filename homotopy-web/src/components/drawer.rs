use yew::prelude::*;
use yew_functional::function_component;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub class: &'static str,
    pub title: &'static str,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Drawer)]
pub fn drawer(props: &Props) -> Html {
    html! {
        <aside class={format!("{} drawer", props.class)}>
            <div class="drawer__header">
                <span class="drawer__title">
                    {props.title}
                </span>
            </div>
            <div class="drawer__content">
                { for props.children.iter() }
            </div>
        </aside>
    }
}
