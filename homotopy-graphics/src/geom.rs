use std::{
    cmp::Ordering,
    ops::{Deref, DerefMut},
};

use homotopy_common::{declare_idx, idx::IdxVec};
use homotopy_core::{
    common::DimensionError, layout::Layout, mesh::Mesh, DiagramN, Direction, Generator, Height,
    SliceIndex,
};
use ultraviolet::Vec4;

// Geometry

declare_idx! {
    pub struct Vert = usize;
    pub struct Curve = usize;
    pub struct Element = usize;
}

#[derive(Clone, Debug)]
pub struct Geometry<ElementData> {
    pub verts: IdxVec<Vert, VertData>,
    pub curves: IdxVec<Curve, CurveData>,
    pub elements: IdxVec<Element, ElementData>,
}

impl<ElementData> Geometry<ElementData> {
    pub fn mk_vert(&mut self, vert: VertData) -> Vert {
        self.verts.push(vert)
    }

    pub fn mk_element(&mut self, element: ElementData) -> Element {
        self.elements.push(element)
    }

    pub fn bounds(&self) -> (Vec4, Vec4) {
        self.verts.values().fold(
            (
                Vec4::broadcast(f32::INFINITY),
                Vec4::broadcast(f32::NEG_INFINITY),
            ),
            |(min, max), vert| (min.min_by_component(**vert), max.max_by_component(**vert)),
        )
    }
}

// Vert data

#[derive(Clone, Debug)]
pub struct VertData {
    pub vert: Vec4,
    pub stratum: isize,
    pub boundary: Boundary,
    pub generator: Generator,
}

impl Deref for VertData {
    type Target = Vec4;

    fn deref(&self) -> &Self::Target {
        &self.vert
    }
}

impl DerefMut for VertData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vert
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Boundary {
    /// Corner - no freedom to move
    Zero = 0,
    /// Edge - free to move along line
    One = 1,
    /// Surface - free to move in space
    Two = 2,
    /// Volume - free to move in time and space
    Three = 3,
}

impl From<usize> for Boundary {
    fn from(boundary: usize) -> Self {
        match boundary {
            0 => Self::Zero,
            1 => Self::One,
            2 => Self::Two,
            _ => Self::Three,
        }
    }
}

pub fn calculate_stratum(path: &[SliceIndex]) -> isize {
    path.iter()
        .map(|&index| match index {
            SliceIndex::Boundary(_) => -1,
            SliceIndex::Interior(Height::Regular(_)) => 0,
            SliceIndex::Interior(Height::Singular(_)) => 1,
        })
        .sum()
}

pub fn calculate_boundary(path: &[SliceIndex]) -> Boundary {
    path.iter()
        .take(path.len().saturating_sub(1))
        .map(|index| match index {
            SliceIndex::Boundary(_) => 0,
            SliceIndex::Interior(_) => 1,
        })
        .sum::<usize>()
        .into()
}

// Curve data

#[derive(Clone, Debug)]
pub struct CurveData {
    pub verts: Vec<Vert>,
    pub generator: Generator,
}

impl Deref for CurveData {
    type Target = Vec<Vert>;

    fn deref(&self) -> &Self::Target {
        &self.verts
    }
}

impl DerefMut for CurveData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.verts
    }
}

// Element data

#[derive(Clone, Debug)]
pub enum Cube {
    Point(Vert),
    Line([Vert; 2]),
    Square([Vert; 4]),
    Cube([Vert; 8]),
}

#[derive(Clone, Debug)]
pub enum Simplex {
    Point(Vert),
    Line([Vert; 2]),
    Tri([Vert; 3]),
    Tetra([Vert; 4]),
}

pub type CubicalGeometry = Geometry<Cube>;
pub type SimplicialGeometry = Geometry<Simplex>;

