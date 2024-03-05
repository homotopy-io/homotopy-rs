use std::convert::{Into, TryFrom};

use homotopy::Homotopy;
use homotopy_core::{
    common::{Boundary, BoundaryPath, Direction, Generator, Height, Mode, SliceIndex},
    contraction::ContractionError,
    diagram::{AttachmentError, NewDiagramError},
    expansion::ExpansionError,
    signature::Signature as _,
    Diagram, Diagram0, DiagramN, Orientation,
};
use im::Vector;
use serde::{Deserialize, Serialize};
pub use signature::*;
use thiserror::Error;

use self::homotopy::{Contract, Expand};
use crate::{migration, proof::generators::GeneratorInfo, serialize};

mod signature;

pub mod generators;
pub mod homotopy;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct View {
    dimension: u8,
}

impl View {
    const MAX: u8 = 4;

    #[must_use]
    pub const fn dimension(self) -> u8 {
        self.dimension
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Workspace {
    pub view: View,
    pub diagram: Diagram,
    pub path: Vector<SliceIndex>,
}

impl Workspace {
    #[must_use]
    pub fn new(diagram: Diagram) -> Self {
        // Default to 2D unless the diagram has dimension 0 or 1.
        let dimension = diagram.dimension().min(2) as u8;
        Self {
            view: View { dimension },
            diagram,
            path: Default::default(),
        }
    }

    #[must_use]
    pub fn visible_diagram(&self) -> Diagram {
        self.path
            .iter()
            .fold(self.diagram.clone(), |diagram, index| {
                DiagramN::try_from(diagram).unwrap().slice(*index).unwrap()
            })
    }

    #[must_use]
    pub fn visible_dimension(&self) -> usize {
        self.diagram.dimension() - self.path.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedBoundary {
    pub boundary: Boundary,
    pub diagram: Diagram,
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct ProofState {
    pub signature: Signature,
    pub workspace: Option<Workspace>,
    pub metadata: Metadata,
    pub boundary: Option<SelectedBoundary>,
    pub stash: Vector<Workspace>,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
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

    Squash,

    Behead,

    Befoot,

    Invert,

    Restrict,

    Theorem,

    SuspendSignature,

    Suspend(Generator, Generator),

    Merge(Generator, Generator),

    ImportProof(SerializedData),

    EditSignature(SignatureEdit),

    EditMetadata(MetadataEdit),

    FlipBoundary,

    RecoverBoundary,

    Stash,

    StashDrop,

    StashPop,

    StashApply,

    Nothing,
}

impl Action {
    /// Determines if a given [Action] is valid given the current [ProofState].
    ///
    /// This should return true iff performing the action does *not* return false.
    #[allow(clippy::match_same_arms)]
    #[must_use]
    pub fn is_valid(&self, proof: &ProofState) -> bool {
        use homotopy_core::{Height::Singular, SliceIndex::Interior};
        match self {
            Self::CreateGeneratorZero => true,
            Self::SetBoundary(_) => proof.workspace.is_some(),
            Self::TakeIdentityDiagram => proof.workspace.is_some(),
            Self::ClearWorkspace => proof.workspace.is_some(),
            Self::ClearBoundary => proof.boundary.is_some(),
            Self::SelectGenerator(_) => true,
            Self::AscendSlice(count) => {
                *count > 0
                    && proof
                        .workspace
                        .as_ref()
                        .is_some_and(|ws| !ws.path.is_empty())
            }
            Self::DescendSlice(_) => proof
                .workspace
                .as_ref()
                .is_some_and(|ws| ws.visible_dimension() > 0),
            Self::SwitchSlice(_) => proof.workspace.as_ref().is_some_and(|ws| {
                ws.path
                    .last()
                    .is_some_and(|index| matches!(index, Interior(_)))
            }),
            Self::IncreaseView(count) => {
                *count > 0
                    && proof.workspace.as_ref().is_some_and(|ws| {
                        ws.view.dimension < std::cmp::min(ws.visible_dimension() as u8, View::MAX)
                    })
            }
            Self::DecreaseView(count) => {
                *count > 0
                    && proof
                        .workspace
                        .as_ref()
                        .is_some_and(|ws| ws.view.dimension > 0)
            }
            Self::Attach(option) => proof
                .workspace
                .as_ref()
                .is_some_and(|ws| option.boundary_path.is_none() || ws.diagram.dimension() > 0),
            Self::Homotopy(_) => proof
                .workspace
                .as_ref()
                .is_some_and(|ws| ws.diagram.dimension() > 0),
            Self::Squash => proof
                .workspace
                .as_ref()
                .is_some_and(|ws| ws.visible_diagram().size().is_some_and(|size| size > 0)),
            Self::Behead | Self::Befoot => {
                proof
                    .workspace
                    .as_ref()
                    .is_some_and(|ws| match &ws.diagram {
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
                .is_some_and(|ws| ws.path.is_empty() && ws.diagram.dimension() > 0),
            Self::Restrict => proof.workspace.as_ref().is_some_and(|ws| {
                !ws.path.is_empty()
                    && ws
                        .path
                        .iter()
                        .all(|index| !matches!(index, Interior(Singular(_))))
            }),
            Self::Theorem => proof
                .workspace
                .as_ref()
                .is_some_and(|ws| ws.diagram.dimension() > 0),
            Self::Suspend(_, _) | Self::SuspendSignature => proof.signature.has_generators(),
            Self::Merge(_, _) => true,
            Self::ImportProof(_) => true,
            Self::EditSignature(_) | Self::EditMetadata(_) => true, /* technically the edits could be trivial but do not worry about that for now */
            Self::FlipBoundary | Self::RecoverBoundary => proof.boundary.is_some(),
            Self::Stash => proof.workspace.is_some(),
            Self::StashDrop | Self::StashPop | Self::StashApply => !proof.stash.is_empty(),
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
    #[error("the diagram cannot be inverted because not all generators are defined as invertible")]
    NotInvertible,
    #[error(transparent)]
    ExpansionError(#[from] ExpansionError),
    #[error(transparent)]
    ContractionError(#[from] ContractionError),
    #[error("import failed")]
    Import,
    #[error(transparent)]
    SignatureError(#[from] SignatureError),
}

impl ProofState {
    /// Update the state in response to an [Action].
    ///
    /// Returns a boolean indicating if the state was updated.
    pub fn update(&mut self, action: &Action) -> Result<bool, ProofError> {
        let result = match action {
            Action::CreateGeneratorZero => self.create_generator_zero(),
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
            Action::Homotopy(Homotopy::Expand(homotopy)) => self.homotopy_expand(homotopy)?,
            Action::Homotopy(Homotopy::Contract(homotopy)) => self.homotopy_contract(homotopy)?,
            Action::Squash => self.squash()?,
            Action::Behead => self.behead(),
            Action::Befoot => self.befoot(),
            Action::Invert => self.invert()?,
            Action::Restrict => self.restrict(),
            Action::Theorem => self.theorem()?,
            Action::SuspendSignature => self.suspend_signature(),
            Action::Suspend(s, t) => self.suspend(*s, *t),
            Action::Merge(from, to) => self.merge(*from, *to)?,
            Action::EditSignature(edit) => self.edit_signature(edit)?,
            Action::FlipBoundary => self.flip_boundary(),
            Action::RecoverBoundary => self.recover_boundary(),
            Action::Stash => self.stash_push(),
            Action::StashDrop => self.stash_drop(),
            Action::StashPop => self.stash_pop(),
            Action::StashApply => self.stash_apply(),
            Action::ImportProof(data) => self.import_proof(data)?,
            Action::EditMetadata(edit) => self.edit_metadata(edit),
            Action::Nothing => false,
        };
        Ok(result)
    }

    /// Determines if a given [Action] should reset the panzoom state, given the current  [ProofState].
    #[must_use]
    pub fn resets_panzoom(&self, action: &Action) -> bool {
        match *action {
            Action::EditSignature(SignatureEdit::Remove(node)) => self
                .workspace
                .as_ref()
                .is_some_and(|ws| self.signature.has_descendents_in(node, &ws.diagram)),
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
    fn create_generator_zero(&mut self) -> bool {
        self.signature.create_generator_zero("Cell");
        true
    }

    /// Handler for [Action::SetBoundary].
    ///
    /// Invalid if the workspace is empty.
    /// Returns an error if the diagrams are incompatible as boundaries.
    fn set_boundary(&mut self, boundary: Boundary) -> Result<bool, ProofError> {
        let Some(ws) = self.workspace.take() else {
            return Ok(false);
        };

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

        Ok(true)
    }

    /// Handler for [Action::TakeIdentityDiagram].
    ///
    /// Invalid if the workspace is empty.
    fn take_identity_diagram(&mut self) -> bool {
        let Some(ws) = &mut self.workspace else {
            return false;
        };

        if ws.diagram.dimension() + ws.path.len() >= 2 {
            ws.path.push_front(Boundary::Target.into());
        } else {
            ws.view.dimension += 1;
        }

        ws.diagram = ws.diagram.clone().identity().into();

        true
    }

    /// Handler for [Action::ClearWorkspace].
    ///
    /// Invalid if the workspace is empty.
    fn clear_workspace(&mut self) -> bool {
        self.workspace.take().is_some()
    }

    /// Handler for [Action::ClearBoundary].
    ///
    /// Invalid if the selected boundary is empty.
    fn clear_boundary(&mut self) -> bool {
        self.boundary.take().is_some()
    }

    /// Handler for [Action::SelectGenerator].
    ///
    /// Returns an error if the generator is not in the signature.
    fn select_generator(&mut self, generator: Generator) -> Result<bool, ProofError> {
        let info = self
            .signature
            .generator_info(generator)
            .ok_or(ProofError::UnknownGeneratorSelected)?;

        self.workspace = Some(Workspace::new(info.diagram.clone()));

        Ok(true)
    }

    /// Handler for [Action::AscendSlice].
    ///
    /// Invalid if the workspace is empty or the path is too short.
    fn ascend_slice(&mut self, count: usize) -> bool {
        let Some(ws) = &mut self.workspace else {
            return false;
        };

        if count == 0 || ws.path.is_empty() {
            return false;
        }

        for _ in 0..count {
            if ws.path.pop_back().is_none() {
                break;
            }

            if ws.view.dimension < 2 {
                ws.view.dimension += 1;
            }
        }

        true
    }

    /// Handler for [Action::DescendSlice].
    ///
    /// Invalid if the workspace is empty or has the wrong dimension.
    ///
    /// Returns an error if the slice is not a valid slice of the diagram.
    fn descend_slice(&mut self, slice: SliceIndex) -> Result<bool, ProofError> {
        let Some(ws) = &mut self.workspace else {
            return Ok(false);
        };

        let Diagram::DiagramN(diagram) = ws.visible_diagram() else {
            return Ok(false);
        };

        if let SliceIndex::Interior(height) = slice {
            if height > Height::Regular(diagram.size()) {
                return Err(ProofError::InvalidSlice);
            }
        }

        ws.path.push_back(slice);
        ws.view.dimension = ws.view.dimension.min(ws.visible_dimension() as u8);

        Ok(true)
    }

    /// Handler for [Action::SwitchSlice].
    ///
    /// Invalid if the workspace is empty, the path is empty, or we cannot step in the given direction.
    fn switch_slice(&mut self, direction: Direction) -> bool {
        let Some(ws) = &mut self.workspace else {
            return false;
        };

        let Some(slice) = ws.path.pop_back() else {
            return false;
        };

        let diagram = DiagramN::try_from(ws.visible_diagram()).unwrap();
        let next_slice = slice.step(diagram.size(), direction);
        ws.path.push_back(next_slice.unwrap_or(slice));
        next_slice.is_some()
    }

    /// Handler for [Action::IncreaseView].
    ///
    /// Invalid if the workspace is empty or the view dimension is too high.
    fn increase_view(&mut self, count: u8) -> bool {
        let Some(ws) = &mut self.workspace else {
            return false;
        };

        let max = std::cmp::min(ws.visible_dimension() as u8, View::MAX);

        if count == 0 || ws.view.dimension == max {
            return false;
        }

        ws.view.dimension = std::cmp::min(ws.view.dimension + count, max);

        true
    }

    /// Handler for [Action::DecreaseView].
    ///
    /// Invalid if the workspace is empty or the view dimension is too low.
    fn decrease_view(&mut self, count: u8) -> bool {
        let Some(ws) = &mut self.workspace else {
            return false;
        };

        if count == 0 || ws.view.dimension == 0 {
            return false;
        }

        ws.view.dimension = ws.view.dimension.saturating_sub(count);

        true
    }

    /// Handler for [Action::Attach].
    ///
    /// Invalid if the workspace is empty or has dimension 0 (if the boundary path is not null).
    fn attach(&mut self, option: &AttachOption) -> Result<bool, ProofError> {
        let Some(ws) = &mut self.workspace else {
            return Ok(false);
        };
        let diagram = &mut ws.diagram;

        let embedding: Vec<_> = option.embedding.iter().copied().collect();

        if let Some(bp) = &option.boundary_path {
            let Diagram::DiagramN(diagram) = diagram else {
                return Ok(false);
            };
            *diagram = diagram.attach(&option.diagram, bp.boundary(), &embedding)?;
        } else {
            *diagram = diagram
                .clone()
                .identity()
                .attach(&option.diagram, Boundary::Target, &embedding)?
                .target();
        }

        Ok(true)
    }

    /// Handler for [Action::Homotopy].
    ///
    /// Invalid if the workspace is empty or has dimension 0.
    fn homotopy_expand(&mut self, homotopy: &Expand) -> Result<bool, ProofError> {
        let Some(ws) = &mut self.workspace else {
            return Ok(false);
        };
        let diagram = &mut ws.diagram;

        let location = {
            let mut location: Vec<_> = ws.path.iter().copied().collect();
            location.extend(&homotopy.location);
            location
        };

        let (boundary_path, mut interior_path) = BoundaryPath::split(&location);

        if let Some(boundary_path) = boundary_path {
            let Diagram::DiagramN(diagram) = diagram else {
                return Ok(false);
            };
            *diagram = diagram.expand(
                boundary_path,
                &mut interior_path,
                homotopy.point,
                homotopy.direction,
                &self.signature,
            )?;
        } else {
            *diagram = diagram
                .clone()
                .identity()
                .expand(
                    Boundary::Target.into(),
                    &mut interior_path,
                    homotopy.point,
                    homotopy.direction,
                    &self.signature,
                )?
                .target();
        }

        let offset = boundary_path.map_or(0, |bp| bp.depth() + 1);
        for i in offset..ws.path.len() {
            ws.path[i] = SliceIndex::Interior(interior_path[i - offset]);
        }

        Ok(true)
    }

    /// Handler for [Action::Homotopy].
    ///
    /// Invalid if the workspace is empty or has dimension 0.
    fn homotopy_contract(&mut self, homotopy: &Contract) -> Result<bool, ProofError> {
        let Some(ws) = &mut self.workspace else {
            return Ok(false);
        };
        let diagram = &mut ws.diagram;

        let location = {
            let mut location: Vec<_> = ws.path.iter().copied().collect();
            location.extend(&homotopy.location);
            location
        };

        let (boundary_path, mut interior_path) = BoundaryPath::split(&location);

        if let Some(boundary_path) = boundary_path {
            let Diagram::DiagramN(diagram) = diagram else {
                return Ok(false);
            };
            *diagram = diagram.contract(
                boundary_path,
                &mut interior_path,
                homotopy.height,
                homotopy.direction,
                homotopy.step,
                homotopy.bias,
                &self.signature,
            )?;
        } else {
            *diagram = diagram
                .clone()
                .identity()
                .contract(
                    Boundary::Target.into(),
                    &mut interior_path,
                    homotopy.height,
                    homotopy.direction,
                    homotopy.step,
                    homotopy.bias,
                    &self.signature,
                )?
                .target();
        }

        let offset = boundary_path.map_or(0, |bp| bp.depth() + 1);
        for i in offset..ws.path.len() {
            ws.path[i] = SliceIndex::Interior(interior_path[i - offset]);
        }

        Ok(true)
    }

    /// Handler for [Action::Squash].
    ///
    /// Invalid if the workspace is empty or the visible diagram has dimension 0 or size 0.
    fn squash(&mut self) -> Result<bool, ProofError> {
        let Some(ws) = &mut self.workspace else {
            return Ok(false);
        };
        let Some(step) = ws
            .visible_diagram()
            .size()
            .and_then(|size| size.checked_sub(1))
        else {
            return Ok(false);
        };

        self.homotopy_contract(&Contract {
            height: 0,
            direction: Direction::Forward,
            step,
            bias: None,
            location: Default::default(),
        })
    }

    /// Handler for [Action::Behead].
    ///
    /// Invalid if the workspace is empty or has dimension 0, or if the path is invalid.
    fn behead(&mut self) -> bool {
        let Some(ws) = &mut self.workspace else {
            return false;
        };
        let Diagram::DiagramN(diagram) = &ws.diagram else {
            return false;
        };

        let max_height = match ws.path.len() {
            0 if diagram.size() > 0 => diagram.size() - 1,
            1 => match ws.path[0] {
                SliceIndex::Boundary(Boundary::Source) => 0,
                SliceIndex::Boundary(Boundary::Target) => diagram.size(),
                SliceIndex::Interior(Height::Regular(j)) => j,
                _ => return false,
            },
            _ => return false,
        };

        ws.diagram = diagram.behead(max_height).into();

        if ws.path.len() == 1 {
            ws.path.pop_back();
            if ws.view.dimension < 2 {
                ws.view.dimension += 1;
            }
        }

        true
    }

    /// Handler for [Action::Befoot].
    ///
    /// Invalid if the workspace is empty or has dimension 0, or if the path is invalid.
    fn befoot(&mut self) -> bool {
        let Some(ws) = &mut self.workspace else {
            return false;
        };
        let Diagram::DiagramN(diagram) = &ws.diagram else {
            return false;
        };

        let min_height = match ws.path.len() {
            0 if diagram.size() > 0 => 1,
            1 => match ws.path[0] {
                SliceIndex::Boundary(Boundary::Source) => 0,
                SliceIndex::Boundary(Boundary::Target) => diagram.size(),
                SliceIndex::Interior(Height::Regular(j)) => j,
                _ => return false,
            },
            _ => return false,
        };

        ws.diagram = diagram.befoot(min_height).into();

        if ws.path.len() == 1 {
            ws.path.pop_back();
            if ws.view.dimension < 2 {
                ws.view.dimension += 1;
            }
        }

        true
    }

    /// Handler for [Action::Invert].
    ///
    /// Invalid if the workspace is empty or has dimension 0.
    ///
    /// Returns an error if the diagram cannot be inverted (if not all generators are invertible).
    fn invert(&mut self) -> Result<bool, ProofError> {
        let Some(ws) = &mut self.workspace else {
            return Ok(false);
        };

        if !ws.path.is_empty() {
            return Ok(false);
        }

        if !ws.diagram.is_invertible(&self.signature) {
            return Err(ProofError::NotInvertible);
        }

        let Diagram::DiagramN(diagram) = &mut ws.diagram else {
            return Ok(false);
        };
        *diagram = diagram.inverse();

        Ok(true)
    }

    /// Handler for [Action::Restrict].
    ///
    /// Invalid if the workspace is empty, or if the path is empty or contains a singular slice.
    fn restrict(&mut self) -> bool {
        let Some(ws) = &mut self.workspace else {
            return false;
        };

        if ws.path.is_empty()
            || ws
                .path
                .iter()
                .any(|index| matches!(index, SliceIndex::Interior(Height::Singular(_))))
        {
            return false;
        }

        ws.diagram = ws.visible_diagram();
        ws.path = Default::default();

        true
    }

    /// Handler for [Action::Theorem].
    ///
    /// Invalid if the workspace is empty or has dimension 0.
    fn theorem(&mut self) -> Result<bool, ProofError> {
        let Some(ws) = self.workspace.take() else {
            return Ok(false);
        };

        let invertible = ws.diagram.is_invertible(&self.signature);
        let Diagram::DiagramN(diagram) = ws.diagram else {
            return Ok(false);
        };

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

        Ok(true)
    }

    /// Handler for [Action::SuspendSignature].
    fn suspend_signature(&mut self) -> bool {
        // New generators need to be fresh
        let id = self.signature.next_generator_id();
        let source = Generator::new(id, 0);
        let target = Generator::new(id + 1, 0);
        self.signature
            .insert(source, Diagram0::from(source), "Base Source", false);
        self.signature
            .insert(target, Diagram0::from(target), "Base Target", false);
        self.suspend(source, target)
    }

    /// Handler for [Action::Suspend].
    fn suspend(&mut self, source: Generator, target: Generator) -> bool {
        self.signature = self.signature.filter_map(|info| {
            if info.generator == source || info.generator == target {
                Some(info.clone())
            } else {
                Some(GeneratorInfo {
                    generator: info.generator.suspended(),
                    diagram: info.diagram.suspend(source, target).into(),
                    oriented: false,
                    ..info.clone()
                })
            }
        });

        if let Some(ws) = &mut self.workspace {
            ws.diagram = ws.diagram.suspend(source, target).into();
        }
        if let Some(bd) = &mut self.boundary {
            bd.diagram = bd.diagram.suspend(source, target).into();
        }
        for ws in self.stash.iter_mut() {
            ws.diagram = ws.diagram.suspend(source, target).into();
        }

        true
    }

    /// Handler for [Action::Merge].
    fn merge(&mut self, from: Generator, to: Generator) -> Result<bool, ProofError> {
        let info_from = self
            .signature
            .generator_info(from)
            .ok_or(ProofError::UnknownGeneratorSelected)?;
        let info_to = self
            .signature
            .generator_info(to)
            .ok_or(ProofError::UnknownGeneratorSelected)?;
        let invertible = info_from.invertible || info_to.invertible;
        let oriented = info_from.oriented || info_to.oriented;

        self.signature = self.signature.filter_map(|info| {
            if info.generator == from {
                None
            } else {
                let (oriented, invertible) = if info.generator == to {
                    (oriented, invertible)
                } else {
                    (info.oriented, info.invertible)
                };
                Some(GeneratorInfo {
                    diagram: info.diagram.replace(from, to, oriented),
                    oriented,
                    invertible,
                    ..info.clone()
                })
            }
        });

        if let Some(ws) = &mut self.workspace {
            ws.diagram = ws.diagram.replace(from, to, oriented);
        }
        if let Some(bd) = &mut self.boundary {
            bd.diagram = bd.diagram.replace(from, to, oriented);
        }
        for ws in self.stash.iter_mut() {
            ws.diagram = ws.diagram.replace(from, to, oriented);
        }

        Ok(true)
    }
    /// Handler for [Action::ImportProof].
    fn import_proof(&mut self, data: &SerializedData) -> Result<bool, ProofError> {
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
        self.boundary = None;
        self.stash = Vector::new();
        Ok(true)
    }

    /// Handler for [Action::EditSignature].
    fn edit_signature(&mut self, edit: &SignatureEdit) -> Result<bool, ProofError> {
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

            // remove from stashed workspaces
            self.stash
                .retain(|ws| !self.signature.has_descendents_in(*node, &ws.diagram));
        }

        if let SignatureEdit::Edit(node, SignatureItemEdit::MakeOriented(true)) = edit {
            if let Some(generator) = self.signature.find_generator(*node) {
                // remove framing from the workspace
                if let Some(ws) = &mut self.workspace {
                    ws.diagram = ws.diagram.remove_framing(generator);
                }

                // remove framing from the boundary
                if let Some(selected) = &mut self.boundary {
                    selected.diagram = selected.diagram.remove_framing(generator);
                }

                // remove framing from stashed workspaces
                for ws in self.stash.iter_mut() {
                    ws.diagram = ws.diagram.remove_framing(generator);
                }
            } else {
                return Ok(false);
            }
        }

        if let SignatureEdit::Edit(node, SignatureItemEdit::MakeInvertible(false)) = edit {
            if let Some(generator) = self.signature.find_generator(*node) {
                if let Some(ws) = &self.workspace {
                    if ws
                        .diagram
                        .generators()
                        .get(&generator)
                        .is_some_and(|os| os.contains(&Orientation::Negative))
                    {
                        return Err(ProofError::SignatureError(
                            SignatureError::CannotBeMadeDirected,
                        ));
                    }
                }

                if let Some(selected) = &self.boundary {
                    if selected
                        .diagram
                        .generators()
                        .get(&generator)
                        .is_some_and(|os| os.contains(&Orientation::Negative))
                    {
                        return Err(ProofError::SignatureError(
                            SignatureError::CannotBeMadeDirected,
                        ));
                    }
                }

                if self.stash.iter().any(|ws| {
                    ws.diagram
                        .generators()
                        .get(&generator)
                        .is_some_and(|os| os.contains(&Orientation::Negative))
                }) {
                    return Err(ProofError::SignatureError(
                        SignatureError::CannotBeMadeDirected,
                    ));
                }
            } else {
                return Ok(false);
            }
        }

        self.signature.update(edit)?;

        Ok(true)
    }

    /// Handler for [Action::EditMetadata].
    fn edit_metadata(&mut self, edit: &MetadataEdit) -> bool {
        match edit {
            MetadataEdit::Title(title) => self.metadata.title = Some(title.clone()),
            MetadataEdit::Author(author) => self.metadata.author = Some(author.clone()),
            MetadataEdit::Abstract(abstr) => self.metadata.abstr = Some(abstr.clone()),
        }

        true
    }

    /// Handler for [Action::FlipBoundary].
    ///
    /// Invalid if the selected boundary is empty.
    fn flip_boundary(&mut self) -> bool {
        let Some(selected) = &mut self.boundary else {
            return false;
        };
        selected.boundary = selected.boundary.flip();
        true
    }

    /// Handler for [Action::RecoverBoundary].
    ///
    /// Invalid if the selected boundary is empty.
    fn recover_boundary(&mut self) -> bool {
        let Some(selected) = self.boundary.as_ref() else {
            return false;
        };
        self.workspace = Some(Workspace::new(selected.diagram.clone()));
        true
    }

    /// Handler for [Action::Stash].
    ///
    /// Invalid if the workspace is empty.
    fn stash_push(&mut self) -> bool {
        let Some(ws) = self.workspace.take() else {
            return false;
        };
        self.stash.push_front(ws);
        true
    }

    /// Handler for [Action::StashDrop].
    ///
    /// Invalid if the stash is empty.
    fn stash_drop(&mut self) -> bool {
        self.stash.pop_front().is_some()
    }

    /// Handler for [Action::StashPop].
    ///
    /// Invalid if the stash is empty.
    fn stash_pop(&mut self) -> bool {
        let Some(stashed) = self.stash.pop_front() else {
            return false;
        };
        self.workspace = Some(stashed);
        true
    }

    /// Handler for [Action::StashApply].
    ///
    /// Invalid if the stash is empty.
    fn stash_apply(&mut self) -> bool {
        let Some(stashed) = self.stash.front() else {
            return false;
        };
        self.workspace = Some(stashed.clone());
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttachOption {
    pub generator: Generator,
    pub boundary_path: Option<BoundaryPath>,
    pub embedding: Vector<usize>,
    pub tag: Option<String>,
    pub diagram: DiagramN,
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
