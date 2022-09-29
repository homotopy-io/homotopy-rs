use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

use crate::model;

#[derive(Debug, Clone)]
pub enum Msg {
    LogIn(User),
    // Need two message types because logging out takes time
    // It won't complete immediately
    LogOut,
    CompleteLogOut,
    SaveProject(bool),
    GetUserProjects,
    FoundUserProjects(Vec<Project>),
}

#[derive(Debug, Properties, Clone, PartialEq)]
pub struct Props {
    pub dispatch: Callback<model::Action>,
}

pub struct AccountView {
    user: Option<User>,
    unsubscribe: JsValue,
    projects: Vec<Project>,
}

impl Component for AccountView {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            user: None,
            unsubscribe: Self::sign_in_callback(ctx, JsValue::NULL),
            projects: Vec::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(ref user) = &self.user {
            let username = user
                .display_name
                .as_ref()
                .or(user.email.as_ref())
                .map(Clone::clone)
                .unwrap_or_default();
            html! {
                <>
                    <h3>{username}</h3>
                    <button onclick={ctx.link().callback(|_| Msg::LogOut)}>{"Log out"}</button>
                    <br/>
                    <button onclick={ctx.link().callback(|_| Msg::SaveProject(false))}>{"Save Project"}</button>
                    <button onclick={ctx.link().callback(|_| Msg::SaveProject(true))}>{"Save Project As"}</button>
                    <button onclick={ctx.link().callback(|_| Msg::GetUserProjects)}>{"get user projects"}</button>
                    <br/>
                    {for self.projects.iter().map(|project| html! {
                        <div class="project-card">{project.id.clone()}</div>
                    })}
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

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if self.user.is_none() {
            initialize_ui();
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LogIn(user) => {
                // Parse user info
                self.user = Some(user);
                true
            }
            Msg::LogOut => {
                Self::log_out(ctx);
                false
            }
            Msg::CompleteLogOut => {
                self.user = None;
                self.unsubscribe = Self::sign_in_callback(ctx, self.unsubscribe.clone());
                true
            }
            Msg::SaveProject(new) => {
                // We send a message to the main app to get access to the serialized proof data.
                // The app then calls `upload_project` below.
                if let Some(ref user) = self.user {
                    ctx.props()
                        .dispatch
                        .emit(model::Action::UploadProject(user.uid.clone(), new));
                }
                true
            }
            Msg::GetUserProjects => {
                if let Some(ref user) = self.user {
                    Self::get_user_projects(ctx, user);
                }
                true
            }
            Msg::FoundUserProjects(projects) => {
                self.projects = projects;
                true
            }
        }
    }
}

impl AccountView {
    // register callbacks for onAuthStateChanged
    // callbacks can only be called once
    // need to re-register and unsubscribe to keep sanity
    fn sign_in_callback(ctx: &Context<Self>, unsubscribe: JsValue) -> JsValue {
        let login_msg = ctx.link().callback(Msg::LogIn);
        let login_js_closure = Closure::once_into_js(move |user: JsValue| {
            login_msg.emit(serde_wasm_bindgen::from_value(user).unwrap());
        });
        register_auth_callback(login_js_closure, unsubscribe)
    }

    fn log_out(ctx: &Context<Self>) {
        let logout_msg = ctx.link().callback(|_| Msg::CompleteLogOut);
        let js_closure = Closure::once_into_js(move |_: JsValue| logout_msg.emit(()));
        log_out(js_closure);
    }

    fn get_user_projects(ctx: &Context<Self>, user: &User) {
        let get_user_projects_cb = ctx.link().callback(Msg::FoundUserProjects);
        let js_closure = Closure::once_into_js(move |projects: JsValue| {
            get_user_projects_cb.emit(serde_wasm_bindgen::from_value(projects).unwrap());
        });
        get_user_projects(JsValue::from_str(&user.uid), js_closure);
    }
}

// In future, the name of the project shouldn't be the key for the firestore database. Also, the
// hom data should be stored in the firebase storage bucket rather than firestore. It should also
// somehow generate a new id for each project so that save/save-as functionality can work
// correctly.
pub fn upload_project(uid: String, new: bool, project: Project) {
    let request = serde_wasm_bindgen::to_value(&SaveProjectRequest { uid, new, project }).unwrap();
    save_project(request);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    uid: String,
    display_name: Option<String>,
    email: Option<String>,
    photo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub data: model::SerializedData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveProjectRequest {
    uid: String,
    new: bool,
    project: Project,
}

#[wasm_bindgen(module = "/src/app/account/account_script.js")]
extern "C" {
    #[wasm_bindgen(js_name = "initializeUI")]
    pub fn initialize_ui();

    #[wasm_bindgen(js_name = "logOut")]
    pub fn log_out(callback: JsValue);

    #[wasm_bindgen(js_name = "resgisterAuthCallback")]
    pub fn register_auth_callback(logInCallback: JsValue, unsubscribe: JsValue) -> JsValue;

    #[wasm_bindgen(js_name = "saveProject")]
    pub fn save_project(saveProjectRequest: JsValue);

    #[wasm_bindgen(js_name = "getUserProjects")]
    pub fn get_user_projects(uid: JsValue, callback: JsValue) -> JsValue;
}
