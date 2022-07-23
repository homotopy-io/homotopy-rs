#![allow(clippy::enum_variant_names)]
#![allow(clippy::use_self)]

use homotopy_common::tree::Tree;
use homotopy_core::{
    common::{Generator, SliceIndex},
    serialize::{Key, Store},
    Diagram,
};
use homotopy_graphics::style::Color;
use im::Vector;
use obake::AnyVersion;
use wasm_bindgen::JsCast;

use super::{
    proof::{generators::GeneratorInfo, SignatureItem, View},
    Signature, Workspace,
};

pub fn generate_download(name: &str, ext: &str, data: &[u8]) -> Result<(), wasm_bindgen::JsValue> {
    let val: js_sys::Uint8Array = data.into();
    let mut options = web_sys::BlobPropertyBag::new();
    options.type_("application/msgpack");
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
        &js_sys::Array::of1(&val.into()).into(),
        &options,
    )?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let body = document.body().ok_or("no body")?;
    let e = document.create_element("a")?;
    let a = e
        .dyn_ref::<web_sys::HtmlElement>()
        .ok_or("failed to create anchor")?;
    a.set_attribute("href", &url)?;
    a.set_attribute("download", &format!("{}.{}", &name, &ext))?;
    body.append_child(a)?;
    a.click();
    a.remove();
    web_sys::Url::revoke_object_url(&url)
}

#[obake::versioned]
#[obake(version("0.1.0"))]
#[obake(version("0.1.2"))]
#[obake(derive(serde::Serialize, serde::Deserialize))]
#[obake(serde(untagged))]
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
struct WorkspaceData {
    diagram: Key<Diagram>,
    path: Vector<SliceIndex>,
    #[obake(cfg(">=0.1.2"))]
    view: View,
}

impl From<WorkspaceData!["0.1.0"]> for WorkspaceData!["0.1.2"] {
    fn from(from: WorkspaceData!["0.1.0"]) -> Self {
        Self {
            diagram: from.diagram,
            path: from.path,
            view: Default::default(),
        }
    }
}

#[obake::versioned]
#[obake(version("0.1.0"))]
#[obake(version("0.1.1"))]
#[obake(version("0.1.2"))]
#[obake(derive(serde::Serialize, serde::Deserialize))]
#[obake(serde(tag = "version"))]
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Data {
    store: Store,
    #[obake(cfg("<0.1.1"))]
    signature: Vec<GeneratorData>,
    #[obake(cfg(">=0.1.1"))]
    signature: Tree<SignatureData>,
    #[obake(cfg("<0.1.2"))]
    workspace: Option<WorkspaceData!["0.1.0"]>,
    #[obake(cfg(">=0.1.2"))]
    workspace: Option<WorkspaceData!["0.1.2"]>,
}

impl From<Data!["0.1.0"]> for Data!["0.1.1"] {
    fn from(from: Data!["0.1.0"]) -> Self {
        Self {
            store: from.store,
            signature: from
                .signature
                .into_iter()
                .map(SignatureData::Item)
                .collect(),
            workspace: from.workspace,
        }
    }
}

impl From<Data!["0.1.1"]> for Data!["0.1.2"] {
    fn from(from: Data!["0.1.1"]) -> Self {
        Self {
            store: from.store,
            signature: from.signature,
            workspace: from.workspace.map(Into::into),
        }
    }
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data").finish()
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
enum SignatureData {
    Folder(String, bool),
    Item(GeneratorData),
}

impl Default for SignatureData {
    fn default() -> Self {
        Self::Folder("".to_owned(), true)
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct GeneratorData {
    generator: Generator,
    name: String,
    color: Color,
    diagram: Key<Diagram>,
}

pub fn serialize(signature: Signature, workspace: Option<Workspace>) -> Vec<u8> {
    let mut data = Data {
        store: Default::default(),
        signature: Default::default(),
        workspace: Default::default(),
    };

    let mut signature = signature.into_tree();
    // Remove noise from signature tree
    signature.clean_up();
    // Pack signature data
    data.signature = signature.map(|item| match item {
        SignatureItem::Folder(name, open) => SignatureData::Folder(name, open),
        SignatureItem::Item(info) => SignatureData::Item(GeneratorData {
            generator: info.generator,
            diagram: data.store.pack_diagram(&info.diagram),
            name: info.name,
            color: info.color,
        }),
    });

    if let Some(workspace) = workspace {
        data.workspace = Some(WorkspaceData {
            diagram: data.store.pack_diagram(&workspace.diagram),
            path: workspace.path,
            view: workspace.view,
        });
    }

    // Tag data with version
    let data: AnyVersion<Data> = data.into();
    // Serialize
    rmp_serde::encode::to_vec_named(&data).unwrap()
}

pub fn deserialize(data: &[u8]) -> Option<(Signature, Option<Workspace>)> {
    // Deserialize with version tag
    let data: AnyVersion<Data> = match rmp_serde::decode::from_slice(data) {
        Err(error) => {
            log::error!("Error while deserializing: {}", error);
            None
        }
        Ok(data) => Some(data),
    }?;
    // Migrate to current version
    let data: Data = data.into();
    let mut store = data.store;

    let signature = data
        .signature
        .map(|s| {
            Some(match s {
                SignatureData::Folder(name, open) => SignatureItem::Folder(name, open),
                SignatureData::Item(gd) => SignatureItem::Item(GeneratorInfo {
                    generator: gd.generator,
                    name: gd.name,
                    color: gd.color,
                    shape: Default::default(),
                    diagram: store.unpack_diagram(gd.diagram)?,
                    framed: true,
                    invertible: false,
                }),
            })
        })
        .transpose()?
        .into();

    let mut workspace = None;
    if let Some(workspace_data) = data.workspace {
        workspace = Some(Workspace {
            diagram: store.unpack_diagram(workspace_data.diagram)?,
            path: workspace_data.path,
            view: workspace_data.view,
            attach: Default::default(),
            attachment_highlight: Default::default(),
            slice_highlight: Default::default(),
        });
    }

    Some((signature, workspace))
}
