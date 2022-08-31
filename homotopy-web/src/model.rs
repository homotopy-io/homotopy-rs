pub use history::Proof;
use history::{History, UndoState};
use homotopy_core::common::Mode;
use homotopy_graphics::{manim, stl, svg, tikz};
pub use homotopy_model::{history, migration, proof, serialize};
use js_sys::JsString;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen::{prelude::*, JsCast};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Proof(proof::Action),
    History(history::Action),
    ImportProof(SerializedData),
    ExportProof,
    ExportActions,
    ExportTikz,
    ExportSvg,
    ExportManim,
    ExportStl,
}

impl Action {
    /// Determines if a given [Action] is valid given the current [Proof].
    pub fn is_valid(&self, proof: &Proof) -> bool {
        use homotopy_core::Direction::{Backward, Forward};

        match self {
            Self::Proof(action) => proof.is_valid(action),
            Self::ExportTikz | Self::ExportSvg | Self::ExportManim => proof
                .workspace
                .as_ref()
                .map_or(false, |ws| ws.view.dimension() == 2),
            Self::ExportStl => proof
                .workspace
                .as_ref()
                .map_or(false, |ws| ws.view.dimension() == 3),
            Self::History(history::Action::Move(dir)) => match dir {
                history::Direction::Linear(Forward) => proof.can_redo(),
                history::Direction::Linear(Backward) => proof.can_undo(),
            },
            _ => true,
        }
    }
}

impl From<proof::Action> for Action {
    fn from(action: proof::Action) -> Self {
        Self::Proof(action)
    }
}

impl From<history::Action> for Action {
    fn from(action: history::Action) -> Self {
        Self::History(action)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedData(Vec<u8>);

impl std::fmt::Debug for SerializedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SerializedData").finish()
    }
}

impl From<Vec<u8>> for SerializedData {
    fn from(data: Vec<u8>) -> Self {
        Self(data)
    }
}

impl From<SerializedData> for Vec<u8> {
    fn from(data: SerializedData) -> Self {
        data.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub history: History,
}

impl State {
    #[inline]
    pub(super) fn with_proof<F, U>(&self, f: F) -> Option<U>
    where
        F: Fn(&Proof) -> U,
    {
        self.history.with_proof(f)
    }

