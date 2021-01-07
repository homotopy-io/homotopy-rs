use homotopy_core::attach::BoundaryPath;
use homotopy_core::common::*;
use homotopy_core::diagram::NewDiagramError;
use homotopy_core::expansion::ExpansionError;
use homotopy_core::{Diagram, DiagramN};
use im::{HashMap, Vector};
use std::collections::BTreeSet;
use std::convert::*;
use thiserror::Error;
pub mod homotopy;
use homotopy::*;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    ToggleDrawer(Drawer),

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
}

#[derive(Clone, PartialEq, Eq)]
pub struct State {
    signature: HashMap<Generator, GeneratorInfo>,
    workspace: Option<Workspace>,
    boundary: Option<SelectedBoundary>,
    drawer: Option<Drawer>,
}

impl Default for State {
    fn default() -> Self {
        State {
            signature: Default::default(),
            workspace: None,
            boundary: Default::default(),
            drawer: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorInfo {
    pub name: String,
    pub color: String,
    pub diagram: Diagram,
}

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

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("the boundaries are not compatible")]
    IncompatibleBoundaries(#[from] NewDiagramError),
    #[error("selected a generator that is not in the signature")]
    UnknownGeneratorSelected,
    #[error("tried to descend into an invalid diagram slice")]
    InvalidSlice,
    #[error("error while performing expansion")]
    ExpansionError(#[from] ExpansionError),
    #[error("error while performing contraction")]
    ContractionError,
}

impl State {
    /// Update the state in response to an [Action].
    pub fn update(&mut self, action: Action) -> Result<(), ModelError> {
        match action {
            Action::CreateGeneratorZero => self.create_generator_zero(),
            Action::RemoveGenerator(_) => unimplemented!(),
            Action::SetBoundary(boundary) => self.set_boundary(boundary),
            Action::TakeIdentityDiagram => self.take_identity_diagram(),
            Action::ClearWorkspace => self.clear_workspace(),
            Action::ClearBoundary => self.clear_boundary(),
            Action::SelectGenerator(generator) => self.select_generator(generator),
            Action::AscendSlice(count) => self.ascend_slice(count),
            Action::DescendSlice(slice) => self.descend_slice(slice),
            Action::SelectPoints(points) => self.select_points(points),
            Action::ToggleDrawer(drawer) => self.toggle_drawer(drawer),
            Action::Attach(option) => self.attach(option),
            Action::HighlightAttachment(option) => self.highlight_attachment(option),
            Action::Homotopy(Homotopy::Expand(homotopy)) => self.homotopy_expansion(homotopy),
            Action::Homotopy(Homotopy::Contract(homotopy)) => self.homotopy_contraction(homotopy),
        }
    }

    /// Handler for [Action::CreateGeneratorZero].
    fn create_generator_zero(&mut self) -> Result<(), ModelError> {
        let id = self.create_generator_id();
        let generator = Generator::new(id, 0);

        let info = GeneratorInfo {
            name: format!("Cell {}", id),
            color: COLORS[id % COLORS.len()].to_owned(),
            diagram: generator.into(),
        };

        self.signature.insert(generator, info);
        Ok(())
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
            color: COLORS[id % COLORS.len()].to_owned(),
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
            .map(|id| id + 1)
            .unwrap_or(0)
    }

    /// Handler for [Action::SetBoundary].
    fn set_boundary(&mut self, boundary: Boundary) -> Result<(), ModelError> {
        use Boundary::*;

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
    fn take_identity_diagram(&mut self) -> Result<(), ModelError> {
        match &mut self.workspace {
            Some(workspace) => {
                workspace.diagram = workspace.diagram.identity().into();

                // TODO: Figure out what to do with the path in all cases
                if workspace.diagram.dimension() >= 2 {
                    workspace.path.push_back(Boundary::Target.into());
                }
            }
            None => {}
        };

        Ok(())
    }

    /// Handler for [Action::ClearWorkspace].
    fn clear_workspace(&mut self) -> Result<(), ModelError> {
        self.workspace = None;
        Ok(())
    }

    /// Handler for [Action::ClearBoundary].
    fn clear_boundary(&mut self) -> Result<(), ModelError> {
        self.boundary = None;
        Ok(())
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
    fn ascend_slice(&mut self, mut count: usize) -> Result<(), ModelError> {
        if let Some(workspace) = &mut self.workspace {
            while count > 0 && !workspace.path.is_empty() {
                workspace.path.pop_back();
                count -= 1;
            }

            workspace.attach = None;
            workspace.highlight = None;
        }

        Ok(())
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
                    .map_err(|_| ModelError::InvalidSlice)?
                    .slice(*height)
                    .ok_or(ModelError::InvalidSlice)?;
            }

            // Update workspace
            workspace.path = path;
            workspace.attach = None;
            workspace.highlight = None;
        }

        Ok(())
    }

    /// Handler for [Action::SelectPoint].
    fn select_points(&mut self, selected: Vec<Vec<SliceIndex>>) -> Result<(), ModelError> {
        if selected.is_empty() {
            return Ok(());
        }

        let workspace = match &self.workspace {
            Some(workspace) => workspace,
            None => {
                return Ok(());
            }
        };

        let mut matches: BTreeSet<AttachOption> = BTreeSet::new();

        for point in selected.into_iter() {
            let (boundary_path, point) = BoundaryPath::split(&point);

            let haystack = match &boundary_path {
                None => workspace.diagram.clone(),
                Some(boundary_path) => DiagramN::try_from(workspace.diagram.clone())
                    .ok()
                    .map(|diagram| boundary_path.follow(&diagram))
                    .flatten()
                    .unwrap(),
            };

            let boundary: Boundary = boundary_path
                .clone()
                .map(|bp| bp.boundary())
                .unwrap_or(Boundary::Target);

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
                Ok(())
            }
            Ordering::Equal => self.attach(matches.into_iter().next().unwrap()),
            Ordering::Greater => {
                let workspace = self.workspace.as_mut().unwrap();
                workspace.attach = Some(matches.into_iter().collect());
                workspace.highlight = None;
                Ok(())
            }
        }
    }

    fn attach(&mut self, option: AttachOption) -> Result<(), ModelError> {
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
                    .attach(generator, bp.boundary(), &embedding)
                    .unwrap(),
                None => diagram
                    .identity()
                    .attach(generator, Boundary::Target, &embedding)
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

        Ok(())
    }

    /// Handler for [Action::ToggleDrawer].
    fn toggle_drawer(&mut self, drawer: Drawer) -> Result<(), ModelError> {
        if self.drawer == Some(drawer) {
            self.drawer = None;
        } else {
            self.drawer = Some(drawer);
        }

        Ok(())
    }

    /// Handler for [Action::HighlightAttachment].
    fn highlight_attachment(&mut self, option: Option<AttachOption>) -> Result<(), ModelError> {
        if let Some(workspace) = &mut self.workspace {
            workspace.highlight = option;
        }

        Ok(())
    }

    fn homotopy_expansion(&mut self, homotopy: Expand) -> Result<(), ModelError> {
        if let Some(workspace) = &mut self.workspace {
            let diagram: DiagramN = workspace.diagram.clone().try_into().unwrap();

            let location = {
                let mut location: Vec<_> = workspace.path.iter().cloned().collect();
                location.extend(homotopy.location);
                location
            };

            workspace.diagram = diagram.expand(&location, homotopy.direction)?.into();

            // TODO: Update path appropriately
        }

        Ok(())
    }

    fn homotopy_contraction(&mut self, homotopy: Contract) -> Result<(), ModelError> {
        // TODO: Proper errors

        if let Some(workspace) = &mut self.workspace {
            let diagram: DiagramN = workspace.diagram.clone().try_into().unwrap();
            let location = {
                let mut location: Vec<_> = workspace.path.iter().cloned().collect();
                location.extend(homotopy.location);
                location
            };

            let (height, bias) = match homotopy.direction {
                Direction::Forward => (homotopy.height, homotopy.bias),
                Direction::Backward => {
                    if homotopy.height == 0 {
                        // TODO: Show an error
                        panic!("Contracting off the edge of the diagram.");
                    }

                    let bias = homotopy.bias.map(|bias| bias.flip());
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
}

impl State {
    pub fn workspace(&self) -> Option<&Workspace> {
        self.workspace.as_ref()
    }

    pub fn signature(&self) -> &HashMap<Generator, GeneratorInfo> {
        &self.signature
    }

    pub fn render_style(&self) -> RenderStyle {
        RenderStyle::default()
    }

    pub fn drawer(&self) -> Option<Drawer> {
        self.drawer
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
        RenderStyle {
            scale: 40.0,
            wire_thickness: 8.0,
            point_radius: 6.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Drawer {
    Project,
    Signature,
    User,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    use Diagram::*;

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
