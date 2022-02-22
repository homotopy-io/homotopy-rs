use std::{cmp::Ordering, mem};

use homotopy_common::{hash::FastHashMap, idx::IdxVec};
use homotopy_core::Direction;
use ultraviolet::{Mat4, Vec4};

use crate::geom::{Area, CubicalGeometry, CurveData, Line, Vert, VertData, Volume};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Pass {
    Lines,
    Squares,
    Cubes,
}

struct Subdivider<'a> {
    geom: &'a mut CubicalGeometry,

    edge_division_memory: FastHashMap<[Vert; 2], Vert>,

    valence: IdxVec<Vert, u32>,
    smoothed: IdxVec<Vert, Vec4>,
    touched: IdxVec<Vert, Option<Pass>>,
}

impl<'a> Subdivider<'a> {
    /// Defines how new cubes should be from linearly divided points.
    ///
    /// Important property that is preserved - first point is a vertex point,
    /// edge and face points are also in precise positions.
    const CUBE_ASSEMBLY_ORDER: [[usize; 8]; 8] = [
        [0, 8, 9, 20, 10, 21, 22, 26],
        [1, 11, 8, 20, 12, 23, 21, 26],
        [2, 9, 13, 20, 14, 22, 24, 26],
        [3, 13, 11, 20, 15, 24, 23, 26],
        [4, 16, 10, 21, 17, 25, 22, 26],
        [5, 18, 12, 23, 16, 25, 21, 26],
        [6, 17, 14, 22, 19, 25, 24, 26],
        [7, 19, 15, 24, 18, 25, 23, 26],
    ];
    /// Defines all 12 edges of a cube based on vertex indices.
    const CUBE_EDGE_ORDER: [[usize; 2]; 12] = [
        [0, 1],
        [0, 2],
        [0, 4],
        [1, 3],
        [1, 5],
        [2, 3],
        [2, 6],
        [3, 7],
        [4, 5],
        [4, 6],
        [5, 7],
        [6, 7],
    ];
    /// Defines all 6 faces of a cube based on vertex indices.
    const CUBE_FACE_ORDER: [[usize; 4]; 6] = [
        [0, 1, 2, 3],
        [0, 1, 4, 5],
        [0, 2, 4, 6],
        [1, 3, 5, 7],
        [2, 3, 6, 7],
        [4, 5, 6, 7],
    ];
    /// Defines how new squares should be from linearly divided points.
    ///
    /// Important property that is preserved - first point is a vertex point, and similar
    /// for edge points.
    const SQUARE_ASSEMBLY_ORDER: [[usize; 4]; 4] =
        [[0, 4, 5, 8], [1, 6, 4, 8], [2, 5, 7, 8], [3, 7, 6, 8]];

    #[inline]
    pub(super) fn new(geom: &'a mut CubicalGeometry) -> Self {
        Self {
            edge_division_memory: FastHashMap::with_capacity_and_hasher(
                geom.lines.len(),
                Default::default(),
            ),
            valence: IdxVec::with_capacity(geom.verts.len()),
            smoothed: IdxVec::with_capacity(geom.verts.len()),
            touched: IdxVec::with_capacity(geom.verts.len()),
            geom,
        }
    }

    #[inline]
    fn update_smoothed(&mut self, vert: Vert, smoothed: Vec4, pass: Pass) {
        if let Some(touch) = self.touched[vert] {
            if touch < pass {
                return;
            }
        }

        self.valence[vert] += 1;
        self.smoothed[vert] += smoothed;
        self.touched[vert] = Some(pass);
    }

    #[inline]
    fn interpolate_edge_uncached(&mut self, [a, b]: [Vert; 2], mk: bool) -> Vert {
        // Interpolate
        let v = {
            let v_0 = &self.geom.verts[a];
            let v_1 = &self.geom.verts[b];
            let v = 0.5 * (v_0.position + v_1.position);
            let flow = 0.5 * (v_0.flow + v_1.flow);
            let boundary = v_0
                .boundary
                .iter()
                .zip(&v_1.boundary)
                .map(|(a, b)| *a && *b)
                .collect();
            let generator = v_0.min_generator(v_1).generator;

            self.geom.mk_vert(VertData {
                position: v,
                flow,
                boundary,
                generator,
            })
        };

        if mk {
            self.geom.mk_line([a, v]);
            self.geom.mk_line([b, v]);
        }

        v
    }

