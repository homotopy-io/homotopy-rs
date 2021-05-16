use std::io::Read;

use super::{Color, GeneratorInfo, Signature, Workspace};
use flate2::{
    read::{GzDecoder, GzEncoder},
    Compression,
};
use homotopy_core::common::{Generator, SliceIndex};
use homotopy_core::serialize::{Key, Store};
use homotopy_core::Diagram;
use im::Vector;
use wasm_bindgen::JsCast;

pub fn generate_download(name: &str, data: &[u8]) -> Result<(), wasm_bindgen::JsValue> {
    let val: js_sys::Uint8Array = data.into();
    let mut options = web_sys::BlobPropertyBag::new();
    options.type_("application/gzip");
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
    body.append_child(&a)?;
    a.click();
    a.remove();
    web_sys::Url::revoke_object_url(&url)
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
struct Data {
    version: usize,
    store: Store,
    signature: Vec<GeneratorData>,
    workspace: Option<WorkspaceData>,
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data").finish()
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct GeneratorData {
    name: String,
    generator: Generator,
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

    for (generator, info) in signature {
        // Since we iterate over an `OrdMap` the vector is sorted by the generator id.
        data.signature.push(GeneratorData {
            generator,
            diagram: data.store.pack_diagram(&info.diagram),
            name: info.name,
            color: info.color,
        });
    }

    if let Some(workspace) = workspace {
        data.workspace = Some(WorkspaceData {
            diagram: data.store.pack_diagram(&workspace.diagram),
            path: workspace.path,
        });
    }

    let json = serde_json::to_string(&data).unwrap();
    let mut bytes = Vec::new();
    GzEncoder::new(json.as_bytes(), Compression::fast())
        .read_to_end(&mut bytes)
        .unwrap();
    bytes
}

pub fn deserialize(data: &[u8]) -> Option<(Signature, Option<Workspace>)> {
    let data: Data = {
        let mut json = String::new();
        GzDecoder::new(data).read_to_string(&mut json).ok()?;
        serde_json::from_str(&json).ok()?
    };

    let mut signature = Signature::default();
    let mut workspace = None;

    for generator_data in data.signature {
        signature.insert(
            generator_data.generator,
            GeneratorInfo {
                name: generator_data.name,
                color: generator_data.color,
                diagram: data.store.unpack_diagram(generator_data.diagram)?,
            },
        );
    }

    if let Some(workspace_data) = data.workspace {
        workspace = Some(Workspace {
            diagram: data.store.unpack_diagram(workspace_data.diagram)?,
            path: workspace_data.path,
            attach: Default::default(),
            highlight: Default::default(),
        });
    }

    Some((signature, workspace))
}
