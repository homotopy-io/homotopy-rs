use std::{cmp, mem};

use homotopy_common::{hash::FastHashMap, idx::IdxVec};
use ultraviolet::{Mat4, Vec4};

use crate::geom::{Area, Boundary, CubicalGeometry, CurveData, Line, Vert, VertData, Volume};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Pass {
    Cubes,
    Squares,
    Lines,
}

struct Subdivider<'a> {
    geom: &'a mut CubicalGeometry,

    edge_division_memory: FastHashMap<[Vert; 2], Vert>,
    face_division_memory: FastHashMap<[Vert; 4], Vert>,

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
            face_division_memory: FastHashMap::with_capacity_and_hasher(
                geom.areas.len(),
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
    fn interpolate_edge_uncached(&mut self, mut line @ [a, b]: [Vert; 2], mk: bool) -> Vert {
        // Interpolate
        let v = {
            let v_0 = &self.geom.verts[a];
            let v_1 = &self.geom.verts[b];
            let v = 0.5 * (**v_0 + **v_1);
            let flow = 0.5 * (v_0.flow + v_1.flow);
            let boundary = cmp::max(Boundary::One, cmp::max(v_0.boundary, v_1.boundary));
            let generator = cmp::min_by_key(v_0, v_1, |v| {
                (v.flow.floor() as usize, v.generator.dimension)
            })
            .generator;

            self.geom.mk_vert(VertData {
                vert: v,
                flow,
                boundary,
                generator,
            })
        };

        if mk {
            self.geom.mk_line([a, v]);
            self.geom.mk_line([b, v]);
        }

        // Cache result
        line.sort_unstable();
        self.edge_division_memory.insert(line, v);
        v
    }

    fn interpolate_edge(&mut self, line: [Vert; 2], mk: bool) -> Vert {
        if line[0] == line[1] {
            return line[0];
        }

        let mut cloned = line;
        cloned.sort_unstable();

        self.edge_division_memory
            .get(&cloned)
            .copied()
            .unwrap_or_else(|| self.interpolate_edge_uncached(line, mk))
    }

    #[inline]
    fn interpolate_face_uncached(
        &mut self,
        mut square @ [a, b, c, d]: [Vert; 4],
        mk: bool,
    ) -> Vert {
        // Interpolate
        let v = {
            let v_1 = self.interpolate_edge([a, b], false);
            let v_2 = self.interpolate_edge([a, c], false);
            let v_3 = self.interpolate_edge([b, d], false);
            let v_4 = self.interpolate_edge([c, d], false);
            let center = self.interpolate_edge([v_1, v_4], false);

            if mk {
                let points = [a, b, c, d, v_1, v_2, v_3, v_4, center];

                for [i, j, k, l] in Self::SQUARE_ASSEMBLY_ORDER {
                    self.geom
                        .mk_area([points[i], points[j], points[k], points[l]]);
                }
            }

            center
        };

        // Cache result
        square.sort_unstable();
        self.face_division_memory.insert(square, v);
        v
    }

    fn interpolate_face(&mut self, square: [Vert; 4], mk: bool) -> Vert {
        let mut cloned = square;
        cloned.sort_unstable();

        // After sorting, we know that all the vertices are identical
        // if and only if the first vertex equals the last vertex. In
        // this case, we just return this unique vertex.
        if cloned[0] == cloned[3] {
            return cloned[0];
        }

        // In any of these three cases, we have a single pair of vertices.
        // As this corresponds to an edge, we just need to divide that edge
        // and move on.
        if (cloned[0] != cloned[1] && cloned[1] == cloned[3])
            || (cloned[0] == cloned[1] && cloned[2] == cloned[3])
            || (cloned[0] == cloned[2] && cloned[2] != cloned[3])
        {
            return self.interpolate_edge([cloned[0], cloned[3]], false);
        }

        self.face_division_memory
            .get(&cloned)
            .copied()
            .unwrap_or_else(|| self.interpolate_face_uncached(square, mk))
    }

    fn interpolate_cube(&mut self, cube: [Vert; 8]) {
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

        points[26] = self.interpolate_edge([points[20], points[25]], false);

        for [a, b, c, d, e, f, g, h] in Self::CUBE_ASSEMBLY_ORDER {
            self.geom.mk_volume([
                points[a], points[b], points[c], points[d], points[e], points[f], points[g],
                points[h],
            ]);
        }
    }

    #[inline]
    fn smooth_cube(&mut self, cube: Volume) {
        let cube @ [a, b, c, d, e, f, g, h] = self.geom.volumes[cube];
        // Gather the boundaries of its constituent vertices
        let bounds = [
            self.geom.verts[a].boundary,
            self.geom.verts[b].boundary,
            self.geom.verts[c].boundary,
            self.geom.verts[d].boundary,
            self.geom.verts[e].boundary,
            self.geom.verts[f].boundary,
            self.geom.verts[g].boundary,
            self.geom.verts[h].boundary,
        ];
        // and calculate a corresponding weight matrix
        let weights = Self::cube_weight_matrix(bounds);
        // Shape vertices as matrix
        let upper = Mat4::new(
            *self.geom.verts[a],
            *self.geom.verts[b],
            *self.geom.verts[c],
            *self.geom.verts[d],
        );
        let lower = Mat4::new(
            *self.geom.verts[e],
            *self.geom.verts[f],
            *self.geom.verts[g],
            *self.geom.verts[h],
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
        // Gather the boundaries of its constituent vertices
        let bounds = [
            self.geom.verts[a].boundary,
            self.geom.verts[b].boundary,
            self.geom.verts[c].boundary,
            self.geom.verts[d].boundary,
        ];
        // and calculate a corresponding weight matrix
        let weights = Self::square_weight_matrix(bounds);
        // Shape vertices as a matrix
        let square_matrix = Mat4::new(
            *self.geom.verts[a],
            *self.geom.verts[b],
            *self.geom.verts[c],
            *self.geom.verts[d],
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
        let bounds = [self.geom.verts[a].boundary, self.geom.verts[b].boundary];
        let weights = Self::line_weight_matrix(bounds);
        let line_matrix = Mat4::new(
            *self.geom.verts[a],
            *self.geom.verts[b],
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
            let mut interpolated = Vec::with_capacity(curve.len() * 2);
            for i in 0..curve.len() - 1 {
                interpolated.push(curve[i]);
                interpolated.push(self.interpolate_edge([curve[i], curve[i + 1]], false));
            }

            if let Some(point) = curve.last() {
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

        for cube in self.geom.volumes.keys() {
            self.smooth_cube(cube);
        }

        for square in self.geom.areas.keys() {
            self.smooth_square(square);
        }

        for line in self.geom.lines.keys() {
            self.smooth_line(line);
        }

        // 4. Update vertex positions and divide by valence
        for (vert, data) in self.smoothed.iter() {
            let valence = self.valence[vert];
            if valence > 0 {
                *self.geom.verts[vert] = *data / valence as f32;
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

    fn line_weight_matrix(bounds: [Boundary; 2]) -> Mat4 {
        let row_0 = match bounds[0] {
            Boundary::Zero => Vec4::unit_x(),
            _ => Vec4::new(0.5, 0.5, 0., 0.),
        };
        let row_1 = Vec4::unit_y();

        Mat4::new(row_0, row_1, Vec4::zero(), Vec4::zero())
    }

    fn square_weight_matrix(bounds: [Boundary; 4]) -> Mat4 {
        use Boundary::{One, Zero};
        // Vertex point
        let row_0 = match bounds[0] {
            Zero => Vec4::unit_x(),
            One => match (bounds[1], bounds[2]) {
                (One, _) => Vec4::new(0.5, 0.5, 0., 0.),
                (_, One) => Vec4::new(0.5, 0., 0.5, 0.),
                _ => Vec4::unit_x(),
            },
            _ => Vec4::broadcast(0.25),
        };
        let row_1 = match bounds[1] {
            One => Vec4::unit_y(),
            _ => Vec4::new(0., 0.5, 0., 0.5),
        };
        let row_2 = match bounds[2] {
            One => Vec4::unit_z(),
            _ => Vec4::new(0., 0., 0.5, 0.5),
        };
        let row_3 = Vec4::unit_w();

        Mat4::new(row_0, row_1, row_2, row_3)
    }

    fn cube_weight_matrix(bounds: [Boundary; 8]) -> [Mat4; 4] {
        use Boundary::{One, Three, Two, Zero};

        // Vertex point
        let (row_00, row_01) = match bounds[0] {
            Zero => (Vec4::unit_x(), Vec4::zero()),
            One => match (bounds[1], bounds[2], bounds[4]) {
                (One, _, _) => (Vec4::new(0.5, 0.5, 0.0, 0.), Vec4::zero()),
                (_, One, _) => (Vec4::new(0.5, 0., 0.5, 0.), Vec4::zero()),
                (_, _, One) => (Vec4::new(0.5, 0., 0., 0.), Vec4::new(0.5, 0., 0., 0.)),
                // if it turns out such geometry is consistent, replace with the identity row
                _ => panic!("Inconsistent geometry!"),
            },
            Two => match (bounds[3], bounds[5], bounds[6]) {
                (Two, _, _) => (Vec4::broadcast(0.25), Vec4::zero()),
                (_, Two, _) => (Vec4::new(0.25, 0.25, 0., 0.), Vec4::new(0.25, 0.25, 0., 0.)),
                (_, _, Two) => (Vec4::new(0.25, 0., 0.25, 0.), Vec4::new(0.25, 0., 0.25, 0.)),
                // if it turns out such geometry is consistent, replace with the identity row
                _ => panic!("Inconsistent geometry!"),
            },
            Three => (Vec4::broadcast(0.125), Vec4::broadcast(0.125)),
        };

        // Edge points
        let (row_10, row_11) = match bounds[1] {
            Zero => panic!("Inconsistent geometry!"),
            One => (Vec4::unit_y(), Vec4::zero()),
            Two => match (bounds[3], bounds[5]) {
                (Two, _) => (Vec4::new(0.0, 0.5, 0.0, 0.5), Vec4::zero()),
                (_, Two) => (Vec4::new(0., 0.5, 0., 0.), Vec4::new(0., 0.5, 0., 0.)),
                // if it turns out such geometry is consistent, replace with the identity row
                _ => panic!("Inconsistent geometry!"),
            },
            Three => (Vec4::new(0., 0.25, 0., 0.25), Vec4::new(0., 0.25, 0., 0.25)),
        };
        let (row_20, row_21) = match bounds[2] {
            Zero => panic!("Inconsistent geometry!"),
            One => (Vec4::unit_z(), Vec4::zero()),
            Two => match (bounds[3], bounds[6]) {
                (Two, _) => (Vec4::new(0., 0., 0.5, 0.5), Vec4::zero()),
                (_, Two) => (Vec4::new(0., 0., 0.5, 0.), Vec4::new(0., 0., 0.5, 0.)),
                // if it turns out such geometry is consistent, replace with the identity row
                _ => panic!("Inconsistent geometry!"),
            },
            Three => (Vec4::new(0., 0., 0.25, 0.25), Vec4::new(0., 0., 0.25, 0.25)),
        };
        let (row_40, row_41) = match bounds[4] {
            Zero => panic!("Inconsistent geometry!"),
            One => (Vec4::zero(), Vec4::unit_x()),
            Two => match (bounds[5], bounds[6]) {
                (Two, _) => (Vec4::zero(), Vec4::new(0.5, 0.5, 0., 0.)),
                (_, Two) => (Vec4::zero(), Vec4::new(0.5, 0., 0.5, 0.)),
                // if it turns out such geometry is consistent, replace with the identity row
                _ => panic!("Inconsistent geometry!"),
            },
            Three => (Vec4::zero(), Vec4::broadcast(0.25)),
        };

        // Face points
        let (row_30, row_31) = match bounds[3] {
            Two => (Vec4::unit_w(), Vec4::zero()),
            Three => (Vec4::new(0., 0., 0., 0.5), Vec4::new(0., 0., 0., 0.5)),
            _ => panic!("Inconsistent geometry!"),
        };
        let (row_50, row_51) = match bounds[5] {
            Two => (Vec4::zero(), Vec4::unit_y()),
            Three => (Vec4::zero(), Vec4::new(0., 0.5, 0., 0.5)),
            _ => panic!("Inconsistent geometry!"),
        };
        let (row_60, row_61) = match bounds[6] {
            Two => (Vec4::zero(), Vec4::unit_z()),
            Three => (Vec4::zero(), Vec4::new(0., 0., 0.5, 0.5)),
            _ => panic!("Inconsistent geometry!"),
        };

        // Centroid
        let (row_70, row_71) = (Vec4::zero(), Vec4::unit_w());

        [
            Mat4::new(row_00, row_10, row_20, row_30),
            Mat4::new(row_40, row_50, row_60, row_70),
            Mat4::new(row_01, row_11, row_21, row_31),
            Mat4::new(row_41, row_51, row_61, row_71),
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
