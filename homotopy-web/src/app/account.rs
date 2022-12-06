// use wasm_bindgen::prelude::*;
// use yew::prelude::*;

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum Msg {
//     // Todo: pass an actual data structure
//     LogIn(String),
//     // Need two message types because logging out takes time
//     // It won't complete immediately
//     LogOut,
//     CompleteLogOut,
// }

// #[derive(Debug, Properties, Clone, PartialEq, Eq)]
// pub struct Props {
//     //pub dispatch: Callback<model::Action>,
// }

// pub struct AccountView {
//     user: Option<User>,
//     unsubscribe: JsValue,
// }

// impl Component for AccountView {
//     type Message = Msg;
//     type Properties = Props;

//     fn create(ctx: &Context<Self>) -> Self {
//         Self {
//             user: None,
//             unsubscribe: Self::sign_in_callback(ctx, JsValue::NULL),
//         }
//     }

//     fn view(&self, ctx: &Context<Self>) -> Html {
//         if let Some(userdata) = &self.user {
//             html! {
//                 <>
//                     <h3>{userdata.display_name.clone()}</h3>
//                     //<span>{userdata.username.clone()}</span>
//                     <button onclick={ctx.link().callback(|_| Msg::LogOut)}>{"Log out"}</button>
//                 </>
//             }
//         } else {
//             html! {
//                 <>
//                     <span>{"You are not logged in. Please try the following methods."}</span>
//                     <div id="firebaseui-auth-container"></div>
//                 </>
//             }
//         }
//     }

//     fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
//         if self.user.is_none() {
//             initialize_ui();
//         }
//     }

//     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
//         match msg {
//             Msg::LogIn(user) => {
//                 // Parse user info
//                 self.user = Some(User { display_name: user });
//                 true
//             }
//             Msg::LogOut => {
//                 Self::log_out(ctx);
//                 false
//             }
//             Msg::CompleteLogOut => {
//                 self.user = None;
//                 self.unsubscribe = Self::sign_in_callback(ctx, self.unsubscribe.clone());
//                 true
//             }
//         }
//     }
// }

// impl AccountView {
//     // register callbacks for onAuthStateChanged
//     // callbacks can only be called once
//     // need to re-register and unsubscribe to keep sanity
//     fn sign_in_callback(ctx: &Context<Self>, unsubscribe: JsValue) -> JsValue {
//         let login_cb = ctx.link().callback(Msg::LogIn);
//         let login_callback = Closure::once_into_js(move |username: String| login_cb.emit(username));
//         register_auth_callback(login_callback, unsubscribe)
//     }

//     fn log_out(ctx: &Context<Self>) {
//         let logout_cb = ctx.link().callback(|_| Msg::CompleteLogOut);
//         let logout_callback = Closure::once_into_js(move |_: JsValue| logout_cb.emit(()));
//         log_out(logout_callback);
//     }
// }

// pub struct User {
//     display_name: String,
//     //email: String,
// }

// #[wasm_bindgen(module = "/src/app/account/account_script.js")]
// extern "C" {
//     #[wasm_bindgen(js_name = "initializeUI")]
//     pub fn initialize_ui();

//     #[wasm_bindgen(js_name = "logOut")]
//     pub fn log_out(callback: JsValue);

//     #[wasm_bindgen(js_name = "resgisterAuthCallback")]
//     pub fn register_auth_callback(logInCallback: JsValue, unsubscribe: JsValue) -> JsValue;
// }
