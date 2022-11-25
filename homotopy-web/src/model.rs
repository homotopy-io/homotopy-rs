pub use history::Proof;
use history::{History, UndoState};
use homotopy_common::hash::FastHashSet;
use homotopy_core::{
    common::{BoundaryPath, RegularHeight},
    Boundary, Diagram, DiagramN, Height, SliceIndex,
};
use homotopy_graphics::{manim, stl, svg, tikz};
use homotopy_model::proof::AttachOption;
pub use homotopy_model::{history, migration, proof, serialize};
use im::Vector;
use js_sys::JsString;
use serde::Serialize;
use thiserror::Error;
use wasm_bindgen::{prelude::*, JsCast};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Action {
    Proof(proof::Action),
    History(history::Action),
    ImportActions(proof::SerializedData),
    ExportProof,
    ExportActions,
    ExportTikz(bool),
    ExportSvg,
    ExportManim(bool),
    ExportStl,
    Select(usize),

    ClearAttach,
    SelectPoints(Vec<Vec<SliceIndex>>),
    HighlightAttachment(Option<AttachOption>),
    HighlightSlice(Option<SliceIndex>),
}

impl Action {
    /// Determines if a given [Action] is valid given the current [Proof].
    pub fn is_valid(&self, proof: &Proof) -> bool {
        use homotopy_core::Direction::{Backward, Forward};

        match self {
            Self::Proof(action) => proof.is_valid(action),
            Self::ExportTikz(_) | Self::ExportSvg | Self::ExportManim(_) => proof
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

#[derive(Debug, Clone, Default)]
pub struct State {
    pub history: History,
    pub attach: Option<Vector<AttachOption>>,
    pub attachment_highlight: Option<AttachOption>,
    pub slice_highlight: Option<SliceIndex>,
}

impl State {
    #[inline]
    pub fn proof(&self) -> &Proof {
        self.history.proof()
    }

    /// Update the state in response to an [Action].
    pub fn update(&mut self, action: Action) -> Result<(), ModelError> {
        match action {
            Action::Proof(action) => {
                // If we are importing a proof,
                // might as well forget about all previous actions.
                // This avoid OOM with multiple imports of big proofs.
                if matches!(action, proof::Action::ImportProof(_)) {
                    drop_actions();
                }
                // Only exfiltrate proof actions, otherwise
                // we risk funny business with circular action imports.
                let data = serde_json::to_string(&action).expect("Failed to serialize action.");
                push_action(JsString::from(data));

                let mut proof = self.proof().clone();
                proof.update(&action).map_err(ModelError::from)?;
                self.history.add(action, proof);
                self.clear_attach();
            }

            Action::History(history::Action::Move(dir)) => {
                use homotopy_core::Direction::{Backward, Forward};
                match dir {
                    history::Direction::Linear(Forward) => {
                        self.history.redo()?;
                        if let Some(action) = self.history.last_action() {
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
                self.clear_attach();
            }

            Action::ExportTikz(with_braid) => {
                let signature = &self.proof().signature;
                let diagram = self.proof().workspace.as_ref().unwrap().visible_diagram();
                let stylesheet = tikz::stylesheet(signature);
                let data = tikz::render(&diagram, &stylesheet, signature, with_braid).unwrap();
                generate_download("homotopy_io_export", "tikz", data.as_bytes())
                    .map_err(ModelError::Export)?;
            }

            Action::ExportSvg => {
                let signature = &self.proof().signature;

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
                    let mut inner_stylesheet = svg::stylesheet(signature);
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

            Action::ExportManim(use_opengl) => {
                let signature = &self.proof().signature;
                let diagram = self.proof().workspace.as_ref().unwrap().visible_diagram();
                let stylesheet = manim::stylesheet(signature);
                let data = manim::render(&diagram, signature, &stylesheet, use_opengl).unwrap();
                generate_download("homotopy_io_export", "py", data.as_bytes())
                    .map_err(ModelError::Export)?;
            }

            Action::ExportStl => {
                let signature = &self.proof().signature;
                let diagram = self.proof().workspace.as_ref().unwrap().visible_diagram();
                let data = stl::render(&diagram, signature).unwrap();
                generate_download("homotopy_io_export", "stl", data.as_bytes())
                    .map_err(ModelError::Export)?;
            }

            Action::ExportActions => {
                let actions = self.history.get_actions();
                let payload: (bool, Vec<proof::Action>) = (true, actions);
                let data = serde_json::to_string(&payload).map_err(|_e| ModelError::Internal)?;
                generate_download("homotopy_io_actions", "txt", data.as_bytes())
                    .map_err(ModelError::Export)?;
            }

            Action::ExportProof => {
                let data = serialize::serialize(
                    self.proof().signature.clone(),
                    self.proof().workspace.clone(),
                    self.proof().metadata.clone(),
                );
                generate_download("homotopy_io_export", "hom", data.as_slice())
                    .map_err(ModelError::Export)?;
            }

            Action::ImportActions(data) => {
                // Leave room for a future "replay on top of current workspace".
                let mut proof: Proof = Default::default();
                let (safe, actions): (bool, Vec<proof::Action>) =
                    serde_json::from_slice(&data.0)
                        .or(Err(ModelError::Proof(proof::ProofError::Import)))?;
                let len = if safe {
                    actions.len()
                } else {
                    actions.len() - 1
                };
                for a in &actions[..len] {
                    if proof.is_valid(a) {
                        proof.update(a)?;
                        self.history.add(a.clone(), proof.clone());
                    } else {
                        return Err(ModelError::Proof(proof::ProofError::InvalidAction));
                    }
                }
            }

            Action::Select(index) => {
                let action = match self.attach.as_ref() {
                    // Select a generator.
                    None => proof::Action::SelectGenerator(
                        self.proof()
                            .signature
                            .iter()
                            .nth(index)
                            .ok_or(ModelError::IndexOutOfBounds)?
                            .generator,
                    ),
                    // Select an attachment option.
                    Some(att) => proof::Action::Attach(
                        att.get(index).ok_or(ModelError::IndexOutOfBounds)?.clone(),
                    ),
                };
                self.update(Action::Proof(action))?;
            }
            Action::SelectPoints(points) => self.select_points(&points)?,
            Action::HighlightAttachment(option) => self.highlight_attachment(option),
            Action::HighlightSlice(slice) => self.highlight_slice(slice),
            Action::ClearAttach => self.clear_attach(),
        }

        Ok(())
    }

    /// Handler for [Action::SelectPoints].
    fn select_points(&mut self, selected: &[Vec<SliceIndex>]) -> Result<(), ModelError> {
        if selected.is_empty() {
            return Ok(());
        }

        let workspace = match self.proof().workspace.as_ref() {
            Some(workspace) => workspace,
            None => return Ok(()),
        };

        let mut matches: FastHashSet<AttachOption> = Default::default();

        let selected_with_path: Vec<_> = selected
            .iter()
            .map(|point| {
                let mut point_with_path: Vec<SliceIndex> = workspace.path.iter().copied().collect();
                point_with_path.extend(point.iter().copied());
                point_with_path
            })
            .collect();

        let attach_on_boundary = selected_with_path
            .iter()
            .any(|point| BoundaryPath::split(point).0.is_some());

        for point in selected_with_path {
            let (boundary_path, point) = BoundaryPath::split(&point);

            if boundary_path.is_none() && attach_on_boundary {
                continue;
            }

            let haystack = match boundary_path {
                None => workspace.diagram.clone(),
                Some(boundary_path) => DiagramN::try_from(workspace.diagram.clone())
                    .ok()
                    .and_then(|diagram| diagram.boundary(boundary_path))
                    .ok_or(ModelError::NoAttachment)?,
            };

            let boundary: Boundary = boundary_path.map_or(Boundary::Target, BoundaryPath::boundary);

            for info in self.proof().signature.iter() {
                macro_rules! extend {
                    ($diagram:expr, $tag:expr) => {
                        let needle = $diagram.slice(boundary.flip()).unwrap();
                        matches.extend(
                            haystack
                                .embeddings(&needle)
                                .filter(|embedding| contains_point(&needle, &point, embedding))
                                .map(|embedding| AttachOption {
                                    generator: info.generator,
                                    diagram: $diagram,
                                    tag: $tag,
                                    boundary_path,
                                    embedding: embedding.into_iter().collect(),
                                }),
                        );
                    };
                }

                match info.generator.dimension.cmp(&(haystack.dimension() + 1)) {
                    std::cmp::Ordering::Less => {
                        if cfg!(feature = "weak-units") {
                            let identity = |mut diagram: Diagram| {
                                while diagram.dimension() < haystack.dimension() + 1 {
                                    diagram = diagram.weak_identity().into();
                                }
                                DiagramN::try_from(diagram).unwrap()
                            };

                            extend!(identity(info.diagram.clone()), Some("identity".to_owned()));
                        }

                        if let Diagram::DiagramN(d) = &info.diagram {
                            if info.invertible {
                                let bubble = |mut diagram: DiagramN| {
                                    while diagram.dimension() < haystack.dimension() + 1 {
                                        diagram = diagram.bubble();
                                    }
                                    diagram
                                };

                                extend!(bubble(d.clone()), Some("bubble".to_owned()));
                                extend!(bubble(d.inverse()), Some("inverse bubble".to_owned()));
                            }
                        }
                    }
                    std::cmp::Ordering::Equal => {
                        if let Diagram::DiagramN(d) = &info.diagram {
                            extend!(d.clone(), None);
                            if info.invertible {
                                extend!(d.inverse(), Some("inverse".to_owned()));
                            }
                        }
                    }
                    std::cmp::Ordering::Greater => (),
                }
            }
        }

        match matches.len() {
            0 => {
                self.clear_attach();
                Err(ModelError::NoAttachment)
            }
            1 => {
                self.clear_attach();
                self.update(Action::Proof(proof::Action::Attach(
                    matches.into_iter().next().unwrap(),
                )))
            }
            _ => {
                self.attach = Some(matches.into_iter().collect());
                self.attachment_highlight = None;
                self.slice_highlight = None;
                Ok(())
            }
        }
    }

    /// Handler for [Action::HighlightAttachment].
    fn highlight_attachment(&mut self, option: Option<AttachOption>) {
        self.attachment_highlight = option;
    }

    /// Handler for [Action::HighlightSlice].
    fn highlight_slice(&mut self, option: Option<SliceIndex>) {
        self.slice_highlight = option;
    }

    /// Handler for [Action::ClearAttach].
    fn clear_attach(&mut self) {
        self.attach = None;
        self.attachment_highlight = None;
        self.slice_highlight = None;
    }
}

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("export failed")]
    Export(wasm_bindgen::JsValue),
    #[error(transparent)]
    Proof(#[from] proof::ProofError),
    #[error(transparent)]
    History(#[from] history::HistoryError),
    #[error("internal error")]
    Internal,
    #[error("no attachment found")]
    NoAttachment,
    #[error("index out of bounds")]
    IndexOutOfBounds,
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
    pub fn drop_actions();

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

fn contains_point(diagram: &Diagram, point: &[Height], embedding: &[RegularHeight]) -> bool {
    use Diagram::{Diagram0, DiagramN};

    match (point.split_first(), diagram) {
        (None, _) => true,
        (Some(_), Diagram0(_)) => false,
        (Some((height, point)), DiagramN(diagram)) => {
            let (shift, embedding) = embedding.split_first().unwrap_or((&0, &[]));
            let shift = Height::Regular(*shift);

            if usize::from(*height) < usize::from(shift) {
                return false;
            }

            let height = Height::from(usize::from(*height) - usize::from(shift));

            match diagram.slice(height) {
                Some(slice) => contains_point(&slice, point, embedding),
                None => false,
            }
        }
    }
}
