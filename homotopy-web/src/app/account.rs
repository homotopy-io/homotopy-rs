use std::{
    fmt,
    ops::Deref,
    time::{Duration, UNIX_EPOCH},
};

use chrono::{DateTime, Utc};
use futures::future::join_all;
use homotopy_model::proof::ProofState;
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use yew::prelude::*;

use crate::{
    components::{
        icon::{Icon, IconSize},
        toast::{toast, Toast, ToastKind},
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
    pub updated: u64,
    pub version: Option<u64>,
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
    LogOut,
    SetProjectsView(ProjectsView),
    FetchProjects,
    ProjectsFetched(ProjectCollection),
    SaveProject,
    SaveNewProject,
    ProjectSaved(Option<RemoteProjectMetadata>),
    PublishProject(String),
    ProjectPublished(Option<PublishResult>),
    PublishProjectVersion(String),
    SetPublic(String, bool),
    ProjectPublicChanged(String, bool),
    DeleteProject(String),
    ProjectDeleted(String),
    OpenPersonalProject(String, String),
    OpenPublishedProject(String, u64),
    ProjectDownloaded((RemoteProjectMetadata, Vec<u8>)),
    Resolved(Toast),
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
    projects: ProjectCollection,
    projects_view: ProjectsView,
    status: OperationStatus,
}

impl Component for AccountView {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().callback(|()| Msg::FetchProjects).emit(());
        tracing::debug!("creating account view");

        Self {
            projects: ProjectCollection::default(),
            projects_view: ProjectsView::Personal,
            status: OperationStatus::Updating,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(userdata) = firebase().auth().current_user() {
            let user_icon = if let Some(url) = &userdata.photo_url() {
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

            let maybe_save = if ctx.props().remote_project_metadata.is_some() {
                html! {
                    <button onclick={ctx.link().callback(|_| Msg::SaveProject)}>{"Save"}</button>
                }
            } else {
                html! {}
            };

            let maybe_publish = if let Some(metadata) = &ctx.props().remote_project_metadata {
                let id = metadata.id.clone();
                match metadata.visibility {
                    ProjectVisibility::Public => html! {
                        <button onclick={ctx.link().callback(move |_| Msg::PublishProject(id.clone()))}>{"Publish"}</button>
                    },
                    ProjectVisibility::Published
                        if ctx
                            .props()
                            .remote_project_metadata
                            .as_ref()
                            .map(|meta| meta.uid.clone())
                            == firebase().auth().current_user().map(|user| user.uid()) =>
                    {
                        html! {
                            <button onclick={ctx.link().callback(move |_| Msg::PublishProjectVersion(id.clone()))}>{"Publish new version"}</button>
                        }
                    }
                    _ => html! {},
                }
            } else {
                html! {}
            };

            html! {
                <>
                    <div style="color:red">
                        <p>
                            {"Warning: This server functionality is a work in progress. Please do not use this for anything important, and export any workspaces which you wish to save."}
                        </p>
                        <p>
                            {"In this testing period, no guarantees will be made about data retention and the server may be wiped at any point."}
                        </p>
                    </div>
                    <div class="account__user-details">
                        { user_icon }
                        <div class="account__user-name">{userdata.display_name().clone()}</div>
                    </div>
                    <button onclick={ctx.link().callback(|_| Msg::LogOut)}>{"Log out"}</button>
                    <button onclick={ctx.link().callback(|_| Msg::FetchProjects)}>{"Refresh Projects"}</button>
                    {maybe_save}
                    <button onclick={ctx.link().callback(|_| Msg::SaveNewProject)}>{"Save new"}</button>
                    {maybe_publish}
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

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        if firebase().auth().current_user().is_none() {
            let logged_in_cb = ctx
                .link()
                .callback(|()| Msg::Resolved(Toast::success("Logged in")));
            let logged_in_cb_js = Closure::once_into_js(move || {
                logged_in_cb.emit(());
            });
            init_ui(logged_in_cb_js);
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let set_remote_project_metadata = ctx
            .props()
            .dispatch
            .reform(model::Action::SetRemoteProjectMetadata);
        match msg {
            Msg::LogOut => {
                ctx.link().send_future(async {
                    firebase().auth().sign_out().await;
                    Msg::Resolved(Toast::success("Logged out"))
                });
                false
            }
            Msg::SetProjectsView(view) => {
                self.projects_view = view;
                true
            }
            Msg::FetchProjects => {
                self.status = OperationStatus::Updating;
                ctx.link().send_future(async {
                    if let Some(collection) = fetch_user_projects().await {
                        Msg::ProjectsFetched(collection)
                    } else {
                        Msg::Resolved(Toast::error("Failed to fetch projects"))
                    }
                });
                false
            }
            Msg::ProjectsFetched(projects) => {
                self.status = OperationStatus::Succeeded;
                self.projects = projects;
                true
            }
            Msg::SaveProject => {
                if let Some(id) = project_id(ctx) {
                    self.status = OperationStatus::Updating;
                    let proof = ctx.props().proof.inner().deref().clone();
                    ctx.link().send_future_batch(async move {
                        vec![
                            Msg::ProjectSaved(save_project(&id, proof).await),
                            Msg::FetchProjects, // TODO: fetch only the new project instead of everything
                        ]
                    });
                }
                false
            }
            Msg::SaveNewProject => {
                self.status = OperationStatus::Updating;
                let proof = ctx.props().proof.inner().deref().clone();
                ctx.link().send_future_batch(async {
                    let id = fresh_id().await;
                    vec![
                        Msg::ProjectSaved(save_project(&id, proof).await),
                        Msg::FetchProjects, // TODO: fetch only the new project instead of everything
                    ]
                });
                false
            }
            Msg::ProjectSaved(new_remote_metadata) => {
                // Show successful save
                if let Some(metadata) = new_remote_metadata {
                    tracing::debug!("project saved successfully");
                    self.status = OperationStatus::Succeeded;
                    toast(Toast::success("Project saved successfully"));
                    set_remote_project_metadata.emit(Some(metadata));
                } else {
                    self.status = OperationStatus::Failed;
                    toast(Toast::error("Project save failed"));
                }
                true
            }
            Msg::PublishProject(id) => {
                self.status = OperationStatus::Updating;
                ctx.link().send_future(async move {
                    let raw = publish_project(&id).await;
                    tracing::debug!("publish result: {:?}", &raw);
                    let res: Option<Wrapped<PublishResult>> =
                        serde_wasm_bindgen::from_value(raw).ok();
                    Msg::ProjectPublished(res.map(|wrapped| wrapped.data))
                });
                false
            }
            Msg::ProjectPublished(res) => {
                if let Some(res) = res {
                    self.status = OperationStatus::Succeeded;
                    toast(Toast::success(format!(
                        "Project published: {}v{}",
                        res.tag, res.version
                    )));
                    let meta =
                        ctx.props()
                            .remote_project_metadata
                            .as_ref()
                            .cloned()
                            .map(|mut m| {
                                m.id = res.tag;
                                m.visibility = ProjectVisibility::Published;
                                m.version = Some(1);
                                m
                            });
                    set_remote_project_metadata.emit(meta);
                } else {
                    self.status = OperationStatus::Failed;
                    toast(Toast::error("Project publish failed"));
                }
                true
            }
            Msg::PublishProjectVersion(tag) => {
                self.status = OperationStatus::Updating;
                let proof = ctx.props().proof.inner().deref().clone();
                ctx.link().send_future(async move {
                    publish_project_version(&tag, proof).await.unwrap();
                    Msg::Resolved(Toast::success(format!("New version published: {tag}")))
                });
                false
            }
            Msg::SetPublic(id, public) => {
                self.status = OperationStatus::Updating;
                ctx.link().send_future(async move {
                    set_public(&id, public).await;
                    Msg::ProjectPublicChanged(id, public)
                });
                false
            }
            Msg::ProjectPublicChanged(id, public) => {
                self.status = OperationStatus::Succeeded;
                let projects = &mut self.projects.personal;
                if let Some(project) = projects.iter_mut().find(|p| p.id == id) {
                    project.visibility = if public {
                        ProjectVisibility::Public
                    } else {
                        ProjectVisibility::Private
                    };
                }
                if ctx.props().remote_project_metadata.as_ref().map(|m| &m.id) == Some(&id) {
                    let metadata =
                        ctx.props()
                            .remote_project_metadata
                            .as_ref()
                            .cloned()
                            .map(|mut m| {
                                m.visibility = if public {
                                    ProjectVisibility::Public
                                } else {
                                    ProjectVisibility::Private
                                };
                                m
                            });
                    set_remote_project_metadata.emit(metadata);
                }
                toast(Toast::success(format!(
                    "Made project {id} {}",
                    if public { "public" } else { "private" }
                )));
                true
            }
            Msg::DeleteProject(id) => {
                self.status = OperationStatus::Updating;
                ctx.link().send_future(async move {
                    delete_project(&id).await;
                    Msg::ProjectDeleted(id.clone())
                });
                false
            }
            Msg::ProjectDeleted(id) => {
                self.status = OperationStatus::Succeeded;
                self.projects.personal.retain(|p| p.id != id);
                if ctx.props().remote_project_metadata.as_ref().map(|m| &m.id) == Some(&id) {
                    set_remote_project_metadata.emit(None);
                }
                toast(Toast::success(format!("Project {id} deleted",)));
                true
            }
            Msg::OpenPersonalProject(uid, id) => {
                self.status = OperationStatus::Updating;
                ctx.link().send_future(async move {
                    if let Some((project, blob)) = download_personal_project(&uid, &id).await {
                        Msg::ProjectDownloaded((project, blob))
                    } else {
                        Msg::Resolved(Toast::error("Failed to download project"))
                    }
                });
                false
            }
            Msg::OpenPublishedProject(id, version) => {
                self.status = OperationStatus::Updating;
                ctx.link().send_future(async move {
                    if let Some((project, blob)) = download_published_project(&id, version).await {
                        Msg::ProjectDownloaded((project, blob))
                    } else {
                        Msg::Resolved(Toast::error("Failed to download project"))
                    }
                });
                false
            }
            Msg::ProjectDownloaded((remote_project_metadata, blob)) => {
                // TODO: handle failure
                self.status = OperationStatus::Succeeded;
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
                toast(Toast::success("Project downloaded"));
                true
            }
            Msg::Resolved(t) => {
                self.status = if t.kind == ToastKind::Success {
                    OperationStatus::Succeeded
                } else {
                    OperationStatus::Failed
                };
                toast(t);
                true
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PublishResult {
    tag: String,
    version: u64,
}

#[derive(Debug, Deserialize)]
struct Wrapped<T> {
    data: T,
}

impl AccountView {
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
                let lm = format!("Updated: {}", format_timestamp(project.updated));
                let update_visibility_cb = {
                    let id = project_id.clone();
                    ctx.link().callback(move |e: MouseEvent| {
                        e.stop_propagation();
                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                        Msg::SetPublic(id.clone(), input.checked())
                    })
                };
                let delete = {
                    let delete_project_id = project_id.clone();
                    let delete_cb = ctx.link().callback(move |e: MouseEvent| {
                        e.stop_propagation();
                        Msg::DeleteProject(delete_project_id.clone())
                    });
                    html! {
                        <button onclick={delete_cb}>{"Delete"}</button>
                    }
                };
                let open_cb = {
                    let project = project.clone();
                    if project.visibility == ProjectVisibility::Published {
                        ctx.link()
                            .callback(move |_| Msg::OpenPublishedProject(project.id.clone(), project.version.unwrap_or(1)))
                    } else {
                        ctx.link()
                            .callback(move |_| Msg::OpenPersonalProject(project.uid.clone(), project.id.clone()))
                    }
                };
                let li_class = if matches!(current_project_id, Some(ref id) if &project_id == id) {
                    "account__project-list-item account__project-list-item-current"
                } else {
                    "account__project-list-item"
                };
                let delete_or_versions = if project.visibility == ProjectVisibility::Published {
                    // TODO: handle other versions
                    html! {
                        <div class="account__project-list-item-versions">
                            {
                                #[allow(clippy::iter_on_single_items)]
                                [project.version.unwrap_or_default()].iter().map(|v| html! { <span>{format!("v{v}")}</span> }).collect::<Html>()
                            }
                        </div>
                    }
                } else {
                    html! {
                        <div class="account__project-list-item-delete">{delete}</div>
                    }
                };
                let visibility_checkbox = if project.visibility == ProjectVisibility::Published {
                    html! {}
                } else {
                    html! {
                        <div class="account__project-list-item-visibility">
                            <div>{"Public"}</div>
                            <input
                                type="checkbox"
                                checked={project.visibility == ProjectVisibility::Public}
                                onclick={update_visibility_cb}
                            />
                        </div>
                    }
                };
                html! {
                    <li class={li_class} onclick={open_cb}>
                        <div class="account__project-list-item-title">{title}</div>
                        <div class="account__project-list-item-author">{author}</div>
                        {visibility_checkbox}
                        <div class="account__project-list-item-id">{project.id.clone()}</div>
                        <div class="account__project-list-item-lm">{lm}</div>
                        {delete_or_versions}
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
}

fn format_timestamp(seconds: u64) -> String {
    let d = UNIX_EPOCH + Duration::from_secs(seconds);
    let dt = DateTime::<Utc>::from(d);
    dt.format("%F %H:%M").to_string()
}

fn user_id() -> Option<String> {
    firebase().auth().current_user().map(|user| user.uid())
}

fn project_id(ctx: &Context<AccountView>) -> Option<String> {
    ctx.props()
        .remote_project_metadata
        .as_ref()
        .map(|metadata| metadata.id.clone())
}

async fn fresh_id() -> String {
    // get an id from the server
    let uid = user_id().unwrap();
    tracing::debug!("getting fresh project id");
    let docref = firebase()
        .firestore()
        .collection(format!("personal-rs/{uid}/projects").as_str())
        .add(js_sys::Object::new().into())
        .await;
    let x: &DocumentReference = docref.as_ref().unchecked_ref();
    let id = x.id();
    tracing::debug!("got fresh id: {}", id);
    id
}

async fn fetch_personal_project(uid: &str, id: &str) -> Option<RemoteProjectMetadata> {
    let ds = firebase()
        .firestore()
        .doc(&format!("personal-rs/{uid}/projects/{id}"))
        .get()
        .await
        .unchecked_into::<DocumentSnapshot>();
    ds.exists()
        .then(|| {
            let decoded = serde_wasm_bindgen::from_value::<PersonalRecord>(ds.data()).ok()?;
            Some(RemoteProjectMetadata {
                id: id.to_owned(),
                uid: uid.to_owned(),
                title: decoded.title,
                author: decoded.author,
                abstr: decoded.r#abstract,
                visibility: if decoded.public {
                    ProjectVisibility::Public
                } else {
                    ProjectVisibility::Private
                },
                updated: decoded.updated.seconds,
                version: None,
            })
        })
        .flatten()
}

async fn fetch_published_project(id: &str, version: u64) -> Option<RemoteProjectMetadata> {
    let uid = firebase()
        .firestore()
        .doc(&format!("published-rs/{id}"))
        .get()
        .await
        .unchecked_into::<DocumentSnapshot>()
        .get("uid")
        .as_string()?;
    let ds = firebase()
        .firestore()
        .doc(&format!("published-rs/{id}/versions/v{version}"))
        .get()
        .await
        .unchecked_into::<DocumentSnapshot>();
    ds.exists()
        .then(|| {
            let decoded = serde_wasm_bindgen::from_value::<PublishedRecord>(ds.data()).ok()?;
            Some(RemoteProjectMetadata {
                id: id.to_owned(),
                uid,
                title: decoded.title,
                author: decoded.author,
                abstr: decoded.r#abstract,
                visibility: ProjectVisibility::Published,
                updated: decoded.created.seconds,
                version: Some(version),
            })
        })
        .flatten()
}

#[allow(clippy::cognitive_complexity)]
async fn fetch_user_projects() -> Option<ProjectCollection> {
    let uid = user_id()?;
    let personal = firebase()
        .firestore()
        .collection(&format!("personal-rs/{uid}/projects"))
        .order_by("updated", "desc")
        .get()
        .await
        .unchecked_into::<QuerySnapshot>()
        .docs()
        .into_iter()
        .filter_map(|qds| {
            let data = qds.data();
            let decoded = serde_wasm_bindgen::from_value::<PersonalRecord>(data).ok()?;
            Some(RemoteProjectMetadata {
                id: qds.id(),
                uid: uid.clone(),
                title: decoded.title,
                author: decoded.author,
                abstr: decoded.r#abstract,
                visibility: if decoded.public {
                    ProjectVisibility::Public
                } else {
                    ProjectVisibility::Private
                },
                updated: decoded.updated.seconds,
                version: None,
            })
        })
        .collect();
    tracing::debug!("personal collection: {:?}", &personal);

    let published_ids: Vec<String> = serde_wasm_bindgen::from_value(
        firebase()
            .firestore()
            .doc(&format!("personal-rs/{uid}"))
            .get()
            .await
            .unchecked_into::<DocumentSnapshot>()
            .get("published"),
    )
    .unwrap_or_default();
    tracing::debug!("published ids: {:?}", &published_ids);
    let published = join_all(published_ids.into_iter().map(|id| async move {
        let versions = &firebase()
            .firestore()
            .collection(&format!("published-rs/{id}/versions"))
            .order_by("created", "desc")
            .get()
            .await
            .unchecked_into::<QuerySnapshot>()
            .docs();
        versions
            .iter()
            .map(|v| {
                let version = Some(v.id()[1..].parse().ok()?);
                let data = v.data();
                tracing::debug!("data: {:?}", &data);
                let decoded = serde_wasm_bindgen::from_value::<PublishedRecord>(data).ok()?;
                tracing::debug!("decoded: {:?}", &decoded);
                Some(RemoteProjectMetadata {
                    id: id.clone(),
                    uid: user_id()?,
                    title: decoded.title,
                    author: decoded.author,
                    abstr: decoded.r#abstract,
                    visibility: ProjectVisibility::Published,
                    updated: decoded.created.seconds,
                    version,
                })
            })
            .collect::<Vec<_>>()
    }))
    .await
    .into_iter()
    .flatten()
    .flatten()
    .collect();
    tracing::debug!("published collection: {:?}", &published);

    Some(ProjectCollection {
        personal,
        published,
    })
}

#[derive(Debug, Serialize)]
struct UploadMetadata {
    #[serde(rename = "contentType")]
    content_type: &'static str,
    #[serde(rename = "customMetadata")]
    custom_metadata: CustomMetadata,
}
#[derive(Debug, Serialize)]
struct CustomMetadata {
    title: String,
    author: String,
    r#abstract: String,
}

async fn save_project(id: &str, proof: ProofState) -> Option<RemoteProjectMetadata> {
    // get destination path
    tracing::debug!("Saving project {id}");
    let uid = user_id().unwrap();
    let path = format!("personal-rs/{uid}/projects/{id}.hom");
    let storage = firebase().storage();
    let storageref = storage.storage_ref(&path);
    tracing::debug!("got path: {:?}", storageref);

    let mut metadata = proof.metadata;

    // Author is filled in with user's name if applicable
    metadata.author = metadata.author.or_else(|| {
        firebase()
            .auth()
            .current_user()
            .as_ref()
            .and_then(User::display_name)
    });

    let blob = model::serialize::serialize(proof.signature, proof.workspace, metadata.clone());

    let upload_metadata = UploadMetadata {
        content_type: "application/msgpack",
        custom_metadata: CustomMetadata {
            title: metadata.title.clone().unwrap_or_default(),
            author: metadata.author.clone().unwrap_or_default(),
            r#abstract: metadata.abstr.clone().unwrap_or_default(),
        },
    };

    tracing::debug!("upload metadata: {:?}", upload_metadata);
    let task = storageref.put_with_metadata(
        Uint8Array::from(blob.as_slice()).into(),
        serde_wasm_bindgen::to_value(&upload_metadata).expect("failed to set custom metadata"),
    );
    task.then(JsValue::null(), JsValue::null()).await;

    Some(RemoteProjectMetadata {
        id: id.to_owned(),
        uid: uid.clone(),
        title: metadata.title.unwrap_or_default(),
        author: metadata.author.unwrap_or_default(),
        abstr: metadata.abstr.unwrap_or_default(),
        visibility: ProjectVisibility::Private,
        updated: 0,
        version: None,
    })
}

async fn publish_project_version(tag: &str, proof: ProofState) -> Option<()> {
    // get destination path
    let path = format!("published-rs/{tag}/versions/new.hom");
    let storage = firebase().storage();
    let storageref = storage.storage_ref(&path);
    tracing::debug!("got path: {:?}", storageref);

    let mut metadata = proof.metadata;

    // Author is filled in with user's name if applicable
    metadata.author = metadata.author.or_else(|| {
        firebase()
            .auth()
            .current_user()
            .as_ref()
            .and_then(User::display_name)
    });

    let blob = model::serialize::serialize(proof.signature, proof.workspace, metadata.clone());

    let upload_metadata = UploadMetadata {
        content_type: "application/msgpack",
        custom_metadata: CustomMetadata {
            title: metadata.title.clone().unwrap_or_default(),
            author: metadata.author.clone().unwrap_or_default(),
            r#abstract: metadata.abstr.clone().unwrap_or_default(),
        },
    };

    tracing::debug!("upload metadata: {:?}", upload_metadata);
    let task = storageref.put_with_metadata(
        Uint8Array::from(blob.as_slice()).into(),
        serde_wasm_bindgen::to_value(&upload_metadata).expect("failed to set custom metadata"),
    );
    task.then(JsValue::null(), JsValue::null()).await;

    Some(())
}

async fn delete_project(id: &str) {
    tracing::debug!("Deleting {id}");
    let uid = user_id().unwrap();
    let path = format!("personal-rs/{uid}/projects/{id}.hom");
    let storage = firebase().storage();
    let storageref = storage.storage_ref(&path);
    tracing::debug!("got path: {:?}", storageref);

    storageref.delete().await;
}

pub(super) async fn download_personal_project(
    uid: &str,
    id: &str,
) -> Option<(RemoteProjectMetadata, Vec<u8>)> {
    tracing::debug!("Downloading {id}");
    let path = format!("personal-rs/{uid}/projects/{id}.hom");
    let storage = firebase().storage();
    let storageref = storage.storage_ref(&path);
    let url = storageref
        .get_download_url()
        .await
        .ok()?
        .as_string()
        .unwrap();
    tracing::debug!("got download URL: {url}");

    let blob = download(&url).await?;
    let project = fetch_personal_project(uid, id).await?;
    Some((project, blob))
}

pub(super) async fn download_published_project(
    tag: &str,
    version: u64,
) -> Option<(RemoteProjectMetadata, Vec<u8>)> {
    tracing::debug!("Downloading {tag}v{version}");
    let path = format!("published-rs/{tag}/versions/v{version}.hom");
    let storage = firebase().storage();
    let storageref = storage.storage_ref(&path);
    let url = storageref
        .get_download_url()
        .await
        .ok()?
        .as_string()
        .unwrap();
    tracing::debug!("got download URL: {url}");

    let blob = download(&url).await?;
    let project = fetch_published_project(tag, version).await?;
    Some((project, blob))
}

async fn download(url: &str) -> Option<Vec<u8>> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("GET");
    opts.mode(web_sys::RequestMode::Cors);
    let req = web_sys::Request::new_with_str_and_init(url, &opts).unwrap();
    req.headers().set("Accept", "application/msgpack").unwrap();
    let window = web_sys::window().unwrap();
    let res: web_sys::Response = JsFuture::from(window.fetch_with_request(&req))
        .await
        .unwrap()
        .unchecked_into();

    let array_buffer = JsFuture::from(res.array_buffer().ok()?).await.ok()?;
    Some(Uint8Array::new(&array_buffer).to_vec())
}

async fn set_public(id: &str, public: bool) {
    tracing::debug!("Making {id} {}", if public { "public" } else { "private" });
    let uid = user_id().unwrap();
    let path = format!("personal-rs/{uid}/projects/{id}");
    let docref = firebase().firestore().doc(&path);
    docref.update("public", public.into()).await;
    tracing::debug!("done setting public");
}

#[wasm_bindgen]
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct ProjectCollection {
    personal: Vec<RemoteProjectMetadata>,
    published: Vec<RemoteProjectMetadata>,
}

#[derive(Debug, Deserialize)]
struct Timestamp {
    seconds: u64,
    _nanoseconds: u64,
}

// metadata that exists in firestore
#[derive(Debug, Deserialize)]
struct PersonalRecord {
    title: String,
    author: String,
    r#abstract: String,
    public: bool,
    _created: Timestamp,
    updated: Timestamp,
}

#[derive(Debug, Deserialize)]
struct PublishedRecord {
    title: String,
    author: String,
    r#abstract: String,
    created: Timestamp,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.uid() == other.uid()
    }
}

impl Eq for User {}

#[wasm_bindgen(module = "/src/app/account/account_script.js")]
extern "C" {
    #[derive(Debug, Clone)]
    pub type Firebase;

    #[wasm_bindgen(js_name = "getFirebase")]
    pub fn firebase() -> Firebase;

    #[wasm_bindgen(method)]
    pub fn auth(this: &Firebase) -> Auth;

    #[wasm_bindgen(method)]
    pub fn firestore(this: &Firebase) -> Firestore;

    #[wasm_bindgen(method)]
    pub fn storage(this: &Firebase) -> Storage;

    // firebase.auth
    #[derive(Debug, Clone)]
    pub type Auth;

    #[wasm_bindgen(method, getter, js_name = "currentUser")]
    pub fn current_user(this: &Auth) -> Option<User>;

    #[wasm_bindgen(method, js_name = "signOut")]
    pub async fn sign_out(this: &Auth);

    #[derive(Debug, Clone)]
    pub type User;

    #[wasm_bindgen(method, getter, js_name = "displayName")]
    pub fn display_name(this: &User) -> Option<String>;

    #[wasm_bindgen(method, getter)]
    pub fn email(this: &User) -> Option<String>;

    #[wasm_bindgen(method, getter, js_name = "photoURL")]
    pub fn photo_url(this: &User) -> Option<String>;

    #[wasm_bindgen(method, getter)]
    pub fn uid(this: &User) -> String;

    // firebase.firestore
    #[derive(Debug, Clone)]
    pub type Firestore;

    #[wasm_bindgen(method)]
    pub fn collection(this: &Firestore, collectionPath: &str) -> CollectionReference;

    #[wasm_bindgen(method)]
    pub fn doc(this: &Firestore, documentPath: &str) -> DocumentReference;

    #[derive(Debug, Clone)]
    pub type CollectionReference;

    #[wasm_bindgen(method)]
    pub async fn add(this: &CollectionReference, data: JsValue) -> JsValue /* DocumentReference */;

    #[wasm_bindgen(method)]
    pub async fn get(this: &CollectionReference) -> JsValue /* QuerySnapshot */;

    #[wasm_bindgen(method, js_name = "orderBy")]
    pub fn order_by(this: &CollectionReference, fieldPath: &str, directionStr: &str) -> Query;

    #[wasm_bindgen(method)]
    pub fn r#where(
        this: &CollectionReference,
        fieldPath: &str,
        opStr: &str,
        value: JsValue,
    ) -> Query;

    #[derive(Debug, Clone)]
    pub type Query;

    #[wasm_bindgen(method)]
    pub async fn get(this: &Query) -> JsValue /* QuerySnapshot */;

    #[wasm_bindgen(method, js_name = "orderBy")]
    pub fn order_by(this: &Query, fieldPath: &str, directionStr: &str) -> Query;

    #[derive(Debug, Clone)]
    pub type DocumentReference;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &DocumentReference) -> String;

    #[wasm_bindgen(method)]
    pub async fn get(this: &DocumentReference) -> JsValue /* DocumentSnapshot */;

    #[wasm_bindgen(method)]
    pub async fn update(this: &DocumentReference, field: &str, value: JsValue);

    #[derive(Debug, Clone)]
    pub type DocumentSnapshot;

    #[wasm_bindgen(method, getter)]
    pub fn exists(this: &DocumentSnapshot) -> bool;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &DocumentSnapshot) -> String;

    #[wasm_bindgen(method)]
    pub fn data(this: &DocumentSnapshot) -> JsValue;

    #[wasm_bindgen(method)]
    pub fn get(this: &DocumentSnapshot, fieldPath: &str) -> JsValue;

    #[derive(Debug, Clone)]
    pub type QuerySnapshot;

    #[wasm_bindgen(method, getter)]
    pub fn docs(this: &QuerySnapshot) -> Vec<QueryDocumentSnapshot>;

    #[derive(Debug, Clone)]
    pub type QueryDocumentSnapshot;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &QueryDocumentSnapshot) -> String;

    #[wasm_bindgen(method)]
    pub fn data(this: &QueryDocumentSnapshot) -> JsValue;

    // firebase.storage
    #[derive(Debug, Clone)]
    pub type Storage;

    #[wasm_bindgen(method, js_name = "ref")]
    pub fn storage_ref(this: &Storage, path: &str) -> Reference;

    #[derive(Debug, Clone)]
    pub type Reference;

    #[wasm_bindgen(method)]
    pub async fn delete(this: &Reference);

    #[wasm_bindgen(method, catch, js_name = "getDownloadURL")]
    pub async fn get_download_url(this: &Reference) -> Result<JsValue, JsValue> /* String */;

    #[wasm_bindgen(method)]
    pub fn put(this: &Reference, data: JsValue) -> UploadTask;

    #[wasm_bindgen(method, js_name = "put")]
    pub fn put_with_metadata(this: &Reference, data: JsValue, metadata: JsValue) -> UploadTask;

    #[derive(Debug, Clone)]
    pub type UploadTask;

    #[wasm_bindgen(method)]
    pub async fn then(this: &UploadTask, onFulfilled: JsValue, onRejected: JsValue) -> JsValue;

    // firebaseui
    #[wasm_bindgen(js_name = "initUI")]
    pub fn init_ui(onSignInSuccessWithAuthResult: JsValue);

    #[wasm_bindgen(js_name = "publishPersonal")]
    pub async fn publish_project(id: &str) -> JsValue;
}
