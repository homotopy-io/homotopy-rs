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
    pub struct Point = usize;
    pub struct Line = usize;
    pub struct Area = usize;
    pub struct Volume = usize;
}

pub trait ElementData {
    type Point: Copy + Eq;
    type Line: Copy + Eq;
    type Area: Copy + Eq;
    type Volume: Copy + Eq;
}

#[derive(Default, Clone, Debug)]
pub struct Geometry<E: ElementData> {
    pub verts: IdxVec<Vert, VertData>,
    pub curves: IdxVec<Curve, CurveData>,
    pub points: IdxVec<Point, E::Point>,
    pub lines: IdxVec<Line, E::Line>,
    pub areas: IdxVec<Area, E::Area>,
    pub volumes: IdxVec<Volume, E::Volume>,
}

impl<E> Geometry<E>
where
    E: ElementData,
{
    pub fn mk_vert(&mut self, vert: VertData) -> Vert {
        self.verts.push(vert)
    }

    pub fn mk_point(&mut self, point: E::Point) -> Point {
        self.points.push(point)
    }

    pub fn mk_line(&mut self, line: E::Line) -> Line {
        self.lines.push(line)
    }

    pub fn mk_area(&mut self, area: E::Area) -> Area {
        self.areas.push(area)
    }

    pub fn mk_volume(&mut self, volume: E::Volume) -> Volume {
        self.volumes.push(volume)
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
    pub stratum: f32,
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

pub fn calculate_stratum(path: &[SliceIndex]) -> f32 {
    path.iter()
        .map(|&index| match index {
            SliceIndex::Boundary(_) => -1.0,
            SliceIndex::Interior(Height::Regular(_)) => 0.0,
            SliceIndex::Interior(Height::Singular(_)) => 1.0,
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

#[derive(Default)]
pub struct CubeData;
#[derive(Default)]
pub struct SimplexData;

impl ElementData for CubeData {
    type Point = Vert;
    type Line = [Vert; 2];
    type Area = [Vert; 4];
    type Volume = [Vert; 8];
}

impl ElementData for SimplexData {
    type Point = Vert;
    type Line = [Vert; 2];
    type Area = [Vert; 3];
    type Volume = [Vert; 4];
}

pub type CubicalGeometry = Geometry<CubeData>;
pub type SimplicialGeometry = Geometry<SimplexData>;

impl CubicalGeometry {
    pub fn new(diagram: &DiagramN, depth: usize) -> Result<Self, DimensionError> {
        if depth > diagram.dimension() {
            return Err(DimensionError);
        }

        // Extract the mesh and layout.
        let mesh = Mesh::new(diagram, depth)?;
        let layout = Layout::new(diagram, depth)?;

        let mut geom: Self = Default::default();
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
            let n = match element.len() {
                1 => 0,
                2 => 1,
                4 => 2,
                8 if depth > 3 => 3,
                _ => continue,
            };

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
                    geom.mk_area(verts.try_into().unwrap());
                }
                8 => {
                    geom.mk_volume(verts.try_into().unwrap());
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
}

// Triangulation

impl CubicalGeometry {
    fn triangulate_square(&self, square: Area) -> impl Iterator<Item = [Vert; 3]> + '_ {
        const TRI_ASSEMBLY_ORDER: [[usize; 3]; 2] = [[0, 1, 3], [0, 3, 2]];

        let verts = self.areas[square];
        // Rotate the square
        let rotation = match self.orientation_of_square(verts) {
            [Direction::Forward, Direction::Forward] => [0, 1, 2, 3],
            [Direction::Forward, Direction::Backward] => [1, 3, 0, 2],
            [Direction::Backward, Direction::Forward] => [2, 0, 3, 1],
            [Direction::Backward, Direction::Backward] => [3, 2, 1, 0],
        };
        let verts = rotation.map(|i| verts[i]);

        TRI_ASSEMBLY_ORDER.into_iter().filter_map(move |[i, j, k]| {
            let tri @ [a, b, c] = [verts[i], verts[j], verts[k]];
            (a != b && a != c && b != c).then(|| tri)
        })
    }

    fn triangulate_cube(&self, cube: Volume) -> impl Iterator<Item = [Vert; 4]> + '_ {
        const TETRA_ASSEMBLY_ORDER: [[usize; 4]; 6] = [
            [0, 3, 1, 7],
            [0, 1, 5, 7],
            [0, 2, 3, 7],
            [0, 2, 7, 6],
            [0, 5, 4, 7],
            [0, 4, 6, 7],
        ];

        let verts = self.volumes[cube];
        // Rotate the cube
        let rotation = match self.orientation_of_cube(verts) {
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
            .filter_map(move |[i, j, k, l]| {
                let tetra @ [a, b, c, d] = [verts[i], verts[j], verts[k], verts[l]];
                (a != b && a != c && b != c && b != d && c != d).then(|| tetra)
            })
    }
}

impl From<CubicalGeometry> for SimplicialGeometry {
    fn from(geom: CubicalGeometry) -> Self {
        let areas = geom
            .areas
            .keys()
            .flat_map(|square| geom.triangulate_square(square))
            .collect();
        let volumes = geom
            .volumes
            .keys()
            .flat_map(|cube| geom.triangulate_cube(cube))
            .collect();

        Self {
            verts: geom.verts,
            curves: geom.curves,
            points: geom.points,
            lines: geom.lines,
            areas,
            volumes,
        }
    }
}

// Orientation

impl CubicalGeometry {
    fn orientation_of_square(&self, verts: [Vert; 4]) -> [Direction; 2] {
        const SQUARE_EDGE_ORDER: [[[usize; 2]; 2]; 2] = [[[0, 2], [1, 3]], [[0, 1], [2, 3]]];

        SQUARE_EDGE_ORDER.map(|edges| {
            edges
                .into_iter()
                .find_map(|[i, j]| {
                    let [v_i, v_j] = [verts[i], verts[j]];
                    match &self.verts[v_i]
                        .stratum
                        .partial_cmp(&self.verts[v_j].stratum)
                        .unwrap()
                    {
                        Ordering::Less => Some(Direction::Forward),
                        Ordering::Equal => None,
                        Ordering::Greater => Some(Direction::Backward),
                    }
                })
                .unwrap()
        })
    }

    fn orientation_of_cube(&self, verts: [Vert; 8]) -> [Direction; 3] {
        const CUBE_EDGE_ORDER: [[[usize; 2]; 4]; 3] = [
            [[0, 4], [1, 5], [2, 6], [3, 7]],
            [[0, 2], [1, 3], [4, 6], [5, 7]],
            [[0, 1], [2, 3], [4, 5], [6, 7]],
        ];

        CUBE_EDGE_ORDER.map(|edges| {
            edges
                .into_iter()
                .find_map(|[i, j]| {
                    let [v_i, v_j] = [verts[i], verts[j]];
                    match &self.verts[v_i]
                        .stratum
                        .partial_cmp(&self.verts[v_j].stratum)
                        .unwrap()
                    {
                        Ordering::Less => Some(Direction::Forward),
                        Ordering::Equal => None,
                        Ordering::Greater => Some(Direction::Backward),
                    }
                })
                .unwrap()
        })
    }
}
