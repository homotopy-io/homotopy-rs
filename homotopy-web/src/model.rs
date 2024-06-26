use std::io::{Cursor, Write};

pub use history::Proof;
use history::{History, UndoState};
use homotopy_core::{
    common::{BoundaryPath, Generator},
    signature::Signature,
    Boundary, Diagram, DiagramN, Height, SliceIndex,
};
use homotopy_graphics::{manim, stl, svg, tikz};
use homotopy_model::proof::AttachOption;
pub use homotopy_model::{history, migration, proof, serialize};
use image::{DynamicImage, RgbaImage};
use serde::Serialize;
use thiserror::Error;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::app::account;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Action {
    Proof(proof::Action),
    History(history::Action),
    ImportActions(proof::SerializedData),
    ExportProof,
    ExportActions,

    ExportImage(ImageFormat, ImageOption),

    Select(usize),
    ClearSelections,
    Merge(Generator),
    SelectPoint(Vec<SliceIndex>, bool),
    HighlightAttachment(Option<AttachOption>),
    HighlightSlice(Option<SliceIndex>),

    SetRemoteProjectMetadata(Option<account::RemoteProjectMetadata>),

    Help,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize)]
pub enum ImageFormat {
    // 3d formats
    Png,
    Stl(stl::StlOptions),

    // 2d formats
    Svg,
    Tikz(tikz::TikzOptions),
    Manim(manim::ManimOptions),
}

impl ImageFormat {
    const fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Stl(_) => "stl",
            Self::Svg => "svg",
            Self::Tikz(_) => "tex",
            Self::Manim(_) => "py",
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize)]
pub enum ImageOption {
    // Export a single image
    Single,

    // Export a zip file containing multiple images
    Multiple,
}

impl Action {
    /// Determines if a given [Action] is valid given the current [Proof].
    #[must_use]
    pub fn is_valid(&self, proof: &Proof) -> bool {
        match self {
            Self::Proof(action) => action.is_valid(proof),
            Self::History(history::Action::Move(dir)) => proof.can_move(dir),
            _ => true,
        }
    }

