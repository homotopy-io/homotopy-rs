use std::{
    cmp::Ordering,
    f32::consts::{PI, TAU},
    mem,
};

use homotopy_common::{declare_idx, hash::FastHashMap, idx::IdxVec};
use homotopy_core::{
    common::DimensionError, layout::Layout, mesh::Mesh, Diagram, Diagram0, SliceIndex,
};
use ultraviolet::{Mat3, Vec3, Vec4};

use crate::{
    parity::Parity,
    style::{GeneratorStyle, SignatureStyleData, VertexShape},
};

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
    pub lines: IdxVec<Line, (E::Line, Parity)>,
    pub areas: IdxVec<Area, (E::Area, Parity)>,
    pub volumes: IdxVec<Volume, (E::Volume, Parity)>,
}

impl<E> Geometry<E>
where
    E: ElementData,
{
    pub fn mk_vert(&mut self, vert: VertData) -> Vert {
        self.verts.push(vert)
    }

    pub fn mk_displaced_copy(&mut self, vert: Vert, displacement: Vec4) -> Vert {
        let vert @ VertData { position, .. } = self.verts[vert].clone();
        self.mk_vert(VertData {
            position: position + displacement,
            ..vert
        })
    }

    pub fn mk_point(&mut self, point: E::Point) -> Point {
        self.points.push(point)
    }

    pub fn mk_line(&mut self, line: E::Line, parity: Parity) -> Line {
        self.lines.push((line, parity))
    }

    pub fn mk_area(&mut self, area: E::Area, parity: Parity) -> Area {
        self.areas.push((area, parity))
    }

    pub fn mk_volume(&mut self, volume: E::Volume, parity: Parity) -> Volume {
        self.volumes.push((volume, parity))
    }

    pub fn bounds(&self) -> (Vec4, Vec4) {
        self.verts.values().fold(
            (
                Vec4::broadcast(f32::INFINITY),
                Vec4::broadcast(f32::NEG_INFINITY),
            ),
            |(min, max), vert| {
                (
                    min.min_by_component(vert.position),
                    max.max_by_component(vert.position),
                )
            },
        )
    }
}

// Vert data

#[derive(Clone, Debug)]
pub struct VertData {
    pub position: Vec4,
    pub boundary: [bool; 4],
    pub generator: Diagram0,
    pub k: usize,
}

pub fn calculate_boundary(path: &[SliceIndex]) -> Vec<bool> {
    path.iter()
        .map(|index| matches!(index, SliceIndex::Boundary(_)))
        .rev()
        .collect()
}

// Curve data

#[derive(Clone, Debug)]
pub struct CurveData {
    pub verts: Vec<Vert>,
    pub parities: Vec<Parity>,
    pub generator: Diagram0,
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
    pub fn new<const N: usize>(diagram: &Diagram) -> Result<Self, DimensionError> {
        if diagram.dimension() < N {
            return Err(DimensionError);
        }

        // Extract the mesh and layout.
        let mesh = Mesh::new(diagram)?;
        let layout = Layout::new(diagram)?;

        let mut geom = Self::default();
        let mut coord_to_vert: FastHashMap<[SliceIndex; N], Vert> = FastHashMap::default();

        for (path, diagram) in mesh.nodes() {
            let position = layout.get(path);
            let position =
                Vec4::from([0, 1, 2, 3].map(|i| position.get(i).copied().unwrap_or_default()));

            let boundary = calculate_boundary(&path);
            let boundary = [0, 1, 2, 3].map(|i| boundary.get(i).copied().unwrap_or_default());

            let vert = geom.mk_vert(VertData {
                position,
                boundary,
                generator: diagram.max_generator(),
                k: usize::MAX,
            });
            coord_to_vert.insert(path, vert);
        }

        for cube in mesh.cubes().filter(|cube| cube.visible) {
            let dim = cube.dimension();

            // We ignore top-dimensional cubes in 3D and 4D.
            if N >= 3 && dim == N {
                continue;
            }

            let verts = cube
                .points
                .into_iter()
                .map(|coord| coord_to_vert[&coord])
                .collect::<Vec<_>>();
            let parity = Parity::from_orientation(&cube.orientation);
            let d = geom.verts[verts[0]].generator;

            // We ignore cubes which are homotopies and are of dimension less than N - 1.
            // For example, this will affect points in 2D, points and lines in 3D, and so on.
            if dim + 1 < N && diagram.dimension().saturating_sub(d.generator.dimension) != dim {
                continue;
            }

            verts.iter().for_each(|v| geom.verts[*v].k = dim);

            match dim {
                0 => {
                    geom.mk_point(verts[0]);
                }
                1 => {
                    let mut verts: [Vert; 2] = verts.try_into().unwrap();
                    geom.mk_line(verts, parity);

                    // Curve extraction.
                    if !parity.is_even() {
                        verts.swap(0, 1);
                    }
                    let curve = geom.curves.values_mut().find(|curve| {
                        let &curve_target = curve.verts.last().unwrap();
                        curve_target == verts[0] && curve.generator == d
                    });
                    if let Some(curve) = curve {
                        curve.verts.push(verts[1]);
                        curve.parities.push(parity);
                    } else {
                        geom.curves.push(CurveData {
                            generator: d,
                            verts: verts.to_vec(),
                            parities: vec![parity],
                        });
                    }
                }
                2 => {
                    let verts: [Vert; 4] = verts.try_into().unwrap();
                    geom.mk_area(verts, parity);
                }
                3 => {
                    let verts: [Vert; 8] = verts.try_into().unwrap();
                    geom.mk_volume(verts, parity);
                }
                _ => (),
            }
        }

        // Center animation on origin and make time interval [-1,1]
        let (min, max) = geom.bounds();
        let translation = 0.5 * (max + min);
        let duration = 0.5 * (max.w - min.w);

        for vert in geom.verts.values_mut() {
            vert.position -= translation;
            vert.position.w /= duration;
        }

        Ok(geom)
    }
}

