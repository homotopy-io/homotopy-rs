use yew::prelude::*;
use yew_functional::function_component;

use crate::components::drawer::Drawer;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

#[function_component(SettingsView)]
pub fn settings_view(_: &Props) -> Html {
    html! {
        <Drawer title="Settings" class="settings">
            {"Hello, World!"}
        </Drawer>
    }
}