impl CubicalGeometry {
    pub fn new(diagram: &DiagramN, depth: usize) -> Result<Self, DimensionError> {
        if depth > diagram.dimension() {
            return Err(DimensionError);
        }

        // Extract the mesh and layout.
        let mesh = Mesh::new(diagram, depth)?;
        let layout = Layout::new(diagram, depth)?;

        let mut geom = Self {
            verts: IdxVec::new(),
            curves: IdxVec::new(),
            elements: IdxVec::new(),
        };
        let mut node_to_vert = IdxVec::with_capacity(mesh.graph.node_count());

        for (path, diagram) in mesh.graph.node_weights() {
            let vert = layout.get(path);
            let vert = [0, 1, 2, 3]
                .map(|i| vert.get(i).copied().unwrap_or_default())
                .into();

            node_to_vert.push(geom.mk_vert(VertData {
                vert,
                stratum: calculate_stratum(path),
                boundary: calculate_boundary(path),
                generator: diagram.max_generator(),
            }));
        }

        // TOOD(@calintat): Inline `flatten_elements`.
        for element in mesh.flatten_elements() {
            // integer logarithm in base 2
            let n = (usize::BITS - element.len().leading_zeros()) as usize;

            if n >= depth {
                continue;
            }

            let verts = element
                .into_iter()
                .map(|n| node_to_vert[n])
                .collect::<Vec<_>>();
            let generator = verts
                .iter()
                .map(|v| geom.verts[*v].generator)
                .min_by_key(|g| g.dimension)
                .unwrap();

            if diagram.dimension().saturating_sub(generator.dimension) != n {
                continue;
            }

            match verts.len() {
                1 => {
                    geom.mk_point(verts[0]);
                }
                2 => {
                    let verts = verts.try_into().unwrap();
                    geom.mk_line(verts);

                    // Curve extraction.
                    let curve = geom.curves.values_mut().find(|curve| {
                        let &curve_target = curve.last().unwrap();
                        curve_target == verts[0]
                    });
                    if let Some(curve) = curve {
                        curve.push(verts[1]);
                    } else {
                        geom.curves.push(CurveData {
                            generator,
                            verts: verts.to_vec(),
                        });
                    }
                }
                4 => {
                    geom.mk_square(verts.try_into().unwrap());
                }
                8 => {
                    geom.mk_cube(verts.try_into().unwrap());
                }
                _ => (),
            }
        }

        let (min, max) = geom.bounds();
        let translation = 0.5 * (max + min);
        let duration = 0.5 * (max.w - min.w);

        for vert in geom.verts.values_mut() {
            **vert -= translation;
            vert.w /= duration;
        }

        Ok(geom)
    }

    pub fn points(&self) -> impl Iterator<Item = Vert> + '_ {
        self.elements.values().filter_map(|cube| {
            if let Cube::Point(vert) = cube {
                Some(*vert)
            } else {
                None
            }
        })
    }

    pub fn lines(&self) -> impl Iterator<Item = [Vert; 2]> + '_ {
        self.elements.values().filter_map(|cube| {
            if let Cube::Line(verts) = cube {
                Some(*verts)
            } else {
                None
            }
        })
    }

    pub fn squares(&self) -> impl Iterator<Item = [Vert; 4]> + '_ {
        self.elements.values().filter_map(|cube| {
            if let Cube::Square(verts) = cube {
                Some(*verts)
            } else {
                None
            }
        })
    }

    pub fn cubes(&self) -> impl Iterator<Item = [Vert; 8]> + '_ {
        self.elements.values().filter_map(|cube| {
            if let Cube::Cube(verts) = cube {
                Some(*verts)
            } else {
                None
            }
        })
    }

    pub fn mk_point(&mut self, vert: Vert) -> Element {
        self.mk_element(Cube::Point(vert))
    }

    pub fn mk_line(&mut self, verts: [Vert; 2]) -> Element {
        self.mk_element(Cube::Line(verts))
    }

    pub fn mk_square(&mut self, verts: [Vert; 4]) -> Element {
        self.mk_element(Cube::Square(verts))
    }

    pub fn mk_cube(&mut self, verts: [Vert; 8]) -> Element {
        self.mk_element(Cube::Cube(verts))
    }
}

impl SimplicialGeometry {
    pub fn points(&self) -> impl Iterator<Item = Vert> + '_ {
        self.elements.values().filter_map(|simplex| {
            if let Simplex::Point(vert) = simplex {
                Some(*vert)
            } else {
                None
            }
        })
    }

    pub fn lines(&self) -> impl Iterator<Item = [Vert; 2]> + '_ {
        self.elements.values().filter_map(|simplex| {
            if let Simplex::Line(verts) = simplex {
                Some(*verts)
            } else {
                None
            }
        })
    }

    pub fn tris(&self) -> impl Iterator<Item = [Vert; 3]> + '_ {
        self.elements.values().filter_map(|simplex| {
            if let Simplex::Tri(verts) = simplex {
                Some(*verts)
            } else {
                None
            }
        })
    }

    pub fn tetras(&self) -> impl Iterator<Item = [Vert; 4]> + '_ {
        self.elements.values().filter_map(|simplex| {
            if let Simplex::Tetra(verts) = simplex {
                Some(*verts)
            } else {
                None
            }
        })
    }

    pub fn mk_point(&mut self, vert: Vert) -> Element {
        self.mk_element(Simplex::Point(vert))
    }

    pub fn mk_line(&mut self, verts: [Vert; 2]) -> Element {
        self.mk_element(Simplex::Line(verts))
    }

    pub fn mk_tri(&mut self, verts: [Vert; 3]) -> Element {
        self.mk_element(Simplex::Tri(verts))
    }

    pub fn mk_tetra(&mut self, verts: [Vert; 4]) -> Element {
        self.mk_element(Simplex::Tetra(verts))
    }
}

// Triangulation

const TRI_ASSEMBLY_ORDER: [[usize; 3]; 2] = [[0, 1, 3], [0, 3, 2]];

const TETRA_ASSEMBLY_ORDER: [[usize; 4]; 6] = [
    [0, 3, 1, 7],
    [0, 1, 5, 7],
    [0, 2, 3, 7],
    [0, 2, 7, 6],
    [0, 5, 4, 7],
    [0, 4, 6, 7],
];

