use im::Vector;
use wasm_bindgen::JsCast;

use homotopy_common::tree::Tree;

use homotopy_core::common::{Generator, SliceIndex};
use homotopy_core::serialize::{Key, Store};
use homotopy_core::Diagram;

use super::{proof::SignatureItem, Color, GeneratorInfo, Signature, Workspace};

pub fn generate_download(name: &str, data: &[u8]) -> Result<(), wasm_bindgen::JsValue> {
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
    a.set_attribute("download", &format!("{}.hom", &name))?;
    body.append_child(a)?;
    a.click();
    a.remove();
    web_sys::Url::revoke_object_url(&url)
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
struct Data {
    version: usize,
    store: Store,
    signature: Tree<SignatureData>,
    workspace: Option<WorkspaceData>,
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

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
struct WorkspaceData {
    diagram: Key<Diagram>,
    path: Vector<SliceIndex>,
}

pub fn serialize(signature: Signature, workspace: Option<Workspace>) -> Vec<u8> {
    let mut data = Data {
        version: 0,
        store: Store::new(),
        signature: Default::default(),
        workspace: Default::default(),
    };

    data.signature = signature.into_tree().map(|item| match item {
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
        });
    }

    rmp_serde::encode::to_vec_named(&data).unwrap()
}

pub fn deserialize(data: &[u8]) -> Option<(Signature, Option<Workspace>)> {
    let data: Data = match rmp_serde::decode::from_slice(data) {
        Err(error) => {
            log::error!("Error while deserializing: {}", error);
            None
        }
        Ok(data) => Some(data),
    }?;
    let store = data.store;

    let signature = data
        .signature
        .map(|s| Some(match s {
            SignatureData::Folder(name, open) => SignatureItem::Folder(name, open),
            SignatureData::Item(gd) => SignatureItem::Item(GeneratorInfo {
                generator: gd.generator,
                name: gd.name,
                color: gd.color,
                diagram: store.unpack_diagram(gd.diagram)?,
            }),
        }))
        .transpose()?
        .into();

    let mut workspace = None;
    if let Some(workspace_data) = data.workspace {
        workspace = Some(Workspace {
            diagram: store.unpack_diagram(workspace_data.diagram)?,
            path: workspace_data.path,
            attach: Default::default(),
            attachment_highlight: Default::default(),
            slice_highlight: Default::default(),
        });
    }

    Some((signature, workspace))
}
