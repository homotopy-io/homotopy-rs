use yew::prelude::*;

use auth_bindings::*;

pub mod auth_bindings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    LogOut,
}

#[derive(Debug, Properties, Clone, PartialEq, Eq)]
pub struct Props {
    //pub dispatch: Callback<model::Action>,
}

pub struct AccountView {}
// User: Option<User>


impl Component for AccountView {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <span id="account-details"></span>
                <button onclick={ctx.link().callback(|_| Msg::LogOut)}>{"Log out"}</button>
                <div id="firebaseui-auth-container"></div>
            </>
        }

    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render && !logged_in() {
            initialize_ui();
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LogOut => log_out(),
        };
        false
    }
}