impl Cube {
    fn triangulate(&self, geom: &CubicalGeometry) -> Vec<Simplex> {
        match *self {
            Cube::Point(vert) => {
                vec![Simplex::Point(vert)]
            }
            Cube::Line(verts) => {
                vec![Simplex::Line(verts)]
            }
            Cube::Square(verts) => {
                // Rotate the square
                let rotation = match geom.orientation_of_square(verts) {
                    [Direction::Forward, Direction::Forward] => [0, 1, 2, 3],
                    [Direction::Forward, Direction::Backward] => [1, 3, 0, 2],
                    [Direction::Backward, Direction::Forward] => [2, 0, 3, 1],
                    [Direction::Backward, Direction::Backward] => [3, 2, 1, 0],
                };
                let verts = rotation.map(|i| verts[i]);

                TRI_ASSEMBLY_ORDER
                    .into_iter()
                    .filter_map(|[i, j, k]| {
                        let tri @ [a, b, c] = [verts[i], verts[j], verts[k]];
                        (a != b && a != c && b != c).then(|| Simplex::Tri(tri))
                    })
                    .collect()
            }
            Cube::Cube(verts) => {
                // Rotate the cube
                let rotation = match geom.orientation_of_cube(verts) {
                    [Direction::Forward, Direction::Forward, Direction::Forward] => {
                        [0, 1, 2, 3, 4, 5, 6, 7]
                    }
                    [Direction::Forward, Direction::Forward, Direction::Backward] => {
                        [1, 0, 5, 4, 3, 2, 7, 6]
                    }
                    [Direction::Forward, Direction::Backward, Direction::Forward] => {
                        [2, 0, 3, 1, 6, 4, 7, 5]
                    }
                    [Direction::Forward, Direction::Backward, Direction::Backward] => {
                        [3, 1, 7, 5, 2, 0, 6, 4]
                    }
                    [Direction::Backward, Direction::Forward, Direction::Forward] => {
                        [4, 0, 6, 2, 5, 1, 7, 3]
                    }
                    [Direction::Backward, Direction::Forward, Direction::Backward] => {
                        [5, 1, 4, 0, 7, 3, 6, 2]
                    }
                    [Direction::Backward, Direction::Backward, Direction::Forward] => {
                        [6, 2, 7, 3, 4, 0, 5, 1]
                    }
                    [Direction::Backward, Direction::Backward, Direction::Backward] => {
                        [7, 3, 5, 1, 6, 2, 4, 0]
                    }
                };
                let verts = rotation.map(|i| verts[i]);

                TETRA_ASSEMBLY_ORDER
                    .into_iter()
                    .filter_map(|[i, j, k, l]| {
                        let tetra @ [a, b, c, d] = [verts[i], verts[j], verts[k], verts[l]];
                        (a != b && a != c && b != c && b != d && c != d)
                            .then(|| Simplex::Tetra(tetra))
                    })
                    .collect()
            }
        }
    }
}

impl From<CubicalGeometry> for SimplicialGeometry {
    fn from(cubical: CubicalGeometry) -> Self {
        Self {
            verts: cubical.verts.clone(),
            curves: cubical.curves.clone(),
            elements: cubical
                .elements
                .values()
                .flat_map(|cube| cube.triangulate(&cubical))
                .collect(),
        }
    }
}

// Orientation

const SQUARE_EDGE_ORDER: [[[usize; 2]; 2]; 2] = [[[0, 2], [1, 3]], [[0, 1], [2, 3]]];

const CUBE_EDGE_ORDER: [[[usize; 2]; 4]; 3] = [
    [[0, 4], [1, 5], [2, 6], [3, 7]],
    [[0, 2], [1, 3], [4, 6], [5, 7]],
    [[0, 1], [2, 3], [4, 5], [6, 7]],
];

impl CubicalGeometry {
    fn orientation_of_square(&self, verts: [Vert; 4]) -> [Direction; 2] {
        SQUARE_EDGE_ORDER.map(|edges| {
            edges
                .into_iter()
                .find_map(|[i, j]| {
                    let [v_i, v_j] = [verts[i], verts[j]];
                    match &self.verts[v_i].stratum.cmp(&self.verts[v_j].stratum) {
                        Ordering::Less => Some(Direction::Forward),
                        Ordering::Equal => None,
                        Ordering::Greater => Some(Direction::Backward),
                    }
                })
                .unwrap_or(Direction::Forward)
        })
    }

    fn orientation_of_cube(&self, verts: [Vert; 8]) -> [Direction; 3] {
        CUBE_EDGE_ORDER.map(|edges| {
            edges
                .into_iter()
                .find_map(|[i, j]| {
                    let [v_i, v_j] = [verts[i], verts[j]];
                    match &self.verts[v_i].stratum.cmp(&self.verts[v_j].stratum) {
                        Ordering::Less => Some(Direction::Forward),
                        Ordering::Equal => None,
                        Ordering::Greater => Some(Direction::Backward),
                    }
                })
                .unwrap_or(Direction::Forward)
        })
    }
}
