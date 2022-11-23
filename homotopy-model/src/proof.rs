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

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
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

impl Action {
    /// Determines if a given [Action] is valid given the current [ProofState].
    #[allow(clippy::match_same_arms)]
    pub fn is_valid(&self, proof: &ProofState) -> bool {
        use homotopy_core::{Height::Singular, SliceIndex::Interior};
        match self {
            Self::CreateGeneratorZero => true,
            Self::SetBoundary(_) => proof.workspace.is_some(),
            Self::TakeIdentityDiagram => proof.workspace.is_some(),
            Self::ClearWorkspace => proof.workspace.is_some(),
            Self::ClearBoundary => proof.boundary.is_some(),
            Self::SelectGenerator(_) => true,
            Self::AscendSlice(count) => proof
                .workspace
                .as_ref()
                .map_or(false, |ws| ws.path.len() >= *count),
            Self::DescendSlice(_) => proof
                .workspace
                .as_ref()
                .map_or(false, |ws| ws.visible_dimension() > 0),
            Self::SwitchSlice(_) => proof.workspace.as_ref().map_or(false, |ws| {
                ws.path
                    .last()
                    .map_or(false, |index| matches!(index, Interior(_)))
            }),
            Self::IncreaseView(count) => {
                *count > 0
                    && proof.workspace.as_ref().map_or(false, |ws| {
                        ws.view.dimension + *count
                            <= std::cmp::min(ws.visible_dimension() as u8, View::MAX)
                    })
            }
            Self::DecreaseView(count) => {
                *count > 0
                    && proof
                        .workspace
                        .as_ref()
                        .map_or(false, |ws| ws.view.dimension >= *count)
            }
            Self::Attach(option) => proof.workspace.as_ref().map_or(false, |ws| {
                option.boundary_path.is_none() || ws.diagram.dimension() > 0
            }),
            Self::Homotopy(_) => proof
                .workspace
                .as_ref()
                .map_or(false, |ws| ws.diagram.dimension() > 0),
            Self::Behead | Self::Befoot => {
                proof
                    .workspace
                    .as_ref()
                    .map_or(false, |ws| match &ws.diagram {
                        Diagram::Diagram0(_) => false,
                        Diagram::DiagramN(diagram) => {
                            (ws.path.is_empty() && diagram.size() > 0)
                                || (ws.path.len() == 1
                                    && !matches!(ws.path[0], Interior(Singular(_))))
                        }
                    })
            }
            Self::Invert => proof
                .workspace
                .as_ref()
                .map_or(false, |ws| ws.diagram.dimension() > 0),
            Self::Restrict => proof.workspace.as_ref().map_or(false, |ws| {
                !ws.path.is_empty()
                    && ws
                        .path
                        .iter()
                        .all(|index| !matches!(index, Interior(Singular(_))))
            }),
            Self::Theorem => proof
                .workspace
                .as_ref()
                .map_or(false, |ws| ws.diagram.dimension() > 0),
            Self::ImportProof(_) => true,
            Self::EditSignature(_) | Self::EditMetadata(_) => true, /* technically the edits could be trivial but do not worry about that for now */
            Self::FlipBoundary | Self::RecoverBoundary => proof.boundary.is_some(),
            Self::Nothing => false,
        }
    }
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
    InvalidSlice,
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

impl From<DimensionError> for ProofError {
    fn from(_: DimensionError) -> Self {
        Self::InvalidAction
    }
}

impl ProofState {
    /// Update the state in response to an [Action].
    pub fn update(&mut self, action: &Action) -> Result<(), ProofError> {
        match action {
            Action::CreateGeneratorZero => self.create_generator_zero(),
            Action::SetBoundary(boundary) => self.set_boundary(*boundary),
            Action::TakeIdentityDiagram => self.take_identity_diagram(),
            Action::ClearWorkspace => self.clear_workspace(),
            Action::ClearBoundary => self.clear_boundary(),
            Action::SelectGenerator(generator) => self.select_generator(*generator),
            Action::AscendSlice(count) => self.ascend_slice(*count),
            Action::DescendSlice(slice) => self.descend_slice(*slice),
            Action::SwitchSlice(direction) => self.switch_slice(*direction),
            Action::IncreaseView(count) => self.increase_view(*count),
            Action::DecreaseView(count) => self.decrease_view(*count),
            Action::Attach(option) => self.attach(option),
            Action::Homotopy(Homotopy::Expand(homotopy)) => self.homotopy_expansion(homotopy),
            Action::Homotopy(Homotopy::Contract(homotopy)) => self.homotopy_contraction(homotopy),
            Action::Behead => self.behead(),
            Action::Befoot => self.befoot(),
            Action::Invert => self.invert(),
            Action::Restrict => self.restrict(),
            Action::Theorem => self.theorem(),
            Action::EditSignature(edit) => self.edit_signature(edit),
            Action::FlipBoundary => self.flip_boundary(),
            Action::RecoverBoundary => self.recover_boundary(),
            Action::ImportProof(data) => self.import_proof(data),
            Action::EditMetadata(edit) => self.edit_metadata(edit),
            Action::Nothing => Err(ProofError::InvalidAction),
        }
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

    /// Handler for [Action::CreateGeneratorZero].
    #[allow(clippy::unnecessary_wraps)]
    fn create_generator_zero(&mut self) -> Result<(), ProofError> {
        self.signature.create_generator_zero("Cell");
        Ok(())
    }

    /// Handler for [Action::SetBoundary].
    ///
    /// Invalid if the workspace is empty.
    /// Returns an error if the diagrams are incompatible as boundaries.
    fn set_boundary(&mut self, boundary: Boundary) -> Result<(), ProofError> {
        let ws = self.workspace.take().ok_or(ProofError::InvalidAction)?;

        match self.boundary.take() {
            Some(selected) if selected.boundary != boundary => {
                let (source, target) = match boundary {
                    Boundary::Source => (ws.diagram, selected.diagram),
                    Boundary::Target => (selected.diagram, ws.diagram),
                };
                self.signature
                    .create_generator(source, target, "Cell", false)?;
            }
            _ => {
                self.boundary = Some(SelectedBoundary {
                    boundary,
                    diagram: ws.diagram,
                });
            }
        }

        Ok(())
    }

    /// Handler for [Action::TakeIdentityDiagram].
    ///
    /// Invalid if the workspace is empty.
    fn take_identity_diagram(&mut self) -> Result<(), ProofError> {
        let ws = self.workspace.as_mut().ok_or(ProofError::InvalidAction)?;

        if ws.diagram.dimension() + ws.path.len() >= 2 {
            ws.path.push_front(Boundary::Target.into());
        } else {
            ws.view.dimension += 1;
        }

        ws.diagram = ws.diagram.clone().identity().into();

        Ok(())
    }

    /// Handler for [Action::ClearWorkspace].
    ///
    /// Invalid if the workspace is empty.
    fn clear_workspace(&mut self) -> Result<(), ProofError> {
        self.workspace.take().ok_or(ProofError::InvalidAction)?;
        Ok(())
    }

    /// Handler for [Action::ClearBoundary].
    ///
    /// Invalid if the selected boundary is empty.
    fn clear_boundary(&mut self) -> Result<(), ProofError> {
        self.boundary.take().ok_or(ProofError::InvalidAction)?;
        Ok(())
    }

    /// Handler for [Action::SelectGenerator].
    ///
    /// Returns an error if the generator is not in the signature.
    fn select_generator(&mut self, generator: Generator) -> Result<(), ProofError> {
        let info = self
            .signature
            .generator_info(generator)
            .ok_or(ProofError::UnknownGeneratorSelected)?;

        self.workspace = Some(Workspace::new(info.diagram.clone()));

        Ok(())
    }

    /// Handler for [Action::AscendSlice].
    ///
    /// Invalid if the workspace is empty or the path is too short.
    fn ascend_slice(&mut self, count: usize) -> Result<(), ProofError> {
        let ws = self.workspace.as_mut().ok_or(ProofError::InvalidAction)?;

        for _ in 0..count {
            ws.path.pop_back().ok_or(ProofError::InvalidAction)?;

            if ws.view.dimension < 2 {
                ws.view.dimension += 1;
            }
        }

        Ok(())
    }

    /// Handler for [Action::DescendSlice].
    ///
    /// Invalid if the workspace is empty or has the wrong dimension.
    ///
    /// Returns an error if the slice is not a valid slice of the diagram.
    fn descend_slice(&mut self, slice: SliceIndex) -> Result<(), ProofError> {
        let ws = self.workspace.as_mut().ok_or(ProofError::InvalidAction)?;

        let diagram: DiagramN = ws.visible_diagram().try_into()?;

        if let SliceIndex::Interior(height) = slice {
            if height > Height::Regular(diagram.size()) {
                return Err(ProofError::InvalidSlice);
            }
        }

        ws.path.push_back(slice);
        ws.view.dimension = ws.view.dimension.min(ws.visible_dimension() as u8);

        Ok(())
    }

    /// Handler for [Action::SwitchSlice].
    ///
    /// Invalid if the workspace is empty, the path is empty, or we cannot step in the given direction.
    fn switch_slice(&mut self, direction: Direction) -> Result<(), ProofError> {
        let ws = self.workspace.as_mut().ok_or(ProofError::InvalidAction)?;

        let slice = ws.path.pop_back().ok_or(ProofError::InvalidAction)?;

        let diagram = DiagramN::try_from(ws.visible_diagram()).unwrap();
        ws.path.push_back(
            slice
                .step(diagram.size(), direction)
                .ok_or(ProofError::InvalidAction)?,
        );

        Ok(())
    }

    /// Handler for [Action::IncreaseView].
    ///
    /// Invalid if the workspace is empty or the view dimension is too high.
    fn increase_view(&mut self, count: u8) -> Result<(), ProofError> {
        let ws = self.workspace.as_mut().ok_or(ProofError::InvalidAction)?;

        if ws.view.dimension + count > std::cmp::min(ws.visible_dimension() as u8, View::MAX) {
            return Err(ProofError::InvalidAction);
        }

        ws.view.dimension += count;

        Ok(())
    }

    /// Handler for [Action::DecreaseView].
    ///
    /// Invalid if the workspace is empty or the view dimension is too low.
    fn decrease_view(&mut self, count: u8) -> Result<(), ProofError> {
        let ws = self.workspace.as_mut().ok_or(ProofError::InvalidAction)?;

        if ws.view.dimension < count {
            return Err(ProofError::InvalidAction);
        }

        ws.view.dimension -= count;

        Ok(())
    }

    /// Handler for [Action::Attach].
    ///
    /// Invalid if the workspace is empty or has dimension 0 (if the boundary path is not null).
    fn attach(&mut self, option: &AttachOption) -> Result<(), ProofError> {
        let ws = self.workspace.as_mut().ok_or(ProofError::InvalidAction)?;
        let diagram = &mut ws.diagram;

        let embedding: Vec<_> = option.embedding.iter().copied().collect();

        if let Some(bp) = &option.boundary_path {
            let diagram: &mut DiagramN = diagram.try_into()?;
            *diagram = diagram.attach(&option.diagram, bp.boundary(), &embedding)?;
        } else {
            *diagram = diagram
                .clone()
                .identity()
                .attach(&option.diagram, Boundary::Target, &embedding)?
                .target();
        }

        Ok(())
    }

    /// Handler for [Action::Homotopy].
    ///
    /// Invalid if the workspace is empty or has dimension 0.
    fn homotopy_expansion(&mut self, homotopy: &Expand) -> Result<(), ProofError> {
        let ws = self.workspace.as_mut().ok_or(ProofError::InvalidAction)?;
        let diagram = &mut ws.diagram;

        let location = {
            let mut location: Vec<_> = ws.path.iter().copied().collect();
            location.extend(homotopy.location.clone());
            location
        };

        let (boundary_path, interior_path) = BoundaryPath::split(&location);

        if let Some(boundary_path) = boundary_path {
            let diagram: &mut DiagramN = diagram.try_into()?;
            *diagram = diagram.expand(
                boundary_path,
                &interior_path,
                homotopy.direction,
                &self.signature,
            )?;
        } else {
            *diagram = diagram
                .clone()
                .identity()
                .expand(
                    Boundary::Target.into(),
                    &interior_path,
                    homotopy.direction,
                    &self.signature,
                )?
                .target();
        }

        // FIXME(@doctorn) this is a stand-in for a more sophisticated approach. Ideally, we
        // would have the path updated such that the image of the slice after the expansion is
        // visible. For now, we just step back up until we find a valid path.
        self.unwind_to_valid_path();

        Ok(())
    }

    /// Handler for [Action::Homotopy].
    ///
    /// Invalid if the workspace is empty or has dimension 0.
    fn homotopy_contraction(&mut self, homotopy: &Contract) -> Result<(), ProofError> {
        let ws = self.workspace.as_mut().ok_or(ProofError::InvalidAction)?;
        let diagram = &mut ws.diagram;

        let location = {
            let mut location: Vec<_> = ws.path.iter().copied().collect();
            location.extend(homotopy.location.clone());
            location
        };

        let (boundary_path, interior_path) = BoundaryPath::split(&location);

        if let Some(boundary_path) = boundary_path {
            let diagram: &mut DiagramN = diagram.try_into()?;
            *diagram = diagram.contract(
                boundary_path,
                &interior_path,
                homotopy.height,
                homotopy.direction,
                homotopy.bias,
                &self.signature,
            )?;
        } else {
            *diagram = diagram
                .clone()
                .identity()
                .contract(
                    Boundary::Target.into(),
                    &interior_path,
                    homotopy.height,
                    homotopy.direction,
                    homotopy.bias,
                    &self.signature,
                )?
                .target();
        }

        // FIXME(@doctorn) see above
        self.unwind_to_valid_path();

        Ok(())
    }

    /// Handler for [Action::Behead].
    ///
    /// Invalid if the workspace is empty or has dimension 0, or if the path is invalid.
    fn behead(&mut self) -> Result<(), ProofError> {
        let ws = self.workspace.as_mut().ok_or(ProofError::InvalidAction)?;
        let diagram: &DiagramN = (&ws.diagram).try_into()?;

        let max_height = match ws.path.len() {
            0 if diagram.size() > 0 => diagram.size() - 1,
            1 => match ws.path[0] {
                SliceIndex::Boundary(Boundary::Source) => 0,
                SliceIndex::Boundary(Boundary::Target) => diagram.size(),
                SliceIndex::Interior(Height::Regular(j)) => j,
                _ => return Err(ProofError::InvalidAction),
            },
            _ => return Err(ProofError::InvalidAction),
        };

        ws.diagram = diagram.behead(max_height).into();
        ws.path = Default::default();

        Ok(())
    }

    /// Handler for [Action::Befoot].
    ///
    /// Invalid if the workspace is empty or has dimension 0, or if the path is invalid.
    fn befoot(&mut self) -> Result<(), ProofError> {
        let ws = self.workspace.as_mut().ok_or(ProofError::InvalidAction)?;
        let diagram: &DiagramN = (&ws.diagram).try_into()?;

        let min_height = match ws.path.len() {
            0 if diagram.size() > 0 => 1,
            1 => match ws.path[0] {
                SliceIndex::Boundary(Boundary::Source) => 0,
                SliceIndex::Boundary(Boundary::Target) => diagram.size(),
                SliceIndex::Interior(Height::Regular(j)) => j,
                _ => return Err(ProofError::InvalidAction),
            },
            _ => return Err(ProofError::InvalidAction),
        };

        ws.diagram = diagram.befoot(min_height).into();
        ws.path = Default::default();

        Ok(())
    }

    /// Handler for [Action::Invert].
    ///
    /// Invalid if the workspace is empty or has dimension 0.
    ///
    /// Returns an error if the diagram cannot be inverted (if not all generators are invertible).
    fn invert(&mut self) -> Result<(), ProofError> {
        let ws = self.workspace.as_mut().ok_or(ProofError::InvalidAction)?;

        if !ws.diagram.is_invertible(&self.signature) {
            return Err(ProofError::NotInvertible);
        }

        let diagram: &mut DiagramN = (&mut ws.diagram).try_into()?;
        *diagram = diagram.inverse();

        self.unwind_to_valid_path();

        Ok(())
    }

    /// Handler for [Action::Restrict].
    ///
    /// Invalid if the workspace is empty, or if the path is empty or contains a singular slice.
    fn restrict(&mut self) -> Result<(), ProofError> {
        let ws = self.workspace.as_mut().ok_or(ProofError::InvalidAction)?;

        if ws.path.is_empty()
            || ws
                .path
                .iter()
                .any(|index| matches!(index, SliceIndex::Interior(Height::Singular(_))))
        {
            return Err(ProofError::InvalidAction);
        }

        ws.diagram = ws.visible_diagram();
        ws.path = Default::default();

        Ok(())
    }

    /// Handler for [Action::Theorem].
    ///
    /// Invalid if the workspace is empty or has dimension 0.
    fn theorem(&mut self) -> Result<(), ProofError> {
        let diagram = self
            .workspace
            .take()
            .ok_or(ProofError::InvalidAction)?
            .diagram;

        let invertible = diagram.is_invertible(&self.signature);
        let diagram: DiagramN = diagram.try_into()?;

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

        Ok(())
    }

    /// Handler for [Action::ImportProof].
    fn import_proof(&mut self, data: &SerializedData) -> Result<(), ProofError> {
        let ((signature, workspace), metadata) = serialize::deserialize(&data.0)
            .or_else(|| migration::deserialize(&data.0))
            .ok_or(ProofError::Import)?;
        for info in signature.iter() {
            info.diagram
                .check(Mode::Deep)
                .map_err(|_err| ProofError::Import)?;
        }
        if let Some(workspace) = workspace.as_ref() {
            workspace
                .diagram
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
                    self.workspace = None;
                }
            }
            // remove from the boundary
            if let Some(selected) = &self.boundary {
                if self.signature.has_descendents_in(*node, &selected.diagram) {
                    self.boundary = None;
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
            if let Some(selected) = &mut self.boundary {
                selected.diagram = selected.diagram.remove_framing(generator);
            }
        }

        self.signature.update(edit)
    }

    /// Handler for [Action::EditMetadata].
    #[allow(clippy::unnecessary_wraps)]
    fn edit_metadata(&mut self, edit: &MetadataEdit) -> Result<(), ProofError> {
        match edit {
            MetadataEdit::Title(title) => self.metadata.title = Some(title.clone()),
            MetadataEdit::Author(author) => self.metadata.author = Some(author.clone()),
            MetadataEdit::Abstract(abstr) => self.metadata.abstr = Some(abstr.clone()),
        }

        Ok(())
    }

    /// Handler for [Action::FlipBoundary].
    ///
    /// Invalid if the selected boundary is empty.
    fn flip_boundary(&mut self) -> Result<(), ProofError> {
        let selected = self.boundary.as_mut().ok_or(ProofError::InvalidAction)?;
        selected.boundary = selected.boundary.flip();
        Ok(())
    }

    /// Handler for [Action::RecoverBoundary].
    ///
    /// Invalid if the selected boundary is empty.
    fn recover_boundary(&mut self) -> Result<(), ProofError> {
        let selected = self.boundary.take().ok_or(ProofError::InvalidAction)?;
        self.workspace = Some(Workspace::new(selected.diagram));
        Ok(())
    }

    fn unwind_to_valid_path(&mut self) {
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