    fn interpolate_edge(&mut self, line @ [a, b]: [Vert; 2], mk: bool) -> Vert {
        let key = match self.direction_of_line(line) {
            None => return a,
            Some(Direction::Forward) => [a, b],
            Some(Direction::Backward) => [b, a],
        };

        self.edge_division_memory
            .get(&key)
            .copied()
            .unwrap_or_else(|| {
                let v = self.interpolate_edge_uncached(line, mk);
                self.edge_division_memory.insert(key, v);
                v
            })
    }

    fn interpolate_face(&mut self, square @ [a, b, c, d]: [Vert; 4], mk: bool) -> Vert {
        // Find the leading diagonal.
        let key = match self.direction_of_face(square) {
            [None, None] => return a,
            [None, Some(_)] => return self.interpolate_edge([a, b], false),
            [Some(_), None] => return self.interpolate_edge([a, c], false),
            [Some(Direction::Forward), Some(Direction::Forward)] => [a, d],
            [Some(Direction::Forward), Some(Direction::Backward)] => [b, c],
            [Some(Direction::Backward), Some(Direction::Forward)] => [c, b],
            [Some(Direction::Backward), Some(Direction::Backward)] => [d, a],
        };

        // Interpolate
        let center = self.interpolate_edge(key, false);

        if mk {
            let v_1 = self.interpolate_edge([a, b], false);
            let v_2 = self.interpolate_edge([a, c], false);
            let v_3 = self.interpolate_edge([b, d], false);
            let v_4 = self.interpolate_edge([c, d], false);

            let points = [a, b, c, d, v_1, v_2, v_3, v_4, center];

            for square in Self::SQUARE_ASSEMBLY_ORDER {
                let square = square.map(|i| points[i]);

                // Ignore the square if it doesn't have any area.
                if matches!(self.direction_of_face(square), [Some(_), Some(_)]) {
                    self.geom.mk_area(square);
                }
            }
        }

        center
    }

    fn interpolate_cube(&mut self, cube: [Vert; 8]) {
        // Find the leading diagonal.
        let diagonal = match self.direction_of_cube(cube) {
            [Some(Direction::Forward), Some(Direction::Forward), Some(Direction::Forward)] => {
                [cube[0], cube[7]]
            }
            [Some(Direction::Forward), Some(Direction::Forward), Some(Direction::Backward)] => {
                [cube[1], cube[6]]
            }
            [Some(Direction::Forward), Some(Direction::Backward), Some(Direction::Forward)] => {
                [cube[2], cube[5]]
            }
            [Some(Direction::Forward), Some(Direction::Backward), Some(Direction::Backward)] => {
                [cube[3], cube[4]]
            }
            [Some(Direction::Backward), Some(Direction::Forward), Some(Direction::Forward)] => {
                [cube[4], cube[3]]
            }
            [Some(Direction::Backward), Some(Direction::Forward), Some(Direction::Backward)] => {
                [cube[5], cube[2]]
            }
            [Some(Direction::Backward), Some(Direction::Backward), Some(Direction::Forward)] => {
                [cube[6], cube[1]]
            }
            [Some(Direction::Backward), Some(Direction::Backward), Some(Direction::Backward)] => {
                [cube[7], cube[0]]
            }
            _ => panic!(),
        };

        let mut points = {
            use homotopy_common::idx::Idx;
            [Vert::new(0); 27]
        };

        points[0..8].copy_from_slice(&cube);

        for (i, edge) in Self::CUBE_EDGE_ORDER.iter().enumerate() {
            points[i + 8] = self.interpolate_edge([points[edge[0]], points[edge[1]]], false);
        }

        for (i, [a, b, c, d]) in Self::CUBE_FACE_ORDER.into_iter().enumerate() {
            points[i + 20] =
                self.interpolate_face([points[a], points[b], points[c], points[d]], false);
        }

        points[26] = self.interpolate_edge(diagonal, false);

        for cube in Self::CUBE_ASSEMBLY_ORDER {
            let cube = cube.map(|i| points[i]);

            // Ignore the cube if it doesn't have any volume.
            if matches!(self.direction_of_cube(cube), [Some(_), Some(_), Some(_)]) {
                self.geom.mk_volume(cube);
            }
        }
    }

    #[inline]
    fn direction_of_line(&self, [a, b]: [Vert; 2]) -> Option<Direction> {
        match self.geom.verts[a]
            .flow
            .partial_cmp(&self.geom.verts[b].flow)
            .unwrap()
        {
            Ordering::Less => Some(Direction::Forward),
            Ordering::Equal => None,
            Ordering::Greater => Some(Direction::Backward),
        }
    }

