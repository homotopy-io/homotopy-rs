use std::{
    collections::BTreeSet,
    convert::{Into, TryFrom, TryInto},
};

use homotopy::Homotopy;
use homotopy_core::{
    attach::BoundaryPath,
    common::{Boundary, DimensionError, Direction, Generator, Height, RegularHeight, SliceIndex},
    contraction::ContractionError,
    diagram::NewDiagramError,
    expansion::ExpansionError,
    signature::SignatureClosure,
    typecheck::TypeError,
    Diagram, DiagramN,
};
use im::Vector;
use serde::{Deserialize, Serialize};
pub use signature::*;
use thiserror::Error;

use self::homotopy::{Contract, Expand};

mod signature;

pub mod homotopy;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    dimension: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub diagram: Diagram,
    pub path: Vector<SliceIndex>,
    pub view: View,
    pub attach: Option<Vector<AttachOption>>,
    pub attachment_highlight: Option<AttachOption>,
    pub slice_highlight: Option<SliceIndex>,
}

impl Workspace {
    pub fn visible_dimension(&self) -> usize {
        self.diagram.dimension() - self.path.len()
    }

    pub fn visible_diagram(&self) -> Diagram {
        let mut diagram = self.diagram.clone();

        for index in &self.path {
            match diagram {
                Diagram::Diagram0(_) => return diagram,
                Diagram::DiagramN(d) => diagram = d.slice(*index).unwrap(),
            }
        }

        diagram
    }
}

impl View {
    const MAX: u8 = 4;
    const MIN: u8 = 2;

    #[must_use]
    pub fn inc(self) -> Self {
        Self {
            dimension: (self.dimension + 1).clamp(Self::MIN, Self::MAX),
        }
    }

    #[must_use]
    pub fn dec(self) -> Self {
        Self {
            dimension: (self.dimension - 1).clamp(Self::MIN, Self::MAX),
        }
    }

    pub fn dimension(self) -> u8 {
        self.dimension
    }
}