    /// Determines if a given [Action] is experimental.
    #[must_use]
    pub const fn is_experimental(&self) -> bool {
        match self {
            Self::Proof(action) => action.is_experimental(),
            _ => false,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selectables {
    Merge(Generator, Vec<Generator>),
    Attach(Vec<AttachOption>),
}

impl Selectables {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Attach(_) => "Attach",
            Self::Merge(_, _) => "Merge",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub remote_project_metadata: Option<account::RemoteProjectMetadata>,
    pub history: History,
    pub options: Option<Selectables>,
    pub attachment_highlight: Option<AttachOption>,
    pub slice_highlight: Option<SliceIndex>,
}

impl State {
    #[inline]
    #[must_use]
    pub fn proof(&self) -> &Proof {
        self.history.proof()
    }

    /// Determines if a given [Action] should reset the panzoom state, given the current [State].
    #[must_use]
    pub fn resets_panzoom(&self, action: &Action) -> bool {
        match action {
            Action::Proof(action) => self.proof().resets_panzoom(action),
            _ => false,
        }
    }

    /// Update the state in response to an [Action].
    pub fn update(&mut self, action: Action) -> Result<bool, ModelError> {
        match action {
            Action::Proof(action) => {
                // Only exfiltrate proof actions, otherwise
                // we risk funny business with circular action imports.
                crate::panic::push_action(&action);

                if self.history.try_redo(&action).is_err() {
                    let mut proof = self.proof().clone();
                    let res = proof.update(&action);
                    if matches!(res, Err(_) | Ok(false)) {
                        crate::panic::pop_action();
                        return Ok(res?);
                    }
                    self.history.add(action, proof);
                }
                self.clear_selections();
            }

            Action::History(history::Action::Move(dir)) => {
                use homotopy_core::Direction::{Backward, Forward};
                match dir {
                    history::Direction::Linear(Forward) => {
                        self.history.redo()?;
                        if let Some(action) = self.history.last_action() {
                            crate::panic::push_action(&action);
                        }
                    }
                    history::Direction::Linear(Backward) => {
                        self.history.undo()?;
                        // The action we just undid is an ImportProof
                        // So we need to reimport the context before
                        // that into the panic handler.
                        if !crate::panic::pop_action() {
                            for a in self.history.get_last_import_segment() {
                                crate::panic::push_action(&a);
                            }
                        }
                    }
                };
                self.clear_selections();
            }

            Action::ExportImage(ImageFormat::Svg, option) => {
                assert_eq!(
                    option,
                    ImageOption::Single,
                    "Multiple SVG export is not supported.",
                );

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
                    format!("<style>{inner_stylesheet}</style>")
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

            Action::ExportImage(ImageFormat::Png, option) => {
                assert_eq!(
                    option,
                    ImageOption::Single,
                    "Multiple PNG export is not supported.",
                );

                // Get the canvas element
                let canvas = web_sys::window()
                    .expect("no window")
                    .document()
                    .expect("no document")
                    .get_element_by_id("canvas")
                    .expect("no canvas")
                    .dyn_into::<HtmlCanvasElement>()
                    .unwrap();

                // Get the WebGL context
                let gl: WebGl2RenderingContext = canvas
                    .get_context("webgl2")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<WebGl2RenderingContext>()
                    .unwrap();

                // Get the dimensions of the canvas
                let width = canvas.width() as i32;
                let height = canvas.height() as i32;

                // Read the pixels from the WebGL context
                let mut pixels = vec![0; (width * height * 4) as usize];
                gl.read_pixels_with_opt_u8_array(
                    0,
                    0,
                    width,
                    height,
                    WebGl2RenderingContext::RGBA,
                    WebGl2RenderingContext::UNSIGNED_BYTE,
                    Some(&mut pixels),
                )
                .unwrap();

                // Create an image from the raw pixels and encode it to PNG
                let mut data = Vec::new();
                DynamicImage::from(
                    RgbaImage::from_raw(width as u32, height as u32, pixels).unwrap(),
                )
                .flipv()
                .write_to(&mut Cursor::new(&mut data), image::ImageFormat::Png)
                .unwrap();

                generate_download("homotopy_io_export", "png", &data)
                    .map_err(ModelError::Export)?;
            }

            Action::ExportImage(format, ImageOption::Single) => {
                let signature = &self.proof().signature;
                let Some(ws) = self.proof().workspace.as_ref() else {
                    return Ok(false);
                };

                let diagram = ws.visible_diagram();
                let view_dimension = ws.view.dimension();

                let data = render(&diagram, view_dimension, signature, format);

                generate_download("homotopy_io_export", format.extension(), data.as_bytes())
                    .map_err(ModelError::Export)?;
            }

            Action::ExportImage(format, ImageOption::Multiple) => {
                let signature = &self.proof().signature;
                let Some(ws) = self.proof().workspace.as_ref() else {
                    return Ok(false);
                };

                let Diagram::DiagramN(diagram) = ws.visible_diagram() else {
                    return Ok(false);
                };
                let view_dimension = ws.view.dimension().min(diagram.dimension() as u8 - 1);

                let extension = format.extension();
                let data = zip_files(diagram.slices().enumerate().map(|(i, slice)| {
                    let name = match Height::from(i) {
                        Height::Regular(i) => format!("regular{i}"),
                        Height::Singular(i) => format!("singular{i}"),
                    };
                    (
                        format!("{name}.{extension}"),
                        render(&slice, view_dimension, signature, format),
                    )
                }));

                generate_download("homotopy_io_export", "zip", &data)
                    .map_err(ModelError::Export)?;
            }

            Action::ExportActions => {
                crate::panic::export_dump(true)?;
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
                let (safe, actions): (bool, Vec<proof::Action>) =
                    serde_json::from_slice(&data.0)
                        .or(Err(ModelError::Proof(proof::ProofError::Import)))?;
                let len = if safe {
                    actions.len()
                } else {
                    actions.len() - 1
                };

                // Replay actions in top of workspace
                let mut proof = self.proof().clone();
                for a in &actions[..len] {
                    if proof.update(a)? {
                        self.history.add(a.clone(), proof.clone());
                    }
                }
            }

            Action::Select(index) => {
                let action = match self.options.as_ref() {
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
                    Some(Selectables::Attach(att)) => proof::Action::Attach(
                        att.get(index).ok_or(ModelError::IndexOutOfBounds)?.clone(),
                    ),
                    Some(Selectables::Merge(from, tos)) => proof::Action::Merge(
                        *from,
                        *tos.get(index).ok_or(ModelError::IndexOutOfBounds)?,
                    ),
                };
                self.update(Action::Proof(action))?;
            }
            Action::SelectPoint(point, weak_units) => self.select_point(&point, weak_units)?,
            Action::HighlightAttachment(option) => self.highlight_attachment(option),
            Action::HighlightSlice(slice) => self.highlight_slice(slice),
            Action::ClearSelections => self.clear_selections(),
            Action::Merge(generator) => self.merge_options(generator),
            Action::SetRemoteProjectMetadata(metadata) => {
                self.set_remote_project_metadata(metadata);
            }
            Action::Help => help()?,
        }

        Ok(true)
    }

    /// Handler for [Action::SelectPoint].
    fn select_point(&mut self, point: &[SliceIndex], weak_units: bool) -> Result<(), ModelError> {
        let Some(workspace) = self.proof().workspace.as_ref() else {
            return Ok(());
        };

        let mut matches: Vec<AttachOption> = Default::default();

        let point = {
            let mut point_with_path: Vec<SliceIndex> = workspace.path.iter().copied().collect();
            point_with_path.extend(point);
            point_with_path
        };

        let (boundary_path, point) = BoundaryPath::split(&point);

        let haystack = match boundary_path {
            None => workspace.diagram.clone(),
            Some(boundary_path) => DiagramN::try_from(workspace.diagram.clone())
                .ok()
                .and_then(|diagram| diagram.boundary(boundary_path))
                .ok_or(ModelError::NoAttachment)?,
        };

        let boundary = boundary_path.map_or(Boundary::Target, BoundaryPath::boundary);

        for info in self.proof().signature.iter() {
            macro_rules! extend {
                ($diagram:expr, $tag:expr) => {
                    let needle = $diagram.slice(boundary.flip()).unwrap();
                    matches.extend(
                        haystack
                            .embeddings(&needle)
                            .filter(|embedding| needle.contains_point(&point, embedding))
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
            let bubble = |mut diagram: DiagramN| {
                while diagram.dimension() < haystack.dimension() + 1 {
                    diagram = diagram.bubble().unwrap();
                }
                diagram
            };
            let weak_unit = |mut diagram: Diagram| {
                while diagram.dimension() < haystack.dimension() + 1 {
                    diagram = diagram.weak_identity().into();
                }
                DiagramN::try_from(diagram).unwrap()
            };

            match info.generator.dimension.cmp(&(haystack.dimension() + 1)) {
                std::cmp::Ordering::Less => {
                    if weak_units {
                        extend!(weak_unit(info.diagram.clone()), Some("identity".to_owned()));
                    }

                    if let Diagram::DiagramN(d) = &info.diagram {
                        if info.invertibility.is_invertible() {
                            if weak_units {
                                extend!(
                                    weak_unit(d.inverse().into()),
                                    Some("inverse identity".to_owned())
                                );
                            }

                            extend!(bubble(d.clone()), Some("bubble".to_owned()));
                            extend!(bubble(d.inverse()), Some("inverse bubble".to_owned()));
                        }
                    }
                }
                std::cmp::Ordering::Equal => {
                    if let Diagram::DiagramN(d) = &info.diagram {
                        extend!(d.clone(), None);
                        if info.invertibility.is_invertible() {
                            extend!(d.inverse(), Some("inverse".to_owned()));
                        }
                    }
                }
                std::cmp::Ordering::Greater => (),
            }
        }

        match matches.len() {
            0 => {
                self.clear_selections();
                return Err(ModelError::NoAttachment);
            }
            1 => {
                self.clear_selections();
                self.update(Action::Proof(proof::Action::Attach(
                    matches.into_iter().next().unwrap(),
                )))?;
            }
            _ => {
                self.options = Some(Selectables::Attach(matches));
                self.attachment_highlight = None;
                self.slice_highlight = None;
            }
        }

        Ok(())
    }

    /// Handler for [Action::HighlightAttachment].
    fn highlight_attachment(&mut self, option: Option<AttachOption>) {
        self.attachment_highlight = option;
    }

    /// Handler for [Action::HighlightSlice].
    fn highlight_slice(&mut self, option: Option<SliceIndex>) {
        self.slice_highlight = option;
    }

    /// Handler for [Action::ClearSelections].
    fn clear_selections(&mut self) {
        self.options = None;
        self.attachment_highlight = None;
        self.slice_highlight = None;
    }

    /// Handler for [Action::MergeOptions].
    fn merge_options(&mut self, generator: Generator) {
        let result = self.proof().signature.globular_pairs(generator);
        if !result.is_empty() {
            self.options = Some(Selectables::Merge(generator, result));
        }
    }

    /// Handler for [Action::SetRemoteProjectId].
    fn set_remote_project_metadata(&mut self, metadata: Option<account::RemoteProjectMetadata>) {
        if let Some(md) = &metadata {
            let path = if md.visibility == account::ProjectVisibility::Published {
                format!("/p/{}", md.id)
            } else {
                format!("/u/{}/{}", md.uid, md.id)
            };
            update_window_url_path(&path);
        } else {
            update_window_url_path("/");
        }
        self.remote_project_metadata = metadata;
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
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error("internal error")]
    Internal,
    #[error("no attachment found")]
    NoAttachment,
    #[error("index out of bounds")]
    IndexOutOfBounds,
}

fn help() -> Result<(), ModelError> {
    let window = web_sys::window().ok_or(ModelError::Internal)?;
    let document = window.document().ok_or(ModelError::Internal)?;
    let location = document.location().ok_or(ModelError::Internal)?;
    location.set_href("#help").or(Err(ModelError::Internal))
}

fn render(
    diagram: &Diagram,
    dimension: u8,
    signature: &homotopy_model::proof::Signature,
    format: ImageFormat,
) -> String {
    match format {
        ImageFormat::Stl(options) => stl::render(diagram, signature, options).unwrap(),
        ImageFormat::Tikz(options) => tikz::render(diagram, dimension, signature, options).unwrap(),
        ImageFormat::Manim(options) => {
            let stylesheet = manim::stylesheet(signature);
            manim::render(diagram, dimension, signature, &stylesheet, options).unwrap()
        }
        _ => unreachable!(),
    }
}

fn zip_files(files: impl Iterator<Item = (String, String)>) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let file_options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::STORE);
        for (name, data) in files {
            zip.start_file(name, file_options).unwrap();
            zip.write_all(data.as_bytes()).unwrap();
        }
    }
    buf
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

pub fn update_window_url_path(new_path: &str) {
    let window = web_sys::window().unwrap();
    let origin = window.location().origin().unwrap();
    window
        .history()
        .unwrap()
        .replace_state_with_url(
            &None::<u8>.into(),
            "title",
            Some(&format!("{origin}{new_path}")),
        )
        .unwrap();
}