    /// Update the state in response to an [Action].
    pub fn update(&mut self, action: Action) -> Result<(), ModelError> {
        match action {
            Action::Proof(action) => {
                // Only exfiltrate proof actions, otherwise
                // we risk funny business with circular action imports.
                let data = serde_json::to_string(&action).expect("Failed to serialize action.");
                push_action(JsString::from(data));

                let mut proof = self.with_proof(Clone::clone).ok_or(ModelError::Internal)?;
                proof.update(&action).map_err(ModelError::from)?;
                self.history.add(action, proof);
            }

            Action::History(history::Action::Move(dir)) => {
                use homotopy_core::Direction::{Backward, Forward};
                match dir {
                    history::Direction::Linear(Forward) => {
                        self.history.redo()?;
                        // Nuclear way to get the action we just re-did
                        // but logs must be accurate!
                        if let Some(action) = self.history.get_actions().last() {
                            let data = serde_json::to_string(&action)
                                .expect("Failed to serialize action.");
                            push_action(JsString::from(data));
                        }
                    }
                    history::Direction::Linear(Backward) => {
                        self.history.undo()?;
                        pop_action();
                    }
                };
            }

            Action::ExportTikz => {
                let signature = self
                    .with_proof(|p| p.signature.clone())
                    .ok_or(ModelError::Internal)?;
                let diagram = self
                    .with_proof(|p| p.workspace.as_ref().unwrap().visible_diagram())
                    .ok_or(ModelError::Internal)?;
                let stylesheet = tikz::stylesheet(&signature);
                let data = tikz::render(&diagram, &stylesheet, &signature).unwrap();
                generate_download("homotopy_io_export", "tikz", data.as_bytes())
                    .map_err(ModelError::Export)?;
            }

            Action::ExportSvg => {
                let signature = self
                    .with_proof(|p| p.signature.clone())
                    .ok_or(ModelError::Internal)?;

                // First we locate the element containing the SVG rendered the SVG rendering
                // pipeline. We *could* do this by using a lookup by class name, which would not
                // require any chances to components/panzoom.rs, but get_elements_by_class_name
                // has a return type that requires a feature of web-sys that is not
                // currently activated. Thus this solution avoids increases in build times.
                let svg_element = web_sys::window()
                    .expect("no window")
                    .document()
                    .expect("no document")
                    .get_element_by_id("panzoom__inner__0")
                    .expect("no SVG in document");
                let svg = svg_element.inner_html();

                // We must now pull all the relevant stylesheets that are needed in the SVG.
                // Failure to do so gives a fully-black SVG. We can generate the stylesheet in the
                // same way the `SignatureStylesheet` struct does.
                // We also strip the styles of whitespace since it is unneeded.
                let stylesheet = {
                    let mut inner_stylesheet = svg::stylesheet(&signature);
                    inner_stylesheet.retain(|c| !c.is_whitespace());
                    format!("<style>{}</style>", inner_stylesheet)
                };

                // So we now have the SVG and its stylesheet in separate strings.
                // It is not enough to just concatenate them, the stylesheets need to be inside the
                // root element. Since we know that this will have form
                // <svg.*><.*
                // We just look for the first >< and use some very light indexing maths to insert
                // the stylesheet in the appropriate place.
                // Again, the function would be half as long and twice as clean if the SVG and the SVG's
                // stylesheet were in the same place in the DOM.
                let content_start = svg.find("><").unwrap() + 1;
                let mut data = String::new();
                data.push_str(&svg[..content_start]);
                data.push_str(&stylesheet);
                data.push_str(&svg[content_start..]);

                generate_download("homotopy_io_export", "svg", data.as_bytes())
                    .map_err(ModelError::Export)?;
            }

            Action::ExportManim => {
                let signature = self
                    .with_proof(|p| p.signature.clone())
                    .ok_or(ModelError::Internal)?;
                let diagram = self
                    .with_proof(|p| p.workspace.as_ref().unwrap().visible_diagram())
                    .ok_or(ModelError::Internal)?;
                let stylesheet = manim::stylesheet(&signature);
                let use_opengl = false;
                let data = manim::render(&diagram, &signature, &stylesheet, use_opengl).unwrap();
                generate_download("homotopy_io_export", "py", data.as_bytes())
                    .map_err(ModelError::Export)?;
            }

            Action::ExportStl => {
                let signature = self
                    .with_proof(|p| p.signature.clone())
                    .ok_or(ModelError::Internal)?;
                let diagram = self
                    .with_proof(|p| p.workspace.as_ref().unwrap().visible_diagram())
                    .ok_or(ModelError::Internal)?;
                let data = stl::render(&diagram, &signature).unwrap();
                generate_download("homotopy_io_export", "stl", data.as_bytes())
                    .map_err(ModelError::Export)?;
            }

            Action::ExportActions => {
                let actions = self.history.get_actions();
                let data = serde_json::to_string(&actions).map_err(|_e| ModelError::Internal)?;
                generate_download("homotopy_io_actions", "json", data.as_bytes())
                    .map_err(ModelError::Export)?;
            }

            Action::ExportProof => {
                let data = serialize::serialize(
                    self.with_proof(|p| p.signature.clone())
                        .ok_or(ModelError::Internal)?,
                    self.with_proof(|p| p.workspace.clone())
                        .ok_or(ModelError::Internal)?,
                    self.with_proof(|p| p.metadata.clone())
                        .ok_or(ModelError::Internal)?,
                );
                generate_download("homotopy_io_export", "hom", data.as_slice())
                    .map_err(ModelError::Export)?;
            }

            Action::ImportProof(data) => {
                if let Some(((signature, workspace), metadata)) =
                    serialize::deserialize(&data.0).or_else(|| migration::deserialize(&data.0))
                {
                    for g in signature.iter() {
                        g.diagram
                            .check(Mode::Deep)
                            .map_err(|_err| ModelError::Import)?;
                    }
                    if let Some(w) = workspace.as_ref() {
                        w.diagram
                            .check(Mode::Deep)
                            .map_err(|_err| ModelError::Import)?;
                    }
                    let mut proof: Proof = Default::default();
                    proof.signature = signature;
                    proof.workspace = workspace;
                    proof.metadata = metadata;
                    self.history.add(proof::Action::Imported, proof);
                } else {
                    // Leave room for a future "replay on top of current workspace".
                    let mut proof: Proof = Default::default();
                    // Act as if we imported an empty workspace
                    self.history.add(proof::Action::Imported, proof.clone());
                    let actions: Vec<proof::Action> =
                        serde_json::from_slice(&data.0).or(Err(ModelError::Import))?;
                    for a in actions {
                        if proof.is_valid(&a) {
                            proof.update(&a)?;
                            self.history.add(a, proof.clone());
                        } else {
                            Err(ModelError::Proof(proof::ModelError::InvalidAction))?;
                        }
                    }
                };
            }
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("export failed")]
    Export(wasm_bindgen::JsValue),
    #[error("import failed")]
    Import,
    #[error(transparent)]
    Proof(#[from] proof::ModelError),
    #[error(transparent)]
    History(#[from] history::HistoryError),
    #[error("internal error")]
    Internal,
}

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

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub fn push_action(a: JsString);

    #[wasm_bindgen]
    pub fn pop_action();

    #[wasm_bindgen]
    pub fn dump_actions() -> JsString;

    #[wasm_bindgen]
    pub fn download_actions();

    #[wasm_bindgen]
    pub fn display_panic_message();
}