    #[inline]
    fn direction_of_face(&self, verts: [Vert; 4]) -> [Option<Direction>; 2] {
        const SQUARE_EDGE_ORDER: [[[usize; 2]; 2]; 2] = [[[0, 2], [1, 3]], [[0, 1], [2, 3]]];

        SQUARE_EDGE_ORDER.map(|edges| {
            edges
                .into_iter()
                .find_map(|[i, j]| self.direction_of_line([verts[i], verts[j]]))
        })
    }

    #[inline]
    fn direction_of_cube(&self, verts: [Vert; 8]) -> [Option<Direction>; 3] {
        const CUBE_EDGE_ORDER: [[[usize; 2]; 4]; 3] = [
            [[0, 4], [1, 5], [2, 6], [3, 7]],
            [[0, 2], [1, 3], [4, 6], [5, 7]],
            [[0, 1], [2, 3], [4, 5], [6, 7]],
        ];

        CUBE_EDGE_ORDER.map(|edges| {
            edges
                .into_iter()
                .find_map(|[i, j]| self.direction_of_line([verts[i], verts[j]]))
        })
    }

    #[inline]
    fn smooth_cube(&mut self, cube: Volume) {
        let cube @ [a, b, c, d, e, f, g, h] = self.geom.volumes[cube];
        // Calculate a corresponding weight matrix
        let weights = Self::cube_weight_matrix();
        // Shape vertices as matrix
        let upper = Mat4::new(
            self.geom.verts[a].position,
            self.geom.verts[b].position,
            self.geom.verts[c].position,
            self.geom.verts[d].position,
        );
        let lower = Mat4::new(
            self.geom.verts[e].position,
            self.geom.verts[f].position,
            self.geom.verts[g].position,
            self.geom.verts[h].position,
        );
        // Tranform
        let upper_transformed = upper * weights[0] + lower * weights[2];
        let lower_transformed = upper * weights[1] + lower * weights[3];
        // Update positions
        for i in 0..4 {
            self.update_smoothed(cube[i], upper_transformed[i], Pass::Cubes);
            self.update_smoothed(cube[4 + i], lower_transformed[i], Pass::Cubes);
        }
    }

    #[inline]
    fn smooth_square(&mut self, square: Area) {
        let square @ [a, b, c, d] = self.geom.areas[square];
        // Calculate a corresponding weight matrix
        let weights = Self::square_weight_matrix();
        // Shape vertices as a matrix
        let square_matrix = Mat4::new(
            self.geom.verts[a].position,
            self.geom.verts[b].position,
            self.geom.verts[c].position,
            self.geom.verts[d].position,
        );
        // Transform
        let transformed = square_matrix * weights;
        // Update positions
        for i in 0..4 {
            self.update_smoothed(square[i], transformed[i], Pass::Squares);
        }
    }

    #[inline]
    fn smooth_line(&mut self, line: Line) {
        let line @ [a, b] = self.geom.lines[line];
        let weights = Self::line_weight_matrix();
        let line_matrix = Mat4::new(
            self.geom.verts[a].position,
            self.geom.verts[b].position,
            Vec4::zero(),
            Vec4::zero(),
        );
        let transformed = line_matrix * weights;
        for i in 0..2 {
            self.update_smoothed(line[i], transformed[i], Pass::Lines);
        }
    }

