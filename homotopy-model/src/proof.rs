use std::convert::{Into, TryFrom};

use homotopy::Homotopy;
use homotopy_common::hash::FastHashMap;
use homotopy_core::{
    common::{Boundary, BoundaryPath, Direction, Generator, Height, Mode, SliceIndex},
    contraction::ContractionError,
    diagram::{AttachmentError, NewDiagramError},
    expansion::ExpansionError,
    signature::Signature as S,
    Diagram, Diagram0, DiagramN,
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

    pub fn dimension(self) -> u8 {
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
    pub fn new(diagram: Diagram) -> Self {
        // Default to 2D unless the diagram has dimension 0 or 1.
        let dimension = diagram.dimension().min(2) as u8;
        Self {
            view: View { dimension },
            diagram,
            path: Default::default(),
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

    Behead,

    Befoot,

    Invert,

    Restrict,

    Theorem,

    Suspend(bool),

    Abelianize,

    ImportProof(SerializedData),

    EditSignature(SignatureEdit),

    EditMetadata(MetadataEdit),

    FlipBoundary,

    RecoverBoundary,

    Nothing,
}

impl Action {
    /// Determines if a given [Action] is valid given the current [ProofState].
    ///
    /// This should return true iff performing the action does *not* return false.
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
            Self::AscendSlice(count) => {
                *count > 0
                    && proof
                        .workspace
                        .as_ref()
                        .map_or(false, |ws| !ws.path.is_empty())
            }
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
                        ws.view.dimension < std::cmp::min(ws.visible_dimension() as u8, View::MAX)
                    })
            }
            Self::DecreaseView(count) => {
                *count > 0
                    && proof
                        .workspace
                        .as_ref()
                        .map_or(false, |ws| ws.view.dimension > 0)
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
                .map_or(false, |ws| ws.path.is_empty() && ws.diagram.dimension() > 0),
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
            Self::Suspend(_) | Self::Abelianize => true,
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
            Action::Behead => self.behead(),
            Action::Befoot => self.befoot(),
            Action::Invert => self.invert()?,
            Action::Restrict => self.restrict(),
            Action::Theorem => self.theorem()?,
            Action::Suspend(l) => self.suspend(*l),
            Action::Abelianize => self.abelianize(),
            Action::EditSignature(edit) => self.edit_signature(edit),
            Action::FlipBoundary => self.flip_boundary(),
            Action::RecoverBoundary => self.recover_boundary(),
            Action::ImportProof(data) => self.import_proof(data)?,
            Action::EditMetadata(edit) => self.edit_metadata(edit),
            Action::Nothing => false,
        };
        Ok(result)
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
    fn create_generator_zero(&mut self) -> bool {
        self.signature.create_generator_zero("Cell");
        true
    }

    /// Handler for [Action::SetBoundary].
    ///
    /// Invalid if the workspace is empty.
    /// Returns an error if the diagrams are incompatible as boundaries.
    fn set_boundary(&mut self, boundary: Boundary) -> Result<bool, ProofError> {
        let Some(ws) = self.workspace.take() else { return Ok(false) };

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
        let Some(ws) = &mut self.workspace else { return false };

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
        let Some(ws) = &mut self.workspace else { return false };

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
        let Some(ws) = &mut self.workspace else { return Ok(false) };

        let Diagram::DiagramN(diagram) = ws.visible_diagram() else { return Ok(false) };

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
        let Some(ws) = &mut self.workspace else { return false };

        let Some(slice) = ws.path.pop_back() else { return false };

        let diagram = DiagramN::try_from(ws.visible_diagram()).unwrap();
        let next_slice = slice.step(diagram.size(), direction);
        ws.path.push_back(next_slice.unwrap_or(slice));
        next_slice.is_some()
    }

    /// Handler for [Action::IncreaseView].
    ///
    /// Invalid if the workspace is empty or the view dimension is too high.
    fn increase_view(&mut self, count: u8) -> bool {
        let Some(ws) = &mut self.workspace else { return false };

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
        let Some(ws) = &mut self.workspace else { return false };

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
        let Some(ws) = &mut self.workspace else { return Ok(false) };
        let diagram = &mut ws.diagram;

        let embedding: Vec<_> = option.embedding.iter().copied().collect();

        if let Some(bp) = &option.boundary_path {
            let Diagram::DiagramN(diagram) = diagram else { return Ok(false) };
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
        let Some(ws) = &mut self.workspace else { return Ok(false) };
        let diagram = &mut ws.diagram;

        let location = {
            let mut location: Vec<_> = ws.path.iter().copied().collect();
            location.extend(homotopy.location.clone());
            location
        };

        let (boundary_path, mut interior_path) = BoundaryPath::split(&location);

        if let Some(boundary_path) = boundary_path {
            let Diagram::DiagramN(diagram) = diagram else { return Ok(false) };
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
        let Some(ws) = &mut self.workspace else { return Ok(false) };
        let diagram = &mut ws.diagram;

        let location = {
            let mut location: Vec<_> = ws.path.iter().copied().collect();
            location.extend(homotopy.location.clone());
            location
        };

        let (boundary_path, mut interior_path) = BoundaryPath::split(&location);

        if let Some(boundary_path) = boundary_path {
            let Diagram::DiagramN(diagram) = diagram else { return Ok(false) };
            *diagram = diagram.contract(
                boundary_path,
                &mut interior_path,
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
                    &mut interior_path,
                    homotopy.height,
                    homotopy.direction,
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

    /// Handler for [Action::Behead].
    ///
    /// Invalid if the workspace is empty or has dimension 0, or if the path is invalid.
    fn behead(&mut self) -> bool {
        let Some(ws) = &mut self.workspace else { return false };
        let Diagram::DiagramN(diagram) = &ws.diagram else { return false };

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
        let Some(ws) = &mut self.workspace else { return false };
        let Diagram::DiagramN(diagram) = &ws.diagram else { return false };

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
        let Some(ws) = &mut self.workspace else { return Ok(false) };

        if !ws.path.is_empty() {
            return Ok(false);
        }

        if !ws.diagram.is_invertible(&self.signature) {
            return Err(ProofError::NotInvertible);
        }

        let Diagram::DiagramN(diagram) = &mut ws.diagram else { return Ok(false) };
        *diagram = diagram.inverse();

        Ok(true)
    }

    /// Handler for [Action::Restrict].
    ///
    /// Invalid if the workspace is empty, or if the path is empty or contains a singular slice.
    fn restrict(&mut self) -> bool {
        let Some(ws) = &mut self.workspace else { return false };

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
        let Some(ws) = self.workspace.take() else { return Ok(false) };

        let invertible = ws.diagram.is_invertible(&self.signature);
        let Diagram::DiagramN(diagram) = ws.diagram else { return Ok(false) };

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

    /// Handler for [Action::Suspend].
    fn suspend(&mut self, loop_mode: bool) -> bool {
        use homotopy_common::tree::Node;

        let mut new_signature: Signature = Default::default();

        // New generators need to be fresh
        let id = self.signature.next_generator_id();
        let source = Generator::new(id, 0);
        let target = if loop_mode {
            new_signature.insert(source, Diagram0::from(source), "Base", false);
            source
        } else {
            new_signature.insert(source, Diagram0::from(source), "Base Source", false);
            let target = Generator::new(id + 1, 0);
            new_signature.insert(target, Diagram0::from(target), "Base Target", false);
            target
        };

        // We are shifting nodes in the tree
        // so we have to remap the parents correctly
        let mut node_mappings: FastHashMap<Node, Node> = Default::default();
        node_mappings.insert(
            self.signature.as_tree().root(),
            new_signature.as_tree().root(),
        );

        // Skip the root node
        for (node, data) in self.signature.as_tree().iter().skip(1) {
            let mapped_node = if let Some(parent) = data.parent() {
                node_mappings[&parent]
            } else {
                new_signature.as_tree().root()
            };
            match data.inner() {
                SignatureItem::Folder(_) => {
                    let new_node = new_signature
                        .push_onto(mapped_node, data.inner().clone())
                        .unwrap();
                    node_mappings.insert(node, new_node);
                }
                SignatureItem::Item(g) => {
                    let gen: GeneratorInfo = GeneratorInfo {
                        generator: g.generator.suspended(),
                        diagram: g.diagram.suspend(source, target),
                        //TODO remove when label logic is implemented
                        oriented: true,
                        ..g.clone()
                    };
                    new_signature.push_onto(mapped_node, SignatureItem::Item(gen));
                }
            }
        }
        self.signature = new_signature;
        if let Some(ws) = &self.workspace {
            self.workspace = Some(Workspace::new(ws.diagram.suspend(source, target)));
        }
        if let Some(bd) = &mut self.boundary {
            bd.diagram = bd.diagram.suspend(source, target);
        }

        true
    }

    /// Handler for [Action::Abelianize].
    fn abelianize(&mut self) -> bool {
        let generators: Vec<Generator> = self
            .signature
            .as_tree()
            .iter()
            .filter_map(|(_, d)| {
                if let SignatureItem::Item(g) = d.inner() {
                    if g.generator.dimension == 0 {
                        Some(g.generator)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .take(2)
            .collect();

        // If signature is empty quit
        // If there is not a unique 0-cell fall back to suspension
        if generators.len() == 0 {
            return false;
        }
        if generators.len() > 1 {
            return self.suspend(true);
        }

        let base = generators[0];

        let mut new_signature: Signature = Default::default();

        // Skip the root node
        for (_node, data) in self.signature.as_tree().iter().skip(1) {
            match (data.parent(), data.inner()) {
                (Some(p), SignatureItem::Folder(_)) => {
                    new_signature.push_onto(p, data.inner().clone()).unwrap();
                }
                (Some(p), SignatureItem::Item(g)) if g.generator == base => {
                    new_signature.push_onto(p, SignatureItem::Item(g.clone()));
                }
                (Some(p), SignatureItem::Item(g)) => {
                    let gen: GeneratorInfo = GeneratorInfo {
                        generator: g.generator.suspended(),
                        diagram: g.diagram.abelianize(base),
                        //TODO remove when label logic is implemented
                        oriented: true,
                        ..g.clone()
                    };
                    new_signature.push_onto(p, SignatureItem::Item(gen));
                }
                (None, _) => {}
            }
        }
        self.signature = new_signature;
        if let Some(ws) = &self.workspace {
            self.workspace = Some(Workspace::new(ws.diagram.abelianize(base)));
        }
        if let Some(bd) = &mut self.boundary {
            bd.diagram = bd.diagram.abelianize(base);
        }

        true
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
        Ok(true)
    }

    /// Handler for [Action::EditSignature].
    fn edit_signature(&mut self, edit: &SignatureEdit) -> bool {
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
            if let Some(generator) = self.signature.find_generator(*node) {
                // remove framing from the workspace
                if let Some(ws) = &mut self.workspace {
                    ws.diagram = ws.diagram.remove_framing(generator);
                }

                // remove framing from the boundary
                if let Some(selected) = &mut self.boundary {
                    selected.diagram = selected.diagram.remove_framing(generator);
                }
            } else {
                return false;
            }
        }

        self.signature.update(edit);

        true
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
        let Some(selected) = &mut self.boundary else { return false };
        selected.boundary = selected.boundary.flip();
        true
    }

    /// Handler for [Action::RecoverBoundary].
    ///
    /// Invalid if the selected boundary is empty.
    fn recover_boundary(&mut self) -> bool {
        let Some(selected) = self.boundary.as_ref() else { return false };
        self.workspace = Some(Workspace::new(selected.diagram.clone()));
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
