use homotopy_core::common::{Boundary, Direction, Generator, Height, RegularHeight, SliceIndex};
use homotopy_core::diagram::NewDiagramError;
use homotopy_core::expansion::ExpansionError;
use homotopy_core::{attach::BoundaryPath, common::DimensionError};
use homotopy_core::{Diagram, DiagramN};
use im::{HashMap, Vector};
use std::{collections::BTreeSet, ops::Deref};
use std::{
    convert::{Into, TryFrom, TryInto},
    fmt::Display,
};
use thiserror::Error;
pub mod homotopy;
use homotopy::{Contract, Expand, Homotopy};

use palette::Srgb;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::str::FromStr;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Color(pub(crate) Srgb<u8>);
impl Eq for Color {}

impl Deref for Color {
    type Target = Srgb<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (r, g, b) = self.into_components();
        write!(f, "#{:x}{:x}{:x}", r, g, b)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorInfo {
    pub name: String,
    pub color: Color,
    pub diagram: Diagram,
}

pub type Signature = HashMap<Generator, GeneratorInfo>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub diagram: Diagram,
    pub path: Vector<SliceIndex>,
    pub attach: Option<Vector<AttachOption>>,
    pub highlight: Option<AttachOption>,
}

impl Workspace {
    pub fn visible_dimension(&self) -> usize {
        self.diagram.dimension() - self.path.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedBoundary {
    pub boundary: Boundary,
    pub diagram: Diagram,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Proof {
    pub(super) signature: Signature,
    pub(super) workspace: Option<Workspace>,
    boundary: Option<SelectedBoundary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Create a new generator of dimension zero.
    CreateGeneratorZero,

    /// Remove a generator from the signature. All generators in the signature that depend on the
    /// generator that is to be removed will be removed as well recursively. If the workspace
    /// diagram or boundaries are set and depend on any removed generator they will be cleared.
    RemoveGenerator(Generator),

    /// Set the current diagram in the workspace as a boundary. If the opposite boundary is
    /// already currently set, a new generator will be created with those boundaries if possible,
    /// after which the selected boundary is cleared. Does nothing if the workspace is empty.
    SetBoundary(Boundary),

    /// Boost the dimension of the current diagram in the workspace by taking the identity.  Does
    /// nothing if the workspace is empty.
    TakeIdentityDiagram,

    /// Clear the workspace by forgetting the current diagram.
    ClearWorkspace,

    /// Clear the currently selected boundary.
    ClearBoundary,

    /// Select a generator from signature. If there is no diagram in the workspace at the moment,
    /// load the generator's diagram into the workspace; else do nothing.
    SelectGenerator(Generator),

    /// Ascend by a number of slices in the currently selected diagram in the workspace. If there
    /// is no diagram in the workspace or it is already displayed in its original dimension,
    /// nothing happens.
    AscendSlice(usize),

    /// Descend by one slice in the currently selected diagram in the workspace. If there is no
    /// diagram in the workspace, nothing happens. If the slice does not exist, an error will be
    /// shown.
    DescendSlice(SliceIndex),

    SelectPoints(Vec<Vec<SliceIndex>>),

    Attach(AttachOption),

    HighlightAttachment(Option<AttachOption>),

    Homotopy(Homotopy),

    Imported,
}

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("the boundaries are not compatible")]
    IncompatibleBoundaries(#[from] NewDiagramError),
    #[error("selected a generator that is not in the signature")]
    UnknownGeneratorSelected,
    #[error("tried to descend into an invalid diagram slice")]
    InvalidSlice(#[from] DimensionError),
    #[error("error while performing expansion")]
    ExpansionError(#[from] ExpansionError),
    #[error("error while performing contraction")]
    ContractionError,
}

impl Proof {
    /// Update the state in response to an [Action].
    pub fn update(&mut self, action: &Action) -> Result<(), ModelError> {
        match action {
            Action::CreateGeneratorZero => {
                self.create_generator_zero();
                Ok(())
            }
            Action::RemoveGenerator(_) => unimplemented!(),
            Action::SetBoundary(boundary) => self.set_boundary(*boundary),
            Action::TakeIdentityDiagram => {
                self.take_identity_diagram();
                Ok(())
            }
            Action::ClearWorkspace => {
                self.clear_workspace();
                Ok(())
            }
            Action::ClearBoundary => {
                self.clear_boundary();
                Ok(())
            }
            Action::SelectGenerator(generator) => self.select_generator(*generator),
            Action::AscendSlice(count) => {
                self.ascend_slice(*count);
                Ok(())
            }
            Action::DescendSlice(slice) => self.descend_slice(*slice),
            Action::SelectPoints(points) => {
                self.select_points(points);
                Ok(())
            }
            Action::Attach(option) => {
                self.attach(&option);
                Ok(())
            }
            Action::HighlightAttachment(option) => {
                self.highlight_attachment(option.clone());
                Ok(())
            }
            Action::Homotopy(Homotopy::Expand(homotopy)) => self.homotopy_expansion(homotopy),
            Action::Homotopy(Homotopy::Contract(homotopy)) => self.homotopy_contraction(homotopy),
            Action::Imported => Ok(()),
        }
    }

    /// Handler for [Action::CreateGeneratorZero].
    fn create_generator_zero(&mut self) {
        let id = self.create_generator_id();
        let generator = Generator::new(id, 0);

        let info = GeneratorInfo {
            name: format!("Cell {}", id),
            color: Color(Srgb::<u8>::from_str(COLORS[id % COLORS.len()]).unwrap()),
            diagram: generator.into(),
        };

        self.signature.insert(generator, info);
    }

    fn create_generator(
        &mut self,
        source: Diagram,
        target: Diagram,
    ) -> Result<(), NewDiagramError> {
        let id = self.create_generator_id();
        let generator = Generator::new(id, source.dimension() + 1);
        let diagram = DiagramN::new(generator, source, target)?;

        let info = GeneratorInfo {
            name: format!("Cell {}", id),
            color: Color(Srgb::<u8>::from_str(COLORS[id % COLORS.len()]).unwrap()),
            diagram: diagram.into(),
        };

        self.signature.insert(generator, info);

        Ok(())
    }

    fn create_generator_id(&self) -> usize {
        self.signature
            .iter()
            .map(|(generator, _)| generator.id)
            .max()
            .map_or(0, |id| id + 1)
    }

    /// Handler for [Action::SetBoundary].
    fn set_boundary(&mut self, boundary: Boundary) -> Result<(), ModelError> {
        use Boundary::{Source, Target};

        match (&self.workspace, &self.boundary) {
            (Some(workspace), Some(selected)) => {
                if boundary == selected.boundary {
                    self.boundary = Some(SelectedBoundary {
                        boundary,
                        diagram: workspace.diagram.clone(),
                    });
                    self.workspace = None;
                } else {
                    let (source, target) = match boundary {
                        Source => (workspace.diagram.clone(), selected.diagram.clone()),
                        Target => (selected.diagram.clone(), workspace.diagram.clone()),
                    };

                    self.create_generator(source, target)
                        .map_err(ModelError::IncompatibleBoundaries)?;

                    self.boundary = None;
                    self.workspace = None;
                }
            }
            (Some(workspace), None) => {
                self.boundary = Some(SelectedBoundary {
                    boundary,
                    diagram: workspace.diagram.clone(),
                });
                self.workspace = None;
            }
            _ => {}
        };

        Ok(())
    }

    /// Handler for [Action::TakeIdentityDiagram].
    fn take_identity_diagram(&mut self) {
        match &mut self.workspace {
            Some(workspace) => {
                workspace.diagram = workspace.diagram.identity().into();

                // TODO: Figure out what to do with the path in all cases
                if workspace.diagram.dimension() >= 2 {
                    workspace.path.push_back(Boundary::Target.into());
                }
            }
            None => {}
        }
    }

    /// Handler for [Action::ClearWorkspace].
    fn clear_workspace(&mut self) {
        self.workspace = None;
    }

    /// Handler for [Action::ClearBoundary].
    fn clear_boundary(&mut self) {
        self.boundary = None;
    }

    /// Handler for [Action::SelectGenerator].
    fn select_generator(&mut self, generator: Generator) -> Result<(), ModelError> {
        if self.workspace.is_some() {
            return Ok(());
        }

        let info = self
            .signature
            .get(&generator)
            .ok_or(ModelError::UnknownGeneratorSelected)?;

        self.workspace = Some(Workspace {
            diagram: info.diagram.clone(),
            path: Default::default(),
            attach: Default::default(),
            highlight: Default::default(),
        });

        Ok(())
    }

    /// Handler for [Action::AscendSlice].
    fn ascend_slice(&mut self, mut count: usize) {
        if let Some(workspace) = &mut self.workspace {
            while count > 0 && !workspace.path.is_empty() {
                workspace.path.pop_back();
                count -= 1;
            }

            workspace.attach = None;
            workspace.highlight = None;
        }
    }

    /// Handler for [Action::DescendSlice].
    fn descend_slice(&mut self, slice: SliceIndex) -> Result<(), ModelError> {
        if let Some(workspace) = &mut self.workspace {
            let mut path = workspace.path.clone();
            path.push_back(slice);

            // Check if path is valid
            let mut part = workspace.diagram.clone();
            for height in &path {
                part = DiagramN::try_from(part)
                    .map_err(ModelError::InvalidSlice)?
                    .slice(*height)
                    .ok_or(ModelError::InvalidSlice(DimensionError))?;
            }

            // Update workspace
            workspace.path = path;
            workspace.attach = None;
            workspace.highlight = None;
        }

        Ok(())
    }

    /// Handler for [Action::SelectPoint].
    fn select_points(&mut self, selected: &[Vec<SliceIndex>]) {
        if selected.is_empty() {
            return;
        }

        let workspace = match &self.workspace {
            Some(workspace) => workspace,
            None => return,
        };

        let mut matches: BTreeSet<AttachOption> = BTreeSet::new();

        for point in selected {
            let (boundary_path, point) = BoundaryPath::split(&point);

            let haystack = match &boundary_path {
                None => workspace.diagram.clone(),
                Some(boundary_path) => DiagramN::try_from(workspace.diagram.clone())
                    .ok()
                    .and_then(|diagram| boundary_path.follow(&diagram))
                    .unwrap(),
            };

            let boundary: Boundary = boundary_path
                .clone()
                .map_or(Boundary::Target, |bp| bp.boundary());

            for (generator, info) in self.signature.iter() {
                if info.diagram.dimension() == haystack.dimension() + 1 {
                    let needle = DiagramN::try_from(info.diagram.clone())
                        .unwrap()
                        .slice(boundary.flip())
                        .unwrap();

                    matches.extend(
                        haystack
                            .embeddings(&needle)
                            .filter(|embedding| contains_point(needle.clone(), &point, embedding))
                            .map(|embedding| AttachOption {
                                embedding: embedding.into_iter().collect(),
                                boundary_path: boundary_path.clone(),
                                generator: *generator,
                            }),
                    );
                }
            }
        }

        match matches.len().cmp(&1) {
            Ordering::Less => {
                let workspace = self.workspace.as_mut().unwrap();
                workspace.attach = None;
                workspace.highlight = None;
            }
            Ordering::Equal => self.attach(&matches.into_iter().next().unwrap()),
            Ordering::Greater => {
                let workspace = self.workspace.as_mut().unwrap();
                workspace.attach = Some(matches.into_iter().collect());
                workspace.highlight = None;
            }
        }
    }

    fn attach(&mut self, option: &AttachOption) {
        if let Some(workspace) = &mut self.workspace {
            // TODO: Better error handling, although none of these errors should occur
            let diagram: DiagramN = workspace.diagram.clone().try_into().unwrap();
            let generator: DiagramN = self
                .signature
                .get(&option.generator)
                .unwrap()
                .diagram
                .clone()
                .try_into()
                .unwrap();
            let embedding: Vec<_> = option.embedding.iter().cloned().collect();

            let result = match &option.boundary_path {
                Some(bp) => diagram
                    .attach(&generator, bp.boundary(), &embedding)
                    .unwrap(),
                None => diagram
                    .identity()
                    .attach(&generator, Boundary::Target, &embedding)
                    .unwrap(),
            };

            workspace.attach = None;
            workspace.highlight = None;

            // TODO: Figure out what should happen with the slice path
            match option.boundary_path {
                Some(_) => {
                    workspace.diagram = result.into();
                }
                None => {
                    workspace.diagram = result.target();
                }
            }
        }
    }

    /// Handler for [Action::HighlightAttachment].
    fn highlight_attachment(&mut self, option: Option<AttachOption>) {
        if let Some(workspace) = &mut self.workspace {
            workspace.highlight = option;
        }
    }

    fn homotopy_expansion(&mut self, homotopy: &Expand) -> Result<(), ModelError> {
        if let Some(workspace) = &mut self.workspace {
            let diagram: DiagramN = workspace.diagram.clone().try_into().unwrap();

            let location = {
                let mut location: Vec<_> = workspace.path.iter().cloned().collect();
                location.extend(homotopy.location.clone());
                location
            };

            workspace.diagram = diagram.expand(&location, homotopy.direction)?.into();

            // TODO: Update path appropriately
        }

        Ok(())
    }

    fn homotopy_contraction(&mut self, homotopy: &Contract) -> Result<(), ModelError> {
        // TODO: Proper errors

        if let Some(workspace) = &mut self.workspace {
            let diagram: DiagramN = workspace.diagram.clone().try_into().unwrap();
            let location = {
                let mut location: Vec<_> = workspace.path.iter().cloned().collect();
                location.extend(homotopy.location.clone());
                location
            };

            let (height, bias) = match homotopy.direction {
                Direction::Forward => (homotopy.height, homotopy.bias),
                Direction::Backward => {
                    if homotopy.height == 0 {
                        // TODO: Show an error
                        panic!("Contracting off the edge of the diagram.");
                    }

                    let bias = homotopy.bias.map(homotopy_core::Bias::flip);
                    (homotopy.height - 1, bias)
                }
            };

            workspace.diagram = diagram
                .contract(&location, height, bias)
                .ok_or(ModelError::ContractionError)?
                .into();

            // TODO: Update path appropriately.
        }

        Ok(())
    }

    pub fn workspace(&self) -> Option<&Workspace> {
        self.workspace.as_ref()
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    pub fn render_style() -> RenderStyle {
        RenderStyle::default()
    }
}

#[derive(Clone, PartialEq, Copy, Debug)]
pub struct RenderStyle {
    pub scale: f32,
    pub wire_thickness: f32,
    pub point_radius: f32,
}

impl Default for RenderStyle {
    fn default() -> Self {
        Self {
            scale: 40.0,
            wire_thickness: 8.0,
            point_radius: 6.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AttachOption {
    pub generator: Generator,
    pub boundary_path: Option<BoundaryPath>,
    pub embedding: Vector<usize>,
}

const COLORS: &[&str] = &[
    "#2980b9", // belize blue
    "#c0392b", // pomegranate
    "#f39c12", // orange
    "#8e44ad", // wisteria
    "#27ae60", // nephritis
    "#f1c40f", // sunflower
    "#ffffff", // white
    "#000000", //black
];

fn contains_point(diagram: Diagram, point: &[Height], embedding: &[RegularHeight]) -> bool {
    use Diagram::{Diagram0, DiagramN};

    match (point.split_first(), diagram) {
        (None, _) => true,
        (Some(_), Diagram0(_)) => false,
        (Some((height, point)), DiagramN(diagram)) => {
            let (shift, embedding) = embedding.split_first().unwrap_or((&0, &[]));
            let shift = Height::Regular(*shift);

            if height.to_int() < shift.to_int() {
                return false;
            }

            let height = Height::from_int(height.to_int() - shift.to_int());

            match diagram.slice(height) {
                Some(slice) => contains_point(slice, point, embedding),
                None => false,
            }
        }
    }
}
