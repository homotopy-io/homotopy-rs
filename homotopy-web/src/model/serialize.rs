use super::{AttachOption, Color, GeneratorInfo, Signature, Workspace};
use homotopy_core::common::{Generator, SliceIndex};
use homotopy_core::serialize::{Key, Keyed, Serialization};
use homotopy_core::Diagram;
use im::{HashMap, Vector};
use wasm_bindgen::JsCast;

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
    a.set_attribute("download", &name)?;
    body.append_child(&a)?;
    a.click();
    a.remove();
    web_sys::Url::revoke_object_url(&url)
}

#[derive(Debug, PartialEq, Eq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct Data {
    signature: Serialization,
    generator_info: HashMap<Generator, (String, Color)>,
    workspace: Option<WorkspaceSer>,
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
struct WorkspaceSer {
    diagram: Key<Diagram>,
    path: Vector<SliceIndex>,
    attach: Option<Vector<AttachOption>>,
    highlight: Option<AttachOption>,
}

impl From<Signature> for Data {
    fn from(sig: Signature) -> Self {
        let mut stripped: std::collections::HashMap<Generator, Diagram> = Default::default();
        let mut generator_info: HashMap<Generator, (String, Color)> = Default::default();
        for (k, v) in sig {
            stripped.insert(k, v.diagram);
            generator_info.insert(k, (v.name, v.color));
        }
        Self {
            signature: Serialization::from(stripped),
            generator_info,
            workspace: None,
        }
    }
}

impl From<(Signature, Workspace)> for Data {
    fn from((sig, ws): (Signature, Workspace)) -> Self {
        let mut stripped: std::collections::HashMap<Generator, Diagram> = Default::default();
        let mut generator_info: HashMap<Generator, (String, Color)> = Default::default();
        let k = ws.diagram.key();
        for (k, v) in sig {
            stripped.insert(k, v.diagram);
            generator_info.insert(k, (v.name, v.color));
        }
        Self {
            signature: Serialization::from((stripped, ws.diagram)),
            generator_info,
            workspace: Some(WorkspaceSer {
                diagram: k,
                path: ws.path,
                attach: ws.attach,
                highlight: ws.highlight,
            }),
        }
    }
}

impl From<Data> for (Signature, Option<Workspace>) {
    fn from(data: Data) -> Self {
        let signature = &data.signature;
        let workspace: Option<Workspace> = data.workspace.map(|ws| Workspace {
            diagram: signature.diagram(&ws.diagram),
            path: ws.path,
            attach: ws.attach,
            highlight: ws.highlight,
        });
        let sig: std::collections::HashMap<Generator, Diagram> = data.signature.into();
        let gi = data.generator_info;
        (
            sig.into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        GeneratorInfo {
                            name: gi[&k].0.clone(),
                            color: gi[&k].1.clone(),
                            diagram: v,
                        },
                    )
                })
                .collect(),
            workspace,
        )
    }
}

impl From<Data> for Vec<u8> {
    fn from(data: Data) -> Self {
        rmp_serde::to_vec(&data).unwrap()
    }
}

impl From<Vec<u8>> for Data {
    fn from(bs: Vec<u8>) -> Self {
        rmp_serde::from_read_ref(&bs).unwrap()
    }
}