impl Default for View {
    fn default() -> Self {
        Self {
            dimension: Self::MIN,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedBoundary {
    pub boundary: Boundary,
    pub diagram: Diagram,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProofState {
    pub(super) signature: Signature,
    pub(super) workspace: Option<Workspace>,
    boundary: Option<SelectedBoundary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Create a new generator of dimension zero.
    CreateGeneratorZero,

    /// Set the current diagram in the workspace as a boundary. If the opposite boundary is
    /// already currently set, a new generator will be created with those boundaries if possible,
    /// after which the selected boundary is cleared. Does nothing if the workspace is empty.
    SetBoundary(Boundary),

    /// Boost the dimension of the current diagram in the workspace by taking the identity.  Does
    /// nothing if the workspace is empty.
    TakeIdentityDiagram,

    /// Clear the attachment state.
    ClearAttach,

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

    /// Switch between adjacent slices in the currently selected diagram in the workspace.
    SwitchSlice(Direction),

    UpdateView(View),

    SelectPoints(Vec<Vec<SliceIndex>>),

    Attach(AttachOption),

    HighlightAttachment(Option<AttachOption>),

    HighlightSlice(Option<SliceIndex>),

    Homotopy(Homotopy),

    Behead,

    Befoot,

    Restrict,

    Theorem,

    Imported,

    EditSignature(SignatureEdit),

    Nothing,
}

impl Action {
    /// Determines if this [Action] is relevant with respect to undo/redo operations.
    pub fn relevant(&self) -> bool {
        !matches!(
            self,
            Action::HighlightSlice(_) | Action::HighlightAttachment(_)
        )
    }
}

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("no attachment found")]
    NoAttachment,
    #[error("the boundaries are not compatible")]
    IncompatibleBoundaries(#[from] NewDiagramError),
    #[error("selected a generator that is not in the signature")]
    UnknownGeneratorSelected,
    #[error("tried to descend into an invalid diagram slice")]
    InvalidSlice(#[from] DimensionError),
    #[error("invalid action")]
    InvalidAction,
    #[error("error while performing expansion: {0}")]
    ExpansionError(#[from] ExpansionError),
    #[error("error while performing contraction: {0}")]
    ContractionError(#[from] ContractionError),
    #[error("error while performing typechecking: {0}")]
    TypecheckingError(#[from] TypeError),
}

impl ProofState {
    /// Update the state in response to an [Action].
    pub fn update(&mut self, action: &Action) -> Result<(), ModelError> {
        match action {
            Action::CreateGeneratorZero => {
                self.signature.create_generator_zero();
                self.clear_attach();
            }
            Action::SetBoundary(boundary) => self.set_boundary(*boundary)?,
            Action::TakeIdentityDiagram => self.take_identity_diagram(),
            Action::ClearAttach => self.clear_attach(),
            Action::ClearWorkspace => self.clear_workspace(),
            Action::ClearBoundary => self.clear_boundary(),
            Action::SelectGenerator(generator) => self.select_generator(*generator)?,
            Action::AscendSlice(count) => self.ascend_slice(*count),
            Action::DescendSlice(slice) => self.descend_slice(*slice)?,
            Action::SwitchSlice(direction) => self.switch_slice(*direction),
            Action::UpdateView(view) => self.update_view(*view),
            Action::SelectPoints(points) => self.select_points(points)?,
            Action::Attach(option) => self.attach(option),
            Action::HighlightAttachment(option) => self.highlight_attachment(option.clone()),
            Action::HighlightSlice(slice) => self.highlight_slice(*slice),
            Action::Homotopy(Homotopy::Expand(homotopy)) => self.homotopy_expansion(homotopy)?,
            Action::Homotopy(Homotopy::Contract(homotopy)) => {
                self.homotopy_contraction(homotopy)?;
            }
            Action::Behead => self.behead()?,
            Action::Befoot => self.befoot()?,
            Action::Restrict => self.restrict()?,
            Action::Theorem => self.theorem()?,
            Action::EditSignature(edit) => self.edit_signature(edit),
            Action::Imported | Action::Nothing => {}
        }

        Ok(())
    }

    /// Determines if a given [Action] should reset the panzoom state, given the current  [ProofState].
    pub fn resets_panzoom(&self, action: &Action) -> bool {
        match *action {
            Action::EditSignature(SignatureEdit::Remove(node)) => {
                self.workspace.as_ref().map_or(false, |ws| {
                    self.signature.has_descendents_in(node, &ws.diagram)
                })
            }
            Action::SelectGenerator(_) => self.workspace.is_none(),
            Action::AscendSlice(i) => i > 0,
            Action::ClearWorkspace | Action::DescendSlice(_) | Action::UpdateView(_) => true,
            _ => false,
        }
    }

    /// Handler for [Action::EditSignature].
    fn edit_signature(&mut self, edit: &SignatureEdit) {
        // intercept remove events in order to clean-up workspace and boundaries
        if let SignatureEdit::Remove(node) = edit {
            // remove from the workspace
            if let Some(ws) = &self.workspace {
                if self.signature.has_descendents_in(*node, &ws.diagram) {
                    self.clear_workspace();
                }
            }
            // remove from the boundary
            if let Some(b) = &self.boundary {
                if self.signature.has_descendents_in(*node, &b.diagram) {
                    self.clear_boundary();
                }
            }
        }

        self.signature.update(edit);
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
                } else {
                    let (source, target) = match boundary {
                        Source => (workspace.diagram.clone(), selected.diagram.clone()),
                        Target => (selected.diagram.clone(), workspace.diagram.clone()),
                    };

                    self.signature
                        .create_generator(source, target)
                        .map_err(ModelError::IncompatibleBoundaries)?;

                    self.boundary = None;
                }
                self.workspace = None;
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
                if workspace.diagram.dimension() + workspace.path.len() >= 2 {
                    workspace.path.push_front(Boundary::Target.into());
                }

                workspace.diagram = workspace.diagram.identity().into();
                self.clear_attach();
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

    /// Handler for [Action::ClearAttach].
    fn clear_attach(&mut self) {
        if let Some(ref mut workspace) = self.workspace {
            workspace.attach = None;
            workspace.attachment_highlight = None;
            workspace.slice_highlight = None;
        }
    }

    /// Handler for [Action::SelectGenerator].
    fn select_generator(&mut self, generator: Generator) -> Result<(), ModelError> {
        if self.workspace.is_some() {
            return Ok(());
        }

        let info = self
            .signature
            .generator_info(generator)
            .ok_or(ModelError::UnknownGeneratorSelected)?;

        self.workspace = Some(Workspace {
            diagram: info.diagram.clone(),
            path: Default::default(),
            view: Default::default(),
            attach: Default::default(),
            attachment_highlight: Default::default(),
            slice_highlight: Default::default(),
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

            self.clear_attach();
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
            if workspace.visible_dimension() < workspace.view.dimension() as usize {
                workspace.view = workspace.view.dec();
            }

            self.clear_attach();
        }

        Ok(())
    }

    /// Handler for [Action::SwitchSlice].
    fn switch_slice(&mut self, direction: Direction) {
        if let Some(workspace) = &mut self.workspace {
            let slice = match workspace.path.pop_back() {
                None => return,
                Some(slice) => slice,
            };

            let diagram = match workspace.visible_diagram() {
                Diagram::Diagram0(_) => unreachable!(),
                Diagram::DiagramN(d) => d,
            };

            let next_slice = slice.step(diagram.size(), direction).unwrap_or(slice);
            workspace.path.push_back(next_slice);
            self.clear_attach();
        }
    }

    /// Handler for [Action::UpdateView]
    fn update_view(&mut self, view: View) {
        if let Some(workspace) = &mut self.workspace {
            workspace.view = view;
        }
    }

    /// Handler for [Action::SelectPoint].
    fn select_points(&mut self, selected: &[Vec<SliceIndex>]) -> Result<(), ModelError> {
        if selected.is_empty() {
            return Ok(());
        }

        let workspace = match &self.workspace {
            Some(workspace) => workspace,
            None => return Ok(()),
        };

        let mut matches: BTreeSet<AttachOption> = BTreeSet::new();

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

            let haystack = match &boundary_path {
                None => workspace.diagram.clone(),
                Some(boundary_path) => DiagramN::try_from(workspace.diagram.clone())
                    .ok()
                    .and_then(|diagram| boundary_path.follow(&diagram))
                    .unwrap(),
            };

            let boundary: Boundary = boundary_path.map_or(Boundary::Target, BoundaryPath::boundary);

            for info in self.signature.iter() {
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
                                boundary_path,
                                generator: info.generator,
                            }),
                    );
                }
            }
        }

        match matches.len() {
            0 => {
                self.clear_attach();
                Err(ModelError::NoAttachment)
            }
            1 => {
                self.attach(&matches.into_iter().next().unwrap());
                Ok(())
            }
            _ => {
                let workspace = self.workspace.as_mut().unwrap();
                workspace.attach = Some(matches.into_iter().collect());
                workspace.attachment_highlight = None;
                Ok(())
            }
        }
    }

    fn attach(&mut self, option: &AttachOption) {
        if let Some(workspace) = &mut self.workspace {
            // TODO: Better error handling, although none of these errors should occur
            let diagram: DiagramN = workspace.diagram.clone().try_into().unwrap();
            let generator: DiagramN = self
                .signature
                .generator_info(option.generator)
                .unwrap()
                .diagram
                .clone()
                .try_into()
                .unwrap();
            let embedding: Vec<_> = option.embedding.iter().copied().collect();

            let result = match &option.boundary_path {
                Some(bp) => diagram
                    .attach(&generator, bp.boundary(), &embedding)
                    .unwrap(),
                None => diagram
                    .identity()
                    .attach(&generator, Boundary::Target, &embedding)
                    .unwrap(),
            };

            // TODO: Figure out what should happen with the slice path
            workspace.diagram = match option.boundary_path {
                Some(_) => result.into(),
                None => result.target(),
            };
        }

        self.clear_attach();
    }

    /// Handler for [Action::HighlightAttachment].
    fn highlight_attachment(&mut self, option: Option<AttachOption>) {
        if let Some(workspace) = &mut self.workspace {
            workspace.attachment_highlight = option;
        }
    }

    /// Handler for [Action::HighlightSlice].
    fn highlight_slice(&mut self, option: Option<SliceIndex>) {
        if let Some(workspace) = &mut self.workspace {
            workspace.slice_highlight = option;
        }
    }

    /// Handler for [Action::Behead].
    fn behead(&mut self) -> Result<(), ModelError> {
        if let Some(ws) = &mut self.workspace {
            let diagram: &DiagramN = (&ws.diagram)
                .try_into()
                .map_err(|_dimerr| ModelError::InvalidAction)?;
            if diagram.size() == 0 {
                return Err(ModelError::InvalidAction);
            }
            let max_height = match ws.path.len() {
                0 => diagram.size() - 1,
                1 => match ws.path[0] {
                    SliceIndex::Boundary(Boundary::Source) => 0,
                    SliceIndex::Boundary(Boundary::Target) => diagram.size(),
                    SliceIndex::Interior(Height::Regular(j)) => j,
                    _ => return Err(ModelError::InvalidAction),
                },
                _ => return Err(ModelError::InvalidAction),
            };
            let beheaded_diagram = diagram.behead(max_height).into();

            ws.diagram = beheaded_diagram;
            ws.path = Default::default();
            self.clear_attach();
        }

        Ok(())
    }

    /// Handler for [Action::Befoot].
    fn befoot(&mut self) -> Result<(), ModelError> {
        if let Some(ws) = &mut self.workspace {
            let diagram: &DiagramN = (&ws.diagram)
                .try_into()
                .map_err(|_dimerr| ModelError::InvalidAction)?;
            if diagram.size() == 0 {
                return Err(ModelError::InvalidAction);
            }

            let min_height = match ws.path.len() {
                0 => 1,
                1 => match ws.path[0] {
                    SliceIndex::Boundary(Boundary::Source) => 0,
                    SliceIndex::Boundary(Boundary::Target) => diagram.size(),
                    SliceIndex::Interior(Height::Regular(j)) => j,
                    _ => return Err(ModelError::InvalidAction),
                },
                _ => return Err(ModelError::InvalidAction),
            };
            let befooted_diagram = diagram.befoot(min_height).into();

            ws.diagram = befooted_diagram;
            ws.path = Default::default();
            self.clear_attach();
        }

        Ok(())
    }

    /// Handler for [Action::Restrict].
    fn restrict(&mut self) -> Result<(), ModelError> {
        if let Some(ws) = &mut self.workspace {
            let mut diagram = ws.diagram.clone();
            for height in &ws.path {
                (matches!(height, SliceIndex::Boundary(_))
                    || matches!(height, SliceIndex::Interior(Height::Regular(_))))
                .then(|| ())
                .ok_or(ModelError::InvalidAction)?;
                diagram = DiagramN::try_from(diagram)
                    .map_err(ModelError::InvalidSlice)?
                    .slice(*height)
                    .ok_or(ModelError::InvalidSlice(DimensionError))?;
            }
            ws.diagram = diagram;
            ws.path = Default::default();
            self.clear_attach();
        }

        Ok(())
    }

    /// Handler for [Action::Theorem].
    fn theorem(&mut self) -> Result<(), ModelError> {
        let diagram: DiagramN = self
            .workspace
            .as_ref()
            .ok_or(ModelError::InvalidAction)?
            .diagram
            .clone()
            .try_into()
            .map_err(|_dimerr| ModelError::InvalidAction)?;

        // new generator of singular height 1 from source to target of current diagram
        let singleton = self
            .signature
            .create_generator(diagram.source(), diagram.target())?;
        // rewrite from singleton to original diagram
        self.signature.create_generator(singleton, diagram.into())?;

        self.clear_workspace();

        Ok(())
    }

    fn homotopy_expansion(&mut self, homotopy: &Expand) -> Result<(), ModelError> {
        let signature = &self.signature;
        let signature = SignatureClosure(|generator| {
            signature
                .generator_info(generator)
                .map(|gi| gi.diagram.clone())
        });

        if let Some(workspace) = &mut self.workspace {
            let diagram: DiagramN = workspace.diagram.clone().try_into().unwrap();

            let location = {
                let mut location: Vec<_> = workspace.path.iter().copied().collect();
                location.extend(homotopy.location.clone());
                location
            };

            let (boundary_path, interior_path) = BoundaryPath::split(&location);

            if let Some(boundary_path) = boundary_path {
                let expanded = diagram.expand(
                    boundary_path,
                    &interior_path,
                    homotopy.direction,
                    &signature,
                )?;
                workspace.diagram = expanded.into();
            } else {
                let expanded = diagram.identity().expand(
                    Boundary::Target.into(),
                    &interior_path,
                    homotopy.direction,
                    &signature,
                )?;
                workspace.diagram = expanded.target();
            }

            // FIXME(@doctorn) this is a stand-in for a more sophisticated approach. Ideally, we
            // would have the path updated such that the image of the slice after the expansion is
            // visible. For now, we just step back up until we find a valid path.
            self.unwind_to_valid_path();
            self.clear_attach();
        }

        Ok(())
    }

    fn homotopy_contraction(&mut self, homotopy: &Contract) -> Result<(), ModelError> {
        // TODO: Proper errors

        let signature = &self.signature;
        let signature = SignatureClosure(|generator| {
            signature
                .generator_info(generator)
                .map(|gi| gi.diagram.clone())
        });

        if let Some(workspace) = &mut self.workspace {
            let diagram: DiagramN = workspace.diagram.clone().try_into().unwrap();
            let location = {
                let mut location: Vec<_> = workspace.path.iter().copied().collect();
                location.extend(homotopy.location.clone());
                location
            };

            let (height, bias) = match homotopy.direction {
                Direction::Forward => (homotopy.height, homotopy.bias),
                Direction::Backward => {
                    if homotopy.height == 0 {
                        // TODO: Show an error
                        log::info!("Contracting off the edge of the diagram.");
                        return Ok(());
                    }

                    let bias = homotopy.bias.map(homotopy_core::Bias::flip);
                    (homotopy.height - 1, bias)
                }
            };

            let (boundary_path, interior_path) = BoundaryPath::split(&location);

            if let Some(boundary_path) = boundary_path {
                let contractum = diagram
                    .contract(boundary_path, &interior_path, height, bias, &signature)
                    .map_err(ModelError::ContractionError)?;
                workspace.diagram = contractum.into();
            } else {
                let contractum = diagram
                    .identity()
                    .contract(
                        Boundary::Target.into(),
                        &interior_path,
                        height,
                        bias,
                        &signature,
                    )
                    .map_err(ModelError::ContractionError)?;
                workspace.diagram = contractum.target();
            }

            // FIXME(@doctorn) see above
            self.unwind_to_valid_path();
            self.clear_attach();
        }

        Ok(())
    }

    pub fn boundary(&self) -> Option<&SelectedBoundary> {
        self.boundary.as_ref()
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

    pub fn unwind_to_valid_path(&mut self) {
        if let Some(workspace) = &mut self.workspace {
            let mut diagram = workspace.diagram.clone();

            for (i, index) in workspace.path.iter().enumerate() {
                match diagram {
                    Diagram::DiagramN(d) if d.slice(*index).is_some() => {
                        diagram = d.slice(*index).unwrap();
                    }
                    _ => {
                        workspace.path = workspace.path.take(i);
                        return;
                    }
                }
            }
        }
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

fn contains_point(diagram: Diagram, point: &[Height], embedding: &[RegularHeight]) -> bool {
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
                Some(slice) => contains_point(slice, point, embedding),
                None => false,
            }
        }
    }
}
