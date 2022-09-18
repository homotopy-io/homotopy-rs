use wasm_bindgen::prelude::*;
use yew::prelude::*;

//use auth_bindings::*;

//pub mod auth_bindings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    LogIn(String),
    LogOut,
}

#[derive(Debug, Properties, Clone, PartialEq, Eq)]
pub struct Props {
    //pub dispatch: Callback<model::Action>,
}

pub struct AccountView {
    user: Option<User>,
}

impl Component for AccountView {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let cb = ctx.link().callback(Msg::LogIn);
        let callback = Closure::once_into_js(move |username: String| cb.emit(username));
        register_auth_callback(callback);

        Self {
            user: Default::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(userdata) = &self.user {
            html! {
                <>
                    <span>{userdata.display_name.clone()}</span>
                    //<span>{userdata.username.clone()}</span>
                    <button onclick={ctx.link().callback(|_| Msg::LogOut)}>{"Log out"}</button>
                </>
            }
        } else {
            html! {
                <>
                    <span>{"You are not logged in. Please try the following methods."}</span>
                    <div id="firebaseui-auth-container"></div>
                </>
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if self.user.is_none() {
            initialize_ui();
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LogIn(user) => {
                // Parse user info
                self.user = Some(User { display_name: user });
                true
            }
            Msg::LogOut => {
                // Handle log out info, if sth goes wrong
                // let result = log_out();
                log_out();

                // re-register callback, since it can only be used once
                let cb = ctx.link().callback(Msg::LogIn);
                let callback = Closure::once_into_js(move |username: String| cb.emit(username));
                register_auth_callback(callback);

                self.user = None;
                true
            }
        }
    }
}

pub struct User {
    display_name: String,
    //email: String,
}

#[wasm_bindgen(module = "/src/app/account/account_script.js")]
extern "C" {
    #[wasm_bindgen(js_name = "initializeUI")]
    pub fn initialize_ui();

    #[wasm_bindgen(js_name = "logOut")]
    pub fn log_out();

    #[wasm_bindgen(js_name = "resgisterAuthCallback")]
    pub fn register_auth_callback(callback: JsValue);
}