// Triangulation

impl CubicalGeometry {
    fn triangulate_square(&self, square: Area) -> impl Iterator<Item = ([Vert; 3], Parity)> + '_ {
        const TRI_ASSEMBLY_ORDER: [([usize; 3], Parity); 2] =
            [([0, 1, 3], Parity::Even), ([0, 2, 3], Parity::Odd)];

        let (verts, parity) = self.areas[square];
        TRI_ASSEMBLY_ORDER
            .into_iter()
            .filter_map(move |([i, j, k], tri_parity)| {
                let tri @ [a, b, c] = [verts[i], verts[j], verts[k]];
                (a != b && b != c).then(|| (tri, parity * tri_parity))
            })
    }

    fn triangulate_cube(&self, cube: Volume) -> impl Iterator<Item = ([Vert; 4], Parity)> + '_ {
        const TETRA_ASSEMBLY_ORDER: [([usize; 4], Parity); 6] = [
            ([0, 1, 3, 7], Parity::Even),
            ([0, 1, 5, 7], Parity::Odd),
            ([0, 2, 3, 7], Parity::Odd),
            ([0, 2, 6, 7], Parity::Even),
            ([0, 4, 5, 7], Parity::Even),
            ([0, 4, 6, 7], Parity::Odd),
        ];

        let (verts, parity) = self.volumes[cube];
        TETRA_ASSEMBLY_ORDER
            .into_iter()
            .filter_map(move |([i, j, k, l], tetra_parity)| {
                let tetra @ [a, b, c, d] = [verts[i], verts[j], verts[k], verts[l]];
                (a != b && b != c && c != d).then(|| (tetra, parity * tetra_parity))
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

// Geometry synthesis
//
// If we every get geometry shaders (via WebGPU) all of this can go
// and be replaced by real-time geometry synthesis.

impl SimplicialGeometry {
    pub fn inflate_point_3d(&mut self, point: Vert, samples: u8, generator_shape: &VertexShape) {
        use VertexShape::{Circle, Square};
        match generator_shape {
            Circle => self.inflate_point_3d_sphere(point, samples),
            Square => self.inflate_point_3d_cube(point),
        }
    }

    fn inflate_point_3d_sphere(&mut self, point: Vert, samples: u8) {
        use homotopy_common::idx::Idx;

        const SPHERE_RADIUS: f32 = 0.1;
        const STACK_SAMPLE_MODIFIER: usize = 3;
        const SECTOR_SAMPLE_MODIFIER: usize = 3;

        let stacks = samples as usize + STACK_SAMPLE_MODIFIER;
        let sectors = samples as usize + SECTOR_SAMPLE_MODIFIER;

        let north_pole = self.mk_displaced_copy(point, Vec4::unit_y() * SPHERE_RADIUS);
        let south_pole = self.mk_displaced_copy(point, -Vec4::unit_y() * SPHERE_RADIUS);

        for i in 1..stacks {
            let theta = 0.5 * PI - (i as f32 * PI / stacks as f32);
            let xz = SPHERE_RADIUS * f32::cos(theta);
            let y = SPHERE_RADIUS * f32::sin(theta);

            let len = self.verts.len();

            for j in 0..sectors {
                let phi = j as f32 * TAU / sectors as f32;
                let x = xz * f32::cos(phi);
                let z = xz * f32::sin(phi);
                self.mk_displaced_copy(point, Vec4::new(x, y, z, 0.));
            }

            if i == 1 {
                for j in 0..sectors {
                    let v_0 = Vert::new(len + j);
                    let v_1 = Vert::new(len + (j + 1) % sectors);
                    self.mk_area([north_pole, v_1, v_0], Parity::Even);
                }
            } else {
                for j in 0..sectors {
                    let v_0 = Vert::new(len + j);
                    let v_1 = Vert::new(len + (j + 1) % sectors);
                    let v_2 = Vert::new(v_0.index() - sectors);
                    let v_3 = Vert::new(v_1.index() - sectors);

                    self.mk_area([v_0, v_2, v_1], Parity::Even);
                    self.mk_area([v_1, v_2, v_3], Parity::Even);
                }
            }

            if i == stacks - 1 {
                for j in 0..sectors {
                    let v_0 = Vert::new(len + j);
                    let v_1 = Vert::new(len + (j + 1) % sectors);
                    self.mk_area([south_pole, v_0, v_1], Parity::Even);
                }
            }
        }
    }

    fn inflate_point_3d_cube(&mut self, point: Vert) {
        use homotopy_common::idx::Idx;

        const CUBE_SIDELENGTH: f32 = 0.17;
        const R: f32 = CUBE_SIDELENGTH / 2.;

        let len = self.verts.len();

        // To fix normals, we duplicate each vertex (four times) and only use each vertex once
        // NOTE: this creates 4 redundant vertices but is neater to implement than manual vertex
        // definitions
        for _ in 0..5 {
            self.mk_displaced_copy(point, Vec4::new(-R, -R, -R, 0.));
            self.mk_displaced_copy(point, Vec4::new(R, -R, -R, 0.));
            self.mk_displaced_copy(point, Vec4::new(R, R, -R, 0.));
            self.mk_displaced_copy(point, Vec4::new(-R, R, -R, 0.));
            self.mk_displaced_copy(point, Vec4::new(-R, -R, R, 0.));
            self.mk_displaced_copy(point, Vec4::new(R, -R, R, 0.));
            self.mk_displaced_copy(point, Vec4::new(R, R, R, 0.));
            self.mk_displaced_copy(point, Vec4::new(-R, R, R, 0.));
        }

        let mk_area = |s: &mut Self, verts: [usize; 3], used: &mut [usize; 8]| {
            let new_verts = [
                Vert::new(len + verts[0] + used[verts[0]]),
                Vert::new(len + verts[1] + used[verts[1]]),
                Vert::new(len + verts[2] + used[verts[2]]),
            ];
            used[verts[0]] += 8;
            used[verts[1]] += 8;
            used[verts[2]] += 8;
            s.mk_area(new_verts, Parity::Even);
        };

        let mut used = [0; 8];

        mk_area(self, [0, 1, 3], &mut used);
        mk_area(self, [3, 1, 2], &mut used);
        mk_area(self, [1, 5, 2], &mut used);
        mk_area(self, [2, 5, 6], &mut used);
        mk_area(self, [5, 4, 6], &mut used);
        mk_area(self, [6, 4, 7], &mut used);
        mk_area(self, [4, 0, 7], &mut used);
        mk_area(self, [7, 0, 3], &mut used);
        mk_area(self, [3, 2, 7], &mut used);
        mk_area(self, [7, 2, 6], &mut used);
        mk_area(self, [4, 5, 0], &mut used);
        mk_area(self, [0, 5, 1], &mut used);
    }

    fn inflate_tube_segment(
        &mut self,
        vert: Vert,
        normal: Vec3,
        binormal: Vec3,
        connect: Option<Parity>,
        sectors: u8,
    ) {
        use homotopy_common::idx::Idx;

        const TUBE_RADIUS: f32 = 0.05;

        let len = self.verts.len();

        for j in 0..sectors {
            let theta = f32::from(j) * TAU / f32::from(sectors);
            self.mk_displaced_copy(
                vert,
                (TUBE_RADIUS * (f32::cos(theta) * normal + f32::sin(theta) * binormal)).into(),
            );
        }

        if let Some(parity) = connect {
            let sectors = sectors as usize;

            for j in 0..sectors {
                let v_0 = Vert::new(len + j);
                let v_1 = Vert::new(len + ((j + 1) % sectors));
                let v_2 = Vert::new(v_0.index() - sectors);
                let v_3 = Vert::new(v_1.index() - sectors);

                if parity.is_even() {
                    self.mk_area([v_2, v_3, v_1], Parity::Even);
                    self.mk_area([v_2, v_0, v_1], Parity::Odd);
                } else {
                    self.mk_area([v_0, v_1, v_3], Parity::Odd);
                    self.mk_area([v_0, v_2, v_3], Parity::Even);
                }
            }
        }
    }

    fn inflate_curve_3d(&mut self, curve: Curve, samples: u8) {
        let mut verts = vec![];
        let mut parities = vec![];
        let sectors = samples;

        mem::swap(&mut self.curves[curve].verts, &mut verts);
        mem::swap(&mut self.curves[curve].parities, &mut parities);

        // The direction of the curve in the previous segment
        let mut d_0 = (self.verts[verts[1]].position - self.verts[verts[0]].position)
            .xyz()
            .normalized();
        // A vector in the previous normal plane (initialised to a numerically
        // stable choice of perpendicular vector to the initial value of `d_0`)
        let mut n = (if d_0.z < d_0.x {
            Vec3::new(d_0.y, -d_0.x, 0.)
        } else {
            Vec3::new(0., -d_0.z, d_0.y)
        })
        .normalized();

        self.inflate_tube_segment(verts[0], n, d_0.cross(n), None, sectors);

        for i in 2..verts.len() {
            let v_0 = verts[i - 1];
            let v_1 = verts[i];

            let d_1 =
                (self.verts[v_1].position.xyz() - self.verts[v_0].position.xyz()).normalized();
            let t = 0.5 * (d_1 + d_0);
            d_0 = d_1;

            n = t.cross(n).cross(t).normalized();
            let bn = t.cross(n).normalized();

            self.inflate_tube_segment(v_0, n, bn, Some(parities[i - 2]), sectors);

            if i == verts.len() - 1 {
                self.inflate_tube_segment(v_1, n, bn, Some(parities[i - 1]), sectors);
            }
        }
    }

    pub fn inflate_3d(&mut self, samples: u8, signature_styles: &impl SignatureStyleData) {
        for point in self.points.keys() {
            let d = self.verts[self.points[point]].generator;
            let shape = signature_styles
                .generator_style(d.generator)
                .map(GeneratorStyle::shape)
                .unwrap_or_default();
            self.inflate_point_3d(self.points[point], samples, &shape);
        }

        for curve in self.curves.keys() {
            self.inflate_curve_3d(curve, samples);
        }

        self.points.clear();
        self.lines.clear();
        self.curves.clear();
    }

    pub fn compute_normals_3d(&self) -> IdxVec<Vert, Vec3> {
        let mut normals = IdxVec::splat(Vec3::zero(), self.verts.len());

        for ([i, j, k], parity) in self.areas.values().copied() {
            if i != j && j != k && k != i {
                let v_1 = self.verts[i].position.xyz();
                let v_2 = self.verts[j].position.xyz();
                let v_3 = self.verts[k].position.xyz();

                let sign = if parity.is_even() { 1. } else { -1. };
                let n = sign * (v_2 - v_1).cross(v_3 - v_2);

                normals[i] += n;
                normals[j] += n;
                normals[k] += n;
            }
        }

        for normal in normals.values_mut() {
            normal.normalize();
        }

        normals
    }

    pub fn compute_normals_4d(&self) -> IdxVec<Vert, Vec4> {
        let mut normals = IdxVec::splat(Vec4::zero(), self.verts.len());

        for ([i, j, k, l], parity) in self.volumes.values().copied() {
            let origin = self.verts[i].position;
            let v_0 = self.verts[j].position - origin;
            let v_1 = self.verts[k].position - origin;
            let v_2 = self.verts[l].position - origin;

            let xs = Vec3::new(v_0.x, v_1.x, v_2.x);
            let ys = Vec3::new(v_0.y, v_1.y, v_2.y);
            let zs = Vec3::new(v_0.z, v_1.z, v_2.z);
            let ws = Vec3::new(v_0.w, v_1.w, v_2.w);

            let m_0 = Mat3::new(ys, zs, ws);
            let m_1 = Mat3::new(xs, zs, ws);
            let m_2 = Mat3::new(xs, ys, ws);
            let m_3 = Mat3::new(xs, ys, zs);

            let sign = if parity.is_even() { 1. } else { -1. };
            let n = sign
                * Vec4::new(
                    -m_0.determinant(),
                    m_1.determinant(),
                    -m_2.determinant(),
                    m_3.determinant(),
                );

            normals[i] += n;
            normals[j] += n;
            normals[k] += n;
            normals[l] += n;
        }

        for normal in normals.values_mut() {
            normal.normalize();
        }

        normals
    }

    pub fn time_order(&self, i: Vert, j: Vert) -> Ordering {
        self.verts[i]
            .position
            .w
            .partial_cmp(&self.verts[j].position.w)
            .unwrap_or(Ordering::Equal)
    }
}
