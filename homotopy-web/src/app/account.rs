use wasm_bindgen::prelude::*;
use yew::prelude::*;

use auth_bindings::*;

pub mod auth_bindings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    LogIn,
    LogOut,
}

#[derive(Debug, Properties, Clone, PartialEq, Eq)]
pub struct Props {
    //pub dispatch: Callback<model::Action>,
}

pub struct AccountView {}

impl Component for AccountView {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        html! {
            <>
                <h3 id="username">{"Guest user"}</h3>
                // Todo: handle this callback function properly
                <button onclick={ctx.link().callback(|_| Msg::LogIn)}>
                    { "Log in" }
                </button>
                <button onclick={ctx.link().callback(|_| Msg::LogOut)}>
                    { "Log out" }
                </button>
                <div id="firebaseui-auth-container"></div>
            </>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            initialize_ui();
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LogIn => {
                // let auth = get_auth();
                // let provider = GoogleAuthProvider::new();

                // let result = sign_in_with_popup_google(auth, provider);
                // if let Some(u) = result {
                //     log::debug!("{}", "yes yes!");
                // } else {
                //     log::debug!("{}", "no no!");
                // }
            },
            Msg::LogOut => {},
        };
        false
    }
}

// #[wasm_bindgen(module = "/src/app/account/account_script.js")]
// extern "C" {
//     #[wasm_bindgen(js_name = "logIn")]
//     pub fn log_in();

//     #[wasm_bindgen(js_name = "logOut")]
//     pub fn log_out();
// }
