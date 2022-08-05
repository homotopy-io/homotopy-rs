use yew::prelude::*;
use yew_macro::function_component;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    pub name: String,
    pub size: IconSize,
    #[prop_or(false)]
    pub light: bool,
    #[prop_or_default]
    pub class: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IconSize {
    Icon18,
    Icon24,
    /* Icon36,
     * Icon48, */
}

#[function_component(Icon)]
pub fn icon(props: &Props) -> Html {
    let class = format!(
        "material-icons md-{} {} {}",
        if props.light { "light" } else { "dark" },
        match props.size {
            IconSize::Icon18 => "md-18",
            IconSize::Icon24 => "md-24",
            /* IconSize::Icon36 => "md-36",
             * IconSize::Icon48 => "md-48", */
        },
        props.class,
    );

    html! {
        <i class={class}>{&props.name}</i>
    }
}
