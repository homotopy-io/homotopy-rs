use std::{
    fmt,
    time::{Duration, UNIX_EPOCH},
};

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

use crate::{
    components::{
        icon::{Icon, IconSize},
        toast::{toast, Toast},
    },
    model,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationStatus {
    Failed,
    Updating,
    Succeeded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteProjectMetadata {
    pub id: String,
    pub uid: String,
    pub title: String,
    pub author: String,
    pub abstr: String,
    pub visibility: ProjectVisibility,
    pub created: u64,
    #[serde(rename = "lastModified")]
    pub last_modified: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectVisibility {
    Private,
    Public,
    Published,
}

impl Default for ProjectVisibility {
    fn default() -> Self {
        Self::Private
    }
}

impl fmt::Display for ProjectVisibility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Private => "Private",
            Self::Public => "Public",
            Self::Published => "Published",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    LogIn(User),
    LogOut,
    CompleteLogOut,
    SetProjectsView(ProjectsView),
    FetchProjects,
    ProjectsFetched(ProjectCollection),
    SaveProject,
    SaveNewProject,
    ProjectSaved(Option<RemoteProjectMetadata>),
    PublishProject,
    ProjectPublished(Option<RemoteProjectMetadata>),
    UpdateProjectMetadata(ProjectMetadataUpdate),
    ProjectMetadataUpdated(Option<RemoteProjectMetadata>),
    DeleteProject(String),
    ProjectDeleted(Option<String>),
    OpenProject(RemoteProjectMetadata),
    ProjectDownloaded((RemoteProjectMetadata, Vec<u8>)),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectsView {
    Personal,
    Published,
}

#[derive(Debug, Properties, Clone, PartialEq)]
pub struct Props {
    pub proof: model::history::Proof,
    pub remote_project_metadata: Option<RemoteProjectMetadata>,
    pub dispatch: Callback<model::Action>,
}

pub struct AccountView {
    user: Option<User>,
    unsubscribe: JsValue,
    projects: ProjectCollection,
    projects_view: ProjectsView,
    status: OperationStatus,
}

impl Component for AccountView {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().callback(|()| Msg::FetchProjects).emit(());

        Self {
            user: None,
            unsubscribe: Self::sign_in_callback(ctx, JsValue::NULL),
            projects: ProjectCollection::default(),
            projects_view: ProjectsView::Personal,
            status: OperationStatus::Updating,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(userdata) = &self.user {
            let user_icon = if let Some(url) = &userdata.photo_url {
                html! {
                    <img
                        src={url.clone()}
                        referrerpolicy="no-referrer"
                        class="account__user-photo"
                        width="36px"
                        height="36px" />
                }
            } else {
                html! {
                    <Icon name="account_circle" size={IconSize::Icon36} />
                }
            };

            let status_indicator = match self.status {
                OperationStatus::Failed => html! {
                    <div class="account__save-status account__save-status-failed">
                        {"An error occurred"}
                    </div>
                },
                OperationStatus::Updating => html! {
                    <div class="account__save-status">
                        {"Updating..."}
                    </div>
                },
                OperationStatus::Succeeded => html! {
                    <div class="account__save-status">
                        {"Up to date"}
                    </div>
                },
            };

            let toggle_personal_cb = ctx
                .link()
                .callback(move |_| Msg::SetProjectsView(ProjectsView::Personal));
            let toggle_published_cb = ctx
                .link()
                .callback(move |_| Msg::SetProjectsView(ProjectsView::Published));
            let (toggle_personal_class, toggle_published_class) = {
                let (a, b) = match self.projects_view {
                    ProjectsView::Personal => ("active", "inactive"),
                    ProjectsView::Published => ("inactive", "active"),
                };
                (
                    format!("account__projects-view-toggle account__projects-view-toggle-{a}"),
                    format!("account__projects-view-toggle account__projects-view-toggle-{b}"),
                )
            };

            html! {
                <>
                    <div class="account__user-details">
                        { user_icon }
                        <div class="account__user-name">{userdata.display_name.clone()}</div>
                    </div>
                    <button onclick={ctx.link().callback(|_| Msg::LogOut)}>{"Log out"}</button>
                    <button onclick={ctx.link().callback(|_| Msg::FetchProjects)}>{"Refresh Projects"}</button>
                    <button onclick={ctx.link().callback(|_| Msg::SaveProject)}>{"Save"}</button>
                    <button onclick={ctx.link().callback(|_| Msg::SaveNewProject)}>{"Save new"}</button>
                    <button onclick={ctx.link().callback(|_| Msg::PublishProject)}>{"Publish"}</button>
                    <p>{status_indicator}</p>
                    <div class="account__project-list-wrapper">
                        <div class="account__projects-view-toggles-wrapper">
                            <div
                                class={toggle_personal_class}
                                onclick={toggle_personal_cb}>
                                    {"Personal"}
                            </div>
                            <div
                                class={toggle_published_class}
                                onclick={toggle_published_cb}>
                                    {"Published"}
                            </div>
                        </div>
                        { self.user_projects_list(ctx) }
                    </div>
                </>
            }
        } else {
            html! {
                <div>
                    <span>{"You are not logged in. Please try the following methods."}</span>
                    <div id="firebaseui-auth-container"></div>
                </div>
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if self.user.is_none() {
            init_ui();
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
            Msg::SetProjectsView(view) => {
                self.projects_view = view;
                true
            }
            Msg::FetchProjects => {
                Self::get_all_user_projects(ctx);
                false
            }
            Msg::ProjectsFetched(projects) => {
                self.status = OperationStatus::Succeeded;
                self.projects = projects;
                true
            }
            Msg::SaveProject => {
                self.save_project(ctx);
                self.status = OperationStatus::Updating;
                true
            }
            Msg::SaveNewProject => {
                self.save_new_project(ctx);
                self.status = OperationStatus::Updating;
                true
            }
            Msg::ProjectSaved(new_remote_metadata) => {
                // Show successful save
                if let Some(metadata) = new_remote_metadata {
                    tracing::debug!("project saved successfully");
                    self.status = OperationStatus::Succeeded;
                    toast(Toast::success("Project saved successfully"));
                    let set_remote_project_metadata = ctx
                        .props()
                        .dispatch
                        .reform(model::Action::SetRemoteProjectMetadata);
                    set_remote_project_metadata.emit(Some(metadata));
                } else {
                    self.status = OperationStatus::Failed;
                    toast(Toast::error("Project save failed"));
                }
                ctx.link().callback(|()| Msg::FetchProjects).emit(());
                true
            }
            Msg::PublishProject => {
                self.publish_project(ctx);
                self.status = OperationStatus::Updating;
                true
            }
            Msg::ProjectPublished(new_remote_metadata) => {
                // Show successful publish
                if let Some(metadata) = new_remote_metadata {
                    tracing::debug!("project published successfully");
                    self.status = OperationStatus::Succeeded;
                    toast(Toast::success("Project published successfully"));
                    let set_remote_project_metadata = ctx
                        .props()
                        .dispatch
                        .reform(model::Action::SetRemoteProjectMetadata);
                    set_remote_project_metadata.emit(Some(metadata));
                } else {
                    self.status = OperationStatus::Failed;
                    toast(Toast::error("Project publish failed"));
                }
                ctx.link().callback(|()| Msg::FetchProjects).emit(());
                true
            }
            Msg::UpdateProjectMetadata(update) => {
                self.update_project_metadata(ctx, &update);
                self.status = OperationStatus::Updating;
                true
            }
            Msg::ProjectMetadataUpdated(new_remote_metadata) => {
                // Show successful update
                if let Some(ref metadata) = new_remote_metadata {
                    tracing::debug!("project updated successfully");
                    for p in &mut self.projects.personal {
                        if p.id == metadata.id {
                            *p = metadata.clone();
                        }
                    }
                    self.status = OperationStatus::Succeeded;
                    toast(Toast::success("Project updated successfully"));
                } else {
                    self.status = OperationStatus::Failed;
                    toast(Toast::error("Project update failed"));
                }
                true
            }
            Msg::DeleteProject(id) => {
                Self::delete_project(ctx, id);
                self.status = OperationStatus::Updating;
                true
            }
            Msg::ProjectDeleted(deleted) => {
                // Show successful publish
                if let Some(id) = deleted {
                    tracing::debug!("project {id} deleted successfully");
                    self.status = OperationStatus::Succeeded;
                    toast(Toast::success("Project deleted successfully ({id})"));
                } else {
                    self.status = OperationStatus::Failed;
                    toast(Toast::error("Project delete failed ({id})"));
                }
                ctx.link().callback(|()| Msg::FetchProjects).emit(());
                true
            }
            Msg::OpenProject(project) => {
                Self::download_project(ctx, &project);
                true
            }
            Msg::ProjectDownloaded((remote_project_metadata, blob)) => {
                let import_proof = ctx
                    .props()
                    .dispatch
                    .reform(move |bytes| model::proof::Action::ImportProof(bytes).into());
                import_proof.emit(blob.into());
                let set_remote_project_metadata = ctx
                    .props()
                    .dispatch
                    .reform(model::Action::SetRemoteProjectMetadata);
                set_remote_project_metadata.emit(Some(remote_project_metadata));
                true
            }
        }
    }
}

impl AccountView {
    // Register callbacks for onAuthStateChanged.
    fn sign_in_callback(ctx: &Context<Self>, unsubscribe: JsValue) -> JsValue {
        let login_cb = ctx.link().callback(Msg::LogIn);
        let login_cb_js =
            Closure::once_into_js(move |display_name: String, photo_url: Option<String>| {
                login_cb.emit(User {
                    display_name,
                    photo_url,
                });
            });
        register_auth_callback_js(login_cb_js, unsubscribe)
    }

    fn log_out(ctx: &Context<Self>) {
        let logout_cb = ctx.link().callback(|()| Msg::CompleteLogOut);
        let logout_cb_js = Closure::once_into_js(move |_: JsValue| logout_cb.emit(()));
        log_out_js(logout_cb_js);
    }

    fn get_all_user_projects(ctx: &Context<Self>) {
        let projects_cb = ctx.link().callback(Msg::ProjectsFetched);
        let projects_cb_js = Closure::once_into_js(move |projects: JsValue| {
            let mut projects: ProjectCollection = serde_wasm_bindgen::from_value(projects).unwrap();
            projects
                .personal
                .sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
            projects
                .published
                .sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
            projects_cb.emit(projects);
        });
        get_user_projects_js(None::<u8>.into(), projects_cb_js);
    }

    fn user_projects_list(&self, ctx: &Context<Self>) -> Html {
        let current_project_id = ctx
            .props()
            .remote_project_metadata
            .as_ref()
            .map(|metadata| metadata.id.clone());

        let projects_list = if self.projects_view == ProjectsView::Personal {
            &self.projects.personal
        } else {
            &self.projects.published
        };

        let list_items = projects_list
            .iter()
            .map(|project| {
                let project_id = project.id.clone();
                let title = if project.title.is_empty() {
                    "(Untitled)".to_owned()
                } else {
                    project.title.clone()
                };
                let author = if project.author.is_empty() {
                    "(No Author)".to_owned()
                } else {
                    project.author.clone()
                };
                let lm = format!("Updated: {}", Self::format_timestamp(project.last_modified));
                let update_visibility_cb = {
                    let project = project.clone();
                    ctx.link().callback(move |e: MouseEvent| {
                        e.stop_propagation();
                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                        let new_visibility = if input.checked() {
                            ProjectVisibility::Public
                        } else {
                            ProjectVisibility::Private
                        };
                        let update = ProjectMetadataUpdate {
                            id: project.id.clone(),
                            visibility: Some(new_visibility),
                            ..Default::default()
                        };
                        Msg::UpdateProjectMetadata(update)
                    })
                };
                let open_cb = {
                    let project = project.clone();
                    ctx.link()
                        .callback(move |_| Msg::OpenProject(project.clone()))
                };
                let li_class = if matches!(current_project_id, Some(ref id) if &project_id == id) {
                    "account__project-list-item account__project-list-item-current"
                } else {
                    "account__project-list-item"
                };
                html! {
                    <li class={li_class} onclick={open_cb}>
                        <div class="account__project-list-item-title">{title}</div>
                        <div class="account__project-list-item-author">{author}</div>
                        <div class="account__project-list-item-visibility">
                            <div>{"Public"}</div>
                            <input
                                type="checkbox"
                                checked={project.visibility == ProjectVisibility::Public}
                                onclick={update_visibility_cb}
                            />
                        </div>
                        <div class="account__project-list-item-id">{project.id.clone()}</div>
                        <div class="account__project-list-item-lm">{lm}</div>
                    </li>
                }
            })
            .collect::<Vec<Html>>();

        if list_items.is_empty() && self.status != OperationStatus::Updating {
            html! {
                <div class="account__no-projects-msg">{"No projects yet"}</div>
            }
        } else {
            html! {
                <ul class="account__project-list">
                    {list_items}
                </ul>
            }
        }
    }

    fn format_timestamp(ts: u64) -> String {
        // let naive = NaiveDateTime::from_timestamp_opt(ts, 0);
        // let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        let d = UNIX_EPOCH + Duration::from_secs(ts);
        let dt = DateTime::<Utc>::from(d);
        dt.format("%F %H:%M").to_string()
    }

    fn save_project(&self, ctx: &Context<Self>) {
        if let Some(remote_metadata) = ctx.props().remote_project_metadata.clone() {
            // Remote project is currently open
            let save_cb = ctx.link().callback(Msg::ProjectSaved);
            let save_cb_js = Closure::once_into_js(move |saved| {
                save_cb.emit(serde_wasm_bindgen::from_value(saved).unwrap());
            });

            let proof = &ctx.props().proof;
            let mut metadata = proof.metadata.clone();

            // Author is filled in with user's name if applicable
            metadata.author = metadata.author.or_else(|| {
                self.user
                    .as_ref()
                    .map(|userdata| userdata.display_name.clone())
            });

            let blob = model::serialize::serialize(
                proof.signature.clone(),
                proof.workspace.clone(),
                proof.metadata.clone(),
            );

            let args = SaveProjectArgs {
                id: Some(remote_metadata.id),
                blob,
                title: metadata.title.clone().unwrap_or_default(),
                author: metadata.author.clone().unwrap_or_default(),
                abstr: metadata.abstr.unwrap_or_default(),
                visibility: remote_metadata.visibility.to_string(),
            };
            save_project_js(args.into(), save_cb_js);
        } else {
            tracing::error!("Cannot save a project which is not open (try \"save as\")");
        }
    }

    fn save_new_project(&self, ctx: &Context<Self>) {
        let save_cb = ctx.link().callback(Msg::ProjectSaved);
        let save_cb_js = Closure::once_into_js(move |saved| {
            save_cb.emit(serde_wasm_bindgen::from_value(saved).unwrap());
        });

        let proof = &ctx.props().proof;
        let mut metadata = proof.metadata.clone();

        // Author is filled in with user's name if applicable
        metadata.author = metadata.author.or_else(|| {
            self.user
                .as_ref()
                .map(|userdata| userdata.display_name.clone())
        });

        let blob = model::serialize::serialize(
            proof.signature.clone(),
            proof.workspace.clone(),
            metadata.clone(),
        );

        let args = SaveProjectArgs {
            id: None,
            blob,
            title: metadata.title.clone().unwrap_or_default(),
            author: metadata.clone().author.unwrap_or_default(),
            abstr: metadata.abstr.unwrap_or_default(),
            visibility: ProjectVisibility::default().to_string(), /* This should be decided by dropdown. */
        };
        save_project_js(args.into(), save_cb_js);
    }

    fn publish_project(&self, ctx: &Context<Self>) {
        let publish_cb = ctx.link().callback(Msg::ProjectPublished);
        let publish_cb_js = Closure::once_into_js(move |published| {
            publish_cb.emit(serde_wasm_bindgen::from_value(published).unwrap());
        });

        let proof = &ctx.props().proof;
        let mut metadata = proof.metadata.clone();

        // Author is filled in with user's name if applicable
        metadata.author = metadata.author.or_else(|| {
            self.user
                .as_ref()
                .map(|userdata| userdata.display_name.clone())
        });

        let blob = model::serialize::serialize(
            proof.signature.clone(),
            proof.workspace.clone(),
            metadata.clone(),
        );

        let id = if let Some(remote_metadata) = &ctx.props().remote_project_metadata {
            if remote_metadata.visibility == ProjectVisibility::Published {
                Some(remote_metadata.id.clone())
            } else {
                None
            }
        } else {
            None
        };

        let args = SaveProjectArgs {
            id,
            blob,
            title: metadata.title.clone().unwrap_or_default(),
            author: metadata.author.clone().unwrap_or_default(),
            abstr: metadata.abstr.unwrap_or_default(),
            visibility: ProjectVisibility::Published.to_string(),
        };
        save_project_js(args.into(), publish_cb_js);
    }

    fn update_project_metadata(&mut self, ctx: &Context<Self>, update: &ProjectMetadataUpdate) {
        let up_cb = ctx.link().callback(Msg::ProjectMetadataUpdated);
        let up_cb_js = Closure::once_into_js(move |updated| {
            up_cb.emit(serde_wasm_bindgen::from_value(updated).unwrap());
        });
        update_project_metadata_js(update.clone().into(), up_cb_js);

        for p in &mut self.projects.personal {
            if p.id == update.id {
                if let Some(ref title) = update.title {
                    p.title = title.clone();
                }
                if let Some(ref author) = update.author {
                    p.author = author.clone();
                }
                if let Some(ref abstr) = update.abstr {
                    p.abstr = abstr.clone();
                }
                if let Some(ref visibility) = update.visibility {
                    if visibility != &ProjectVisibility::Published {
                        p.visibility = visibility.clone();
                    }
                }
            }
        }
    }

    fn delete_project(ctx: &Context<Self>, id: String) {
        let dp_cb = ctx.link().callback(Msg::ProjectDeleted);
        let dp_cb_js = Closure::once_into_js(move |deleted| {
            dp_cb.emit(serde_wasm_bindgen::from_value(deleted).unwrap());
        });

        let args = DeleteProjectArgs { id };
        delete_project_js(args.into(), dp_cb_js);
    }

    fn download_project(ctx: &Context<Self>, project: &RemoteProjectMetadata) {
        tracing::debug!("Downloading {}", project.id);
        let id = project.id.clone();
        let published = project.visibility == ProjectVisibility::Published;
        let dl_cb = ctx.link().callback(Msg::ProjectDownloaded);

        download_project_with_id(None, id, published, dl_cb);
    }
}

pub fn download_project_with_id(
    uid: Option<String>,
    id: String,
    published: bool,
    cb: yew::Callback<(RemoteProjectMetadata, Vec<u8>)>,
) {
    let cb_js = Closure::once_into_js(move |metadata_and_blob: JsValue| {
        let pair: Option<(RemoteProjectMetadata, Vec<u8>)> =
            serde_wasm_bindgen::from_value(metadata_and_blob).unwrap();
        if let Some((metadata, blob)) = pair {
            cb.emit((metadata, blob));
        }
    });

    let args = DownloadProjectArgs {
        uid,
        id,
        published,
        specific_version: None,
    };
    download_project_js(args.into(), cb_js);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    display_name: String,
    photo_url: Option<String>,
    //email: String,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct ProjectCollection {
    personal: Vec<RemoteProjectMetadata>,
    published: Vec<RemoteProjectMetadata>,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SaveProjectArgs {
    id: Option<String>,
    blob: Vec<u8>,
    title: String,
    author: String,
    abstr: String,
    visibility: String,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct DeleteProjectArgs {
    id: String,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct DownloadProjectArgs {
    uid: Option<String>,
    id: String,
    published: bool,
    specific_version: Option<u32>,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectMetadataUpdate {
    id: String,
    title: Option<String>,
    author: Option<String>,
    abstr: Option<String>,
    visibility: Option<ProjectVisibility>,
}

#[wasm_bindgen]
impl SaveProjectArgs {
    pub fn id(&self) -> Option<String> {
        self.id.clone()
    }

    pub fn blob(&self) -> Vec<u8> {
        self.blob.clone()
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn author(&self) -> String {
        self.author.clone()
    }

    pub fn abstr(&self) -> String {
        self.abstr.clone()
    }

    pub fn visibility(&self) -> String {
        self.visibility.clone()
    }
}

#[wasm_bindgen]
impl DeleteProjectArgs {
    pub fn id(&self) -> String {
        self.id.clone()
    }
}

#[wasm_bindgen]
impl DownloadProjectArgs {
    pub fn uid(&self) -> Option<String> {
        self.uid.clone()
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn published(&self) -> bool {
        self.published
    }

    #[wasm_bindgen(js_name = "specificVersion")]
    pub fn specific_version(&self) -> Option<u32> {
        self.specific_version
    }
}

#[wasm_bindgen]
impl ProjectMetadataUpdate {
    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn title(&self) -> Option<String> {
        self.title.clone()
    }

    pub fn author(&self) -> Option<String> {
        self.author.clone()
    }

    #[wasm_bindgen(js_name = "abstract")]
    pub fn abstr(&self) -> Option<String> {
        self.abstr.clone()
    }

    pub fn visibility(&self) -> Option<String> {
        self.visibility.as_ref().map(ToString::to_string)
    }
}

#[wasm_bindgen(module = "/src/app/account/account_script.js")]
extern "C" {
    #[wasm_bindgen(js_name = "initUI")]
    pub fn init_ui();

    #[wasm_bindgen(js_name = "logOut")]
    pub fn log_out_js(callback: JsValue);

    #[wasm_bindgen(js_name = "registerAuthCallback")]
    pub fn register_auth_callback_js(logInCallback: JsValue, unsubscribe: JsValue) -> JsValue;

    #[wasm_bindgen(js_name = "getUserProjects")]
    pub fn get_user_projects_js(maybeProject: JsValue, projectsCallback: JsValue) -> JsValue;

    #[wasm_bindgen(js_name = "saveProject")]
    pub fn save_project_js(args: JsValue, saveCallback: JsValue) -> JsValue;

    #[wasm_bindgen(js_name = "updateProjectMetadata")]
    pub fn update_project_metadata_js(args: JsValue, updateCallback: JsValue) -> JsValue;

    #[wasm_bindgen(js_name = "deleteProject")]
    pub fn delete_project_js(args: JsValue, deleteCallback: JsValue) -> JsValue;

    #[wasm_bindgen(js_name = "downloadProject")]
    pub fn download_project_js(args: JsValue, downloadCallback: JsValue) -> JsValue;
}
