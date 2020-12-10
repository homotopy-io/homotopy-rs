use homotopy_core::attach::BoundaryPath;
use homotopy_core::common::*;
use homotopy_core::complex::Simplex;
use homotopy_core::diagram::NewDiagramError;
use homotopy_core::{Diagram, DiagramN};
use im::{HashMap, Vector};
use std::convert::*;
use thiserror::Error;

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
        let x = Generator {
            id: 0,
            dimension: 0,
        };
        let f = Generator {
            id: 1,
            dimension: 1,
        };
        let m = Generator {
            id: 2,
            dimension: 2,
        };
        let a = Generator {
            id: 3,
            dimension: 3,
        };

        let fd = DiagramN::new(f, x, x).unwrap();
        let ffd = fd.attach(fd.clone(), Boundary::Target, &[]).unwrap();
        let md = DiagramN::new(m, ffd, fd.clone()).unwrap();
        let left = md.attach(md.clone(), Boundary::Source, &[0]).unwrap();
        let right = md.attach(md.clone(), Boundary::Source, &[1]).unwrap();
        let ad = DiagramN::new(a, left, right).unwrap();

        // for _ in 0..1 {
        //     result = result.attach(md.clone(), Boundary::Source, &[0]).unwrap();
        // }

        let mut signature: HashMap<Generator, GeneratorInfo> = Default::default();

        signature.insert(
            x,
            GeneratorInfo {
                name: "x".to_string(),
                color: COLORS[0].to_owned(),
                diagram: x.into(),
            },
        );

        signature.insert(
            f,
            GeneratorInfo {
                name: "f".to_string(),
                color: COLORS[1].to_owned(),
                diagram: fd.into(),
            },
        );

        signature.insert(
            m,
            GeneratorInfo {
                name: "m".to_string(),
                color: COLORS[2].to_owned(),
                diagram: md.into(),
            },
        );

        signature.insert(
            a,
            GeneratorInfo {
                name: "a".to_string(),
                color: COLORS[3].to_owned(),
                diagram: ad.clone().into(),
            },
        );

        log::info!("signature size: {}", signature.len());

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
}

impl State {
    /// Update the state in response to an [Action].
    pub fn update(&mut self, action: Action) -> Result<(), ModelError> {
        match action {
            Action::CreateGeneratorZero => Ok(self.create_generator_zero()),
            Action::RemoveGenerator(_) => unimplemented!(),
            Action::SetBoundary(boundary) => self.set_boundary(boundary),
            Action::TakeIdentityDiagram => Ok(self.take_identity_diagram()),
            Action::ClearWorkspace => Ok(self.clear_workspace()),
            Action::ClearBoundary => Ok(self.clear_boundary()),
            Action::SelectGenerator(generator) => self.select_generator(generator),
            Action::AscendSlice(count) => Ok(self.ascend_slice(count)),
            Action::DescendSlice(slice) => self.descend_slice(slice),
            Action::SelectPoints(points) => self.select_points(points),
            Action::ToggleDrawer(drawer) => Ok(self.toggle_drawer(drawer)),
        }
    }

    /// Handler for [Action::CreateGeneratorZero].
    fn create_generator_zero(&mut self) {
        let id = self.create_generator_id();
        let generator = Generator::new(id, 0);

        let info = GeneratorInfo {
            name: format!("Cell {}", id),
            color: COLORS[id % COLORS.len()].to_owned(),
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
                        Source => (workspace.diagram.clone().into(), selected.diagram.clone()),
                        Target => (selected.diagram.clone(), workspace.diagram.clone().into()),
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
                // TODO: Figure out what to do with the path
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
        });

        Ok(())
    }

    /// Handler for [Action::AscendSlice].
    fn ascend_slice(&mut self, mut count: usize) {
        if let Some(workspace) = &mut self.workspace {
            while count > 0 && workspace.path.len() > 0 {
                workspace.path.pop_back();
                count -= 1;
            }
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
                    .map_err(|_| ModelError::InvalidSlice)?
                    .slice(*height)
                    .ok_or(ModelError::InvalidSlice)?;
            }

            // Update workspace
            workspace.path = path;
        }

        Ok(())
    }

    /// Handler for [Action::SelectPoint].
    fn select_points(&mut self, selected: Vec<Vec<SliceIndex>>) -> Result<(), ModelError> {
        if selected.len() == 0 {
            return Ok(());
        }

        let workspace = match &self.workspace {
            Some(workspace) => workspace,
            None => {
                return Ok(());
            }
        };

        fn path_depth(boundary_path: &Option<BoundaryPath>) -> usize {
            match boundary_path {
                Some(boundary_path) => boundary_path.depth() + 1,
                None => 0,
            }
        }

        let (boundary_path, point) = {
            // TODO: It must be possible to do this more cleanly.
            let candidates: Vec<_> = selected
                .into_iter()
                .map(|p| {
                    let mut point: Vec<_> = workspace.path.iter().cloned().collect();
                    point.extend(p);
                    BoundaryPath::split(&point)
                })
                .collect();

            let max_depth = candidates
                .iter()
                .map(|(bp, _)| path_depth(bp))
                .max()
                .unwrap();
            candidates
                .into_iter()
                .find(|(bp, _)| path_depth(bp) == max_depth)
                .unwrap()
        };

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

        log::info!("position: {:?}", (&boundary_path, &point));

        let matches: Vec<AttachOption> = {
            let mut matches = Vec::new();

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

            matches
        };

        log::info!("matches: {:#?}", matches);

        if matches.len() == 1 {
            self.attach(matches.into_iter().next().unwrap());
            Ok(())
        } else {
            Ok(())
        }
    }

    fn attach(&mut self, option: AttachOption) {
        if let Some(workspace) = &mut self.workspace {
            // TODO: Better error handling, although none of these errors should occur
            let diagram = DiagramN::try_from(workspace.diagram.clone()).unwrap();
            let generator = DiagramN::try_from(
                self.signature
                    .get(&option.generator)
                    .unwrap()
                    .diagram
                    .clone(),
            )
            .unwrap();
            let boundary: Boundary = option
                .boundary_path
                .clone()
                .map(|bp| bp.boundary())
                .unwrap_or(Boundary::Target);
            let embedding: Vec<_> = option.embedding.iter().cloned().collect();

            let result = diagram.attach(generator, boundary, &embedding).unwrap();

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

    /// Handler for [Action::ToggleDrawer].
    fn toggle_drawer(&mut self, drawer: Drawer) {
        if self.drawer == Some(drawer) {
            self.drawer = None;
        } else {
            self.drawer = Some(drawer);
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachOption {
    generator: Generator,
    boundary_path: Option<BoundaryPath>,
    embedding: Vector<usize>,
}

const COLORS: &[&'static str] = &[
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
    assert!(embedding.len() <= point.len());

    if diagram.dimension() != point.len() {
        return false;
    }

    match diagram {
        Diagram0(_) => true,
        DiagramN(diagram) => {
            let (height, point) = point.split_first().unwrap();
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
