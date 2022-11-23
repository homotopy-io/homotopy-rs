use std::convert::{Into, TryFrom, TryInto};

use homotopy::Homotopy;
use homotopy_core::{
    common::{
        Boundary, BoundaryPath, DimensionError, Direction, Generator, Height, Mode, SliceIndex,
    },
    contraction::ContractionError,
    diagram::{AttachmentError, NewDiagramError},
    expansion::ExpansionError,
    signature::Signature as S,
    Diagram, DiagramN,
};
use im::Vector;
use serde::{Deserialize, Serialize};
pub use signature::*;
use thiserror::Error;

use self::homotopy::{Contract, Expand};
use crate::{migration, serialize};

mod signature;

pub mod generators;
pub mod homotopy;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    dimension: u8,
}

#[cfg(feature = "fuzz")]
impl<'a> arbitrary::Arbitrary<'a> for View {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::new(u.int_in_range(0..=4)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub diagram: Diagram,
    pub path: Vector<SliceIndex>,
    pub view: View,
}

impl Workspace {
    pub fn new(diagram: Diagram) -> Self {
        // Default to 2D unless the diagram has dimension 0 or 1.
        let dimension = diagram.dimension().min(2) as u8;
        Self {
            diagram,
            path: Default::default(),
            view: View { dimension },
        }
    }

    pub fn visible_diagram(&self) -> Diagram {
        self.path
            .iter()
            .fold(self.diagram.clone(), |diagram, index| {
                DiagramN::try_from(diagram).unwrap().slice(*index).unwrap()
            })
    }

    pub fn visible_dimension(&self) -> usize {
        self.diagram.dimension() - self.path.len()
    }
}

impl View {
    const MAX: u8 = 4;