    #[inline]
    pub(super) fn subdivide_once(&mut self) {
        // TODO(@doctorn) see below
        // (0. In debug, clone a copy of the original diagram for sanity checking)
        // #[cfg(debug_assertions)]
        // let unmodified = self.geom.clone();

        // 1. Remove all elements from geom
        // These capacities are carefully specified to minimise allocations during
        // subdivision. This keeps the caches hot and avoids wasting time in `malloc`.
        let mut curves = IdxVec::with_capacity(self.geom.curves.len());
        let mut lines = IdxVec::with_capacity(2 * self.geom.lines.len());
        let mut squares = IdxVec::with_capacity(4 * self.geom.areas.len());
        let mut cubes = IdxVec::with_capacity(8 * self.geom.volumes.len());
        mem::swap(&mut self.geom.curves, &mut curves);
        mem::swap(&mut self.geom.lines, &mut lines);
        mem::swap(&mut self.geom.areas, &mut squares);
        mem::swap(&mut self.geom.volumes, &mut cubes);

        // 2. Subdivide and obtain valence
        //
        // The order in which these passes are performed is important. We only want to
        // generate new geometrical elements when they're semantically important. Thus,
        // if we subdivide an edge of a square, it should only result in new lines if
        // that edge was already a line. Subdividing lines first gives us this property.
        for line in lines.into_values() {
            self.interpolate_edge(line, true);
        }

        for square in squares.into_values() {
            self.interpolate_face(square, true);
        }

        for cube in cubes.into_values() {
            self.interpolate_cube(cube);
        }

        for curve in curves.into_values() {
            let mut interpolated = Vec::with_capacity(curve.verts.len() * 2);
            for i in 0..curve.verts.len() - 1 {
                interpolated.push(curve.verts[i]);
                interpolated
                    .push(self.interpolate_edge([curve.verts[i], curve.verts[i + 1]], false));
            }

            if let Some(point) = curve.verts.last() {
                interpolated.push(*point);
            }

            self.geom.curves.push(CurveData {
                verts: interpolated,
                generator: curve.generator,
            });
        }

        // 3. Smooth
        //
        // Again, the order of these passes is critical. In particular, we smooth in
        // the reverse order to the order we interpolated. This guarantees that a vertex's
        // new position reflects its role in the highest-dimensional geometrical element
        // it participates in.
        let len = self.geom.verts.len();
        self.valence = IdxVec::splat(0, len);
        self.smoothed = IdxVec::splat(Vec4::zero(), len);
        self.touched = IdxVec::splat(None, len);

        for line in self.geom.lines.keys() {
            self.smooth_line(line);
        }

        for square in self.geom.areas.keys() {
            self.smooth_square(square);
        }

        for cube in self.geom.volumes.keys() {
            self.smooth_cube(cube);
        }

        // 4. Update vertex positions and divide by valence
        for (vert, data) in self.smoothed.iter() {
            let valence = self.valence[vert];
            if valence > 0 {
                let vert = &mut self.geom.verts[vert];
                let new = *data / valence as f32;
                vert.position = [0, 1, 2, 3]
                    .map(|i| {
                        if i == 4 || vert.boundary[i] {
                            vert.position[i]
                        } else {
                            new[i]
                        }
                    })
                    .into();
            }
        }

        // TODO(@doctorn) fix spurious failures
        // (5. In debug, sanity check the subdivided geometry)
        // #[cfg(debug_assertions)]
        // debug_assert!(self.bounds_preserved(&unmodified));
    }

    // TODO(@doctorn)
    #[allow(unused)]
    #[cfg(debug_assertions)]
    fn bounds_preserved(&self, unmodified: &CubicalGeometry) -> bool {
        let (unmodified_min, unmodified_max) = unmodified.bounds();
        let (min, max) = self.geom.bounds();

        min.x >= unmodified_min.x
            && min.y >= unmodified_min.y
            && min.z >= unmodified_min.z
            && min.w >= unmodified_min.w
            && max.x <= unmodified_max.x
            && max.y <= unmodified_max.y
            && max.z <= unmodified_max.z
            && max.w <= unmodified_max.w
    }

    fn line_weight_matrix() -> Mat4 {
        Mat4::new(
            Vec4::new(0.5, 0.5, 0., 0.),
            Vec4::new(0.5, 0.5, 0., 0.),
            Vec4::zero(),
            Vec4::zero(),
        )
    }

    fn square_weight_matrix() -> Mat4 {
        Mat4::new(
            Vec4::broadcast(0.25),
            Vec4::broadcast(0.25),
            Vec4::broadcast(0.25),
            Vec4::broadcast(0.25),
        )
    }

    fn cube_weight_matrix() -> [Mat4; 4] {
        [
            Mat4::new(
                Vec4::broadcast(0.125),
                Vec4::broadcast(0.125),
                Vec4::broadcast(0.125),
                Vec4::broadcast(0.125),
            ),
            Mat4::new(
                Vec4::broadcast(0.125),
                Vec4::broadcast(0.125),
                Vec4::broadcast(0.125),
                Vec4::broadcast(0.125),
            ),
            Mat4::new(
                Vec4::broadcast(0.125),
                Vec4::broadcast(0.125),
                Vec4::broadcast(0.125),
                Vec4::broadcast(0.125),
            ),
            Mat4::new(
                Vec4::broadcast(0.125),
                Vec4::broadcast(0.125),
                Vec4::broadcast(0.125),
                Vec4::broadcast(0.125),
            ),
        ]
    }
}

impl CubicalGeometry {
    pub fn subdivide(&mut self, depth: u8) {
        if depth == 0 {
            return;
        }

        let mut engine = Subdivider::new(self);
        for _ in 0..depth {
            engine.subdivide_once();
        }
    }
}