    pub fn dimension(self) -> u8 {
        self.dimension
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedBoundary {
    pub boundary: Boundary,
    pub diagram: Diagram,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProofState {
    pub signature: Signature,
    pub workspace: Option<Workspace>,
    pub metadata: Metadata,
    pub boundary: Option<SelectedBoundary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
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

    IncreaseView(u8),
    DecreaseView(u8),

    Attach(AttachOption),

    Homotopy(Homotopy),

    Behead,

    Befoot,

    Invert,

    Restrict,

    Theorem,

    ImportProof(SerializedData),

    EditSignature(SignatureEdit),

    EditMetadata(MetadataEdit),

    FlipBoundary,

    RecoverBoundary,

    Nothing,
}

#[derive(Debug, Error)]
pub enum ProofError {
    #[error(transparent)]
    NewDiagramError(#[from] NewDiagramError),
    #[error(transparent)]
    AttachmentError(#[from] AttachmentError),
    #[error("selected a generator that is not in the signature")]
    UnknownGeneratorSelected,
    #[error("tried to descend into an invalid diagram slice")]
    InvalidSlice(#[from] DimensionError),
    #[error("invalid action")]
    InvalidAction,
    #[error("the diagram cannot be inverted because not all generators are defined as invertible")]
    NotInvertible,
    #[error(transparent)]
    ExpansionError(#[from] ExpansionError),
    #[error(transparent)]
    ContractionError(#[from] ContractionError),
    #[error("import failed")]
    Import,
}

impl ProofState {
    /// Update the state in response to an [Action].
    pub fn update(&mut self, action: &Action) -> Result<(), ProofError> {
        match action {
            Action::CreateGeneratorZero => {
                self.signature.create_generator_zero("Cell");
            }
            Action::SetBoundary(boundary) => self.set_boundary(*boundary)?,
            Action::TakeIdentityDiagram => self.take_identity_diagram(),
            Action::ClearWorkspace => self.clear_workspace(),
            Action::ClearBoundary => self.clear_boundary(),
            Action::SelectGenerator(generator) => self.select_generator(*generator)?,
            Action::AscendSlice(count) => self.ascend_slice(*count),
            Action::DescendSlice(slice) => self.descend_slice(*slice)?,
            Action::SwitchSlice(direction) => self.switch_slice(*direction),
            Action::IncreaseView(count) => self.increase_view(*count),
            Action::DecreaseView(count) => self.decrease_view(*count),
            Action::Attach(option) => self.attach(option)?,
            Action::Homotopy(Homotopy::Expand(homotopy)) => self.homotopy_expansion(homotopy)?,
            Action::Homotopy(Homotopy::Contract(homotopy)) => {
                self.homotopy_contraction(homotopy)?;
            }
            Action::Behead => self.behead()?,
            Action::Befoot => self.befoot()?,
            Action::Invert => self.invert()?,
            Action::Restrict => self.restrict()?,
            Action::Theorem => self.theorem()?,
            Action::EditSignature(edit) => self.edit_signature(edit)?,
            Action::FlipBoundary => self.flip_boundary(),
            Action::RecoverBoundary => self.recover_boundary(),
            Action::ImportProof(data) => self.import_proof(data)?,
            Action::EditMetadata(edit) => self.edit_metadata(edit),
            Action::Nothing => {}
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
            Action::AscendSlice(i) => i > 0,
            Action::SelectGenerator(_)
            | Action::ClearWorkspace
            | Action::DescendSlice(_)
            | Action::IncreaseView(_)
            | Action::DecreaseView(_) => true,
            _ => false,
        }
    }

    /// Determines if a given [Action] is valid, given the current [ProofState].
    pub fn is_valid(&self, action: &Action) -> bool {
        use homotopy_core::{
            Height::Regular,
            SliceIndex::{Boundary, Interior},
        };
        match *action {
            Action::SetBoundary(_) | Action::TakeIdentityDiagram | Action::ClearWorkspace => {
                self.workspace.is_some()
            }
            Action::Theorem => self
                .workspace
                .as_ref()
                .map_or(false, |ws| ws.diagram.dimension() > 0),
            Action::Restrict => self.workspace.as_ref().map_or(false, |ws| {
                !ws.path.is_empty()
                    && ws
                        .path
                        .iter()
                        .all(|si| matches!(si, Boundary(_) | Interior(Regular(_))))
            }),
            Action::Behead | Action::Befoot => {
                self.workspace
                    .as_ref()
                    .map_or(false, |ws| match &ws.diagram {
                        Diagram::Diagram0(_) => false,
                        Diagram::DiagramN(d) => {
                            d.size() > 0
                                && (ws.path.is_empty()
                                    || (ws.path.len() == 1
                                        && matches!(
                                            ws.path[0],
                                            Boundary(_) | Interior(Regular(_))
                                        )))
                        }
                    })
            }
            Action::Invert => self
                .workspace
                .as_ref()
                .map_or(false, |ws| ws.diagram.dimension() > 0 && ws.path.is_empty()),
            Action::AscendSlice(_) | Action::SwitchSlice(_) => self
                .workspace
                .as_ref()
                .map_or(false, |ws| !ws.path.is_empty()),
            Action::DescendSlice(_) => self
                .workspace
                .as_ref()
                .map_or(false, |ws| ws.visible_dimension() > 0),
            _ => true,
        }
    }

    /// Handler for [Action::ImportProof].
    fn import_proof(&mut self, data: &SerializedData) -> Result<(), ProofError> {
        let ((signature, workspace), metadata) = serialize::deserialize(&data.0)
            .or_else(|| migration::deserialize(&data.0))
            .ok_or(ProofError::Import)?;
        for g in signature.iter() {
            g.diagram
                .check(Mode::Deep)
                .map_err(|_err| ProofError::Import)?;
        }
        if let Some(w) = workspace.as_ref() {
            w.diagram
                .check(Mode::Deep)
                .map_err(|_err| ProofError::Import)?;
        }
        self.signature = signature;
        self.workspace = workspace;
        self.metadata = metadata;
        Ok(())
    }

    /// Handler for [Action::EditSignature].
    fn edit_signature(&mut self, edit: &SignatureEdit) -> Result<(), ProofError> {
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

        if let SignatureEdit::Edit(node, SignatureItemEdit::MakeOriented(true)) = edit {
            let generator = self.signature.find_generator(*node).unwrap();

            // remove framing from the workspace
            if let Some(ws) = &mut self.workspace {
                ws.diagram = ws.diagram.remove_framing(generator);
            }

            // remove framing from the boundary
            if let Some(b) = &mut self.boundary {
                b.diagram = b.diagram.remove_framing(generator);
            }
        }

        self.signature.update(edit)
    }

    fn edit_metadata(&mut self, edit: &MetadataEdit) {
        match edit {
            MetadataEdit::Title(title) => self.metadata.title = Some(title.clone()),
            MetadataEdit::Author(author) => self.metadata.author = Some(author.clone()),
            MetadataEdit::Abstract(abstr) => self.metadata.abstr = Some(abstr.clone()),
        }
    }

    /// Handler for [Action::SetBoundary].
    fn set_boundary(&mut self, boundary: Boundary) -> Result<(), ProofError> {
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
                        .create_generator(source, target, "Cell", false)?;

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

    /// Handler for [Action::FlipBoundary].
    fn flip_boundary(&mut self) {
        if let Some(selected) = &self.boundary {
            self.boundary = Some(SelectedBoundary {
                boundary: selected.boundary.flip(),
                diagram: selected.diagram.clone(),
            });
        };
    }

    /// Handler for [Action::RecoverBoundary]
    fn recover_boundary(&mut self) {
        if self.workspace.is_none() {
            if let Some(selected) = &self.boundary {
                self.workspace = Some(Workspace::new(selected.diagram.clone()));
            }
        }
    }

    /// Handler for [Action::TakeIdentityDiagram].
    fn take_identity_diagram(&mut self) {
        match &mut self.workspace {
            Some(workspace) => {
                if workspace.diagram.dimension() + workspace.path.len() >= 2 {
                    workspace.path.push_front(Boundary::Target.into());
                } else {
                    workspace.view.dimension += 1;
                }

                workspace.diagram = workspace.diagram.clone().identity().into();
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
    fn select_generator(&mut self, generator: Generator) -> Result<(), ProofError> {
        let info = self
            .signature
            .generator_info(generator)
            .ok_or(ProofError::UnknownGeneratorSelected)?;

        self.workspace = Some(Workspace::new(info.diagram.clone()));

        Ok(())
    }

    /// Handler for [Action::AscendSlice].
    fn ascend_slice(&mut self, mut count: usize) {
        if let Some(workspace) = &mut self.workspace {
            while count > 0 && !workspace.path.is_empty() {
                workspace.path.pop_back();
                count -= 1;

                // Boost the view dimension if necessary.
                if workspace.view.dimension < 2 {
                    workspace.view.dimension += 1;
                }
            }
        }
    }

    /// Handler for [Action::DescendSlice].
    fn descend_slice(&mut self, slice: SliceIndex) -> Result<(), ProofError> {
        if let Some(workspace) = &mut self.workspace {
            let mut path = workspace.path.clone();
            path.push_back(slice);

            // Check if path is valid
            let mut part = workspace.diagram.clone();
            for height in &path {
                part = DiagramN::try_from(part)
                    .map_err(ProofError::InvalidSlice)?
                    .slice(*height)
                    .ok_or(ProofError::InvalidSlice(DimensionError))?;
            }

            // Update workspace
            workspace.path = path;
            if workspace.visible_dimension() < workspace.view.dimension as usize {
                workspace.view.dimension -= 1;
            }
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
        }
    }

    /// Handler for [Action::IncreaseView].
    fn increase_view(&mut self, count: u8) {
        if let Some(workspace) = &mut self.workspace {
            workspace.view.dimension = std::cmp::min(
                workspace.view.dimension + count,
                std::cmp::min(workspace.visible_dimension() as u8, View::MAX),
            );
        }
    }

    /// Handler for [Action::DecreaseView].
    fn decrease_view(&mut self, count: u8) {
        if let Some(workspace) = &mut self.workspace {
            workspace.view.dimension = workspace.view.dimension.saturating_sub(count);
        }
    }

    fn attach(&mut self, option: &AttachOption) -> Result<(), ProofError> {
        if let Some(workspace) = &mut self.workspace {
            // TODO: Better error handling, although none of these errors should occur
            let attachment: &DiagramN = &option.diagram;
            let embedding: Vec<_> = option.embedding.iter().copied().collect();

            workspace.diagram = match &option.boundary_path {
                Some(bp) => <&DiagramN>::try_from(&workspace.diagram)?
                    .attach(attachment, bp.boundary(), &embedding)?
                    .into(),
                None => workspace
                    .diagram
                    .clone()
                    .identity()
                    .attach(attachment, Boundary::Target, &embedding)?
                    .target(),
            };
        }

        Ok(())
    }

    /// Handler for [Action::Behead].
    fn behead(&mut self) -> Result<(), ProofError> {
        if let Some(ws) = &mut self.workspace {
            let diagram: &DiagramN = (&ws.diagram)
                .try_into()
                .map_err(|_dimerr| ProofError::InvalidAction)?;
            if diagram.size() == 0 {
                return Err(ProofError::InvalidAction);
            }
            let max_height = match ws.path.len() {
                0 => diagram.size() - 1,
                1 => match ws.path[0] {
                    SliceIndex::Boundary(Boundary::Source) => 0,
                    SliceIndex::Boundary(Boundary::Target) => diagram.size(),
                    SliceIndex::Interior(Height::Regular(j)) => j,
                    _ => return Err(ProofError::InvalidAction),
                },
                _ => return Err(ProofError::InvalidAction),
            };
            let beheaded_diagram = diagram.behead(max_height).into();

            ws.diagram = beheaded_diagram;
            ws.path = Default::default();
        }

        Ok(())
    }

    /// Handler for [Action::Befoot].
    fn befoot(&mut self) -> Result<(), ProofError> {
        if let Some(ws) = &mut self.workspace {
            let diagram: &DiagramN = (&ws.diagram)
                .try_into()
                .map_err(|_dimerr| ProofError::InvalidAction)?;
            if diagram.size() == 0 {
                return Err(ProofError::InvalidAction);
            }

            let min_height = match ws.path.len() {
                0 => 1,
                1 => match ws.path[0] {
                    SliceIndex::Boundary(Boundary::Source) => 0,
                    SliceIndex::Boundary(Boundary::Target) => diagram.size(),
                    SliceIndex::Interior(Height::Regular(j)) => j,
                    _ => return Err(ProofError::InvalidAction),
                },
                _ => return Err(ProofError::InvalidAction),
            };
            let befooted_diagram = diagram.befoot(min_height).into();

            ws.diagram = befooted_diagram;
            ws.path = Default::default();
        }

        Ok(())
    }

    /// Handler for [Action::Invert].
    fn invert(&mut self) -> Result<(), ProofError> {
        if let Some(ws) = &mut self.workspace {
            if ws.diagram.is_invertible(&self.signature) {
                let diagram: &DiagramN = (&ws.diagram)
                    .try_into()
                    .map_err(|_dimerr| ProofError::InvalidAction)?;
                ws.diagram = diagram.inverse().into();
            } else {
                return Err(ProofError::NotInvertible);
            }
        }

        Ok(())
    }

    /// Handler for [Action::Restrict].
    fn restrict(&mut self) -> Result<(), ProofError> {
        if let Some(ws) = &mut self.workspace {
            let mut diagram = ws.diagram.clone();
            for height in &ws.path {
                (matches!(height, SliceIndex::Boundary(_))
                    || matches!(height, SliceIndex::Interior(Height::Regular(_))))
                .then_some(())
                .ok_or(ProofError::InvalidAction)?;
                diagram = DiagramN::try_from(diagram)
                    .map_err(ProofError::InvalidSlice)?
                    .slice(*height)
                    .ok_or(ProofError::InvalidSlice(DimensionError))?;
            }
            ws.diagram = diagram;
            ws.path = Default::default();
        }

        Ok(())
    }

    /// Handler for [Action::Theorem].
    fn theorem(&mut self) -> Result<(), ProofError> {
        let diagram: DiagramN = self
            .workspace
            .as_ref()
            .ok_or(ProofError::InvalidAction)?
            .diagram
            .clone()
            .try_into()
            .map_err(|_dimerr| ProofError::InvalidAction)?;

        let invertible = Diagram::from(diagram.clone()).is_invertible(&self.signature);
        // new generator of singular height 1 from source to target of current diagram
        let singleton = self.signature.create_generator(
            diagram.source(),
            diagram.target(),
            "Theorem",
            invertible,
        )?;

        // rewrite from singleton to original diagram
        self.signature
            .create_generator(singleton.into(), diagram.into(), "Proof", true)?;

        self.clear_workspace();

        Ok(())
    }

    fn homotopy_expansion(&mut self, homotopy: &Expand) -> Result<(), ProofError> {
        if let Some(workspace) = &mut self.workspace {
            let diagram: DiagramN = workspace.diagram.clone().try_into()?;

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
                    &self.signature,
                )?;
                workspace.diagram = expanded.into();
            } else {
                let expanded = diagram.identity().expand(
                    Boundary::Target.into(),
                    &interior_path,
                    homotopy.direction,
                    &self.signature,
                )?;
                workspace.diagram = expanded.target();
            }

            // FIXME(@doctorn) this is a stand-in for a more sophisticated approach. Ideally, we
            // would have the path updated such that the image of the slice after the expansion is
            // visible. For now, we just step back up until we find a valid path.
            self.unwind_to_valid_path();
        }

        Ok(())
    }

    fn homotopy_contraction(&mut self, homotopy: &Contract) -> Result<(), ProofError> {
        if let Some(workspace) = &mut self.workspace {
            let diagram: DiagramN = workspace.diagram.clone().try_into()?;
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
                    .contract(boundary_path, &interior_path, height, bias, &self.signature)
                    .map_err(ProofError::ContractionError)?;
                workspace.diagram = contractum.into();
            } else {
                let contractum = diagram
                    .identity()
                    .contract(
                        Boundary::Target.into(),
                        &interior_path,
                        height,
                        bias,
                        &self.signature,
                    )
                    .map_err(ProofError::ContractionError)?;
                workspace.diagram = contractum.target();
            }

            // FIXME(@doctorn) see above
            self.unwind_to_valid_path();
        }

        Ok(())
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttachOption {
    pub generator: Generator,
    pub diagram: DiagramN,
    pub tag: Option<String>,
    pub boundary_path: Option<BoundaryPath>,
    pub embedding: Vector<usize>,
}

#[cfg(feature = "fuzz")]
impl<'a> arbitrary::Arbitrary<'a> for AttachOption {
    fn arbitrary(_u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        /*
        Ok(AttachOption {
            generator: u.arbitrary()?,
            diagram: u.arbitrary()?,
            tag: u.arbitrary()?,
            boundary_path: u.arbitrary()?,
            embedding: Vector::from(u.arbitrary::<Vec<_>>()?),
        })
        */
        Err(arbitrary::Error::EmptyChoose)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedData(pub Vec<u8>);

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
