use std::mem;

use homotopy_common::{hash::FastHashMap, idx::IdxVec};
use ultraviolet::Vec4;

use crate::{
    geom::{Area, CurveData, Line, SimplicialGeometry, Vert, VertData, Volume},
    parity::Parity,
};

struct Subdivider<'a> {
    geom: &'a mut SimplicialGeometry,
    smooth_time: bool,

    edge_division_memory: FastHashMap<[Vert; 2], Vert>,

    valence: IdxVec<Vert, u32>,
    smoothed: IdxVec<Vert, Vec4>,
}

impl<'a> Subdivider<'a> {
    #[inline]
    pub(super) fn new(geom: &'a mut SimplicialGeometry, smooth_time: bool) -> Self {
        Self {
            edge_division_memory: FastHashMap::with_capacity_and_hasher(
                geom.lines.len(),
                Default::default(),
            ),
            valence: IdxVec::with_capacity(geom.verts.len()),
            smoothed: IdxVec::with_capacity(geom.verts.len()),
            geom,
            smooth_time,
        }
    }

    #[inline]
    fn update_smoothed(&mut self, vert: Vert, smoothed: Vec4) {
        self.valence[vert] += 1;
        self.smoothed[vert] += smoothed;
    }

    fn interpolate_edge_uncached(&mut self, [a, b]: [Vert; 2], mk: Option<Parity>) -> Vert {
        // Interpolate
        let v = {
            let v_0 = &self.geom.verts[a];
            let v_1 = &self.geom.verts[b];
            let position = 0.5 * (v_0.position + v_1.position);
            let boundary = [0, 1, 2, 3].map(|i| v_0.boundary[i] && v_1.boundary[i]);
            let generator = v_0.generator;

            self.geom.mk_vert(VertData {
                position,
                boundary,
                generator,
                k: v_0.k,
            })
        };

        if let Some(parity) = mk {
            self.geom.mk_line([a, v], parity);
            self.geom.mk_line([v, b], parity);
        }

        v
    }

    fn interpolate_edge(&mut self, line: [Vert; 2], mk: Option<Parity>) -> Vert {
        self.edge_division_memory
            .get(&line)
            .copied()
            .unwrap_or_else(|| {
                let v = self.interpolate_edge_uncached(line, mk);
                self.edge_division_memory.insert(line, v);
                v
            })
    }

    fn interpolate_tri(&mut self, [a, b, c]: [Vert; 3], parity: Parity) {
        let v_0 = self.interpolate_edge([a, b], None);
        let v_1 = self.interpolate_edge([a, c], None);
        let v_2 = self.interpolate_edge([b, c], None);

        // Corners
        self.geom.mk_area([a, v_0, v_1], parity);
        self.geom.mk_area([v_0, b, v_2], parity);
        self.geom.mk_area([v_1, v_2, c], parity);

        // Inner triangle
        self.geom.mk_area([v_0, v_1, v_2], parity.flip());
    }

    fn interpolate_tetra(&mut self, [a, b, c, d]: [Vert; 4], parity: Parity) {
        let v_0 = self.interpolate_edge([a, b], None);
        let v_1 = self.interpolate_edge([a, c], None);
        let v_2 = self.interpolate_edge([a, d], None);
        let v_3 = self.interpolate_edge([b, c], None);
        let v_4 = self.interpolate_edge([b, d], None);
        let v_5 = self.interpolate_edge([c, d], None);

        // Corners
        self.geom.mk_volume([a, v_0, v_1, v_2], parity);
        self.geom.mk_volume([v_0, b, v_3, v_4], parity);
        self.geom.mk_volume([v_1, v_3, c, v_5], parity);
        self.geom.mk_volume([v_2, v_4, v_5, d], parity);

        // Inner octahedron
        self.geom.mk_volume([v_0, v_1, v_2, v_4], parity);
        self.geom.mk_volume([v_0, v_1, v_3, v_4], parity.flip());
        self.geom.mk_volume([v_1, v_3, v_4, v_5], parity.flip());
        self.geom.mk_volume([v_1, v_2, v_4, v_5], parity);
    }

    fn smooth_line(&mut self, line: Line) {
        let (line, _) = self.geom.lines[line];
        let smoothed = line
            .iter()
            .map(|v| self.geom.verts[*v].position)
            .sum::<Vec4>()
            / 2.0;

        for v in line {
            self.update_smoothed(v, smoothed);
        }
    }

    fn smooth_tri(&mut self, tri: Area) {
        let (tri, _) = self.geom.areas[tri];
        let smoothed = tri
            .iter()
            .map(|v| self.geom.verts[*v].position)
            .sum::<Vec4>()
            / 3.0;

        for v in tri {
            self.update_smoothed(v, smoothed);
        }
    }

    fn smooth_tetra(&mut self, tetra: Volume) {
        let (tetra, _) = self.geom.volumes[tetra];
        let smoothed = tetra
            .iter()
            .map(|v| self.geom.verts[*v].position)
            .sum::<Vec4>()
            / 4.0;

        for v in tetra {
            self.update_smoothed(v, smoothed);
        }
    }

    #[inline]
    pub(super) fn subdivide_once(&mut self) {
        // These capacities are carefully specified to minimise allocations during
        // subdivision. This keeps the caches hot and avoids wasting time in `malloc`.
        let mut curves = IdxVec::with_capacity(self.geom.curves.len());
        let mut lines = IdxVec::with_capacity(2 * self.geom.lines.len());
        let mut tris = IdxVec::with_capacity(4 * self.geom.areas.len());
        let mut tetras = IdxVec::with_capacity(8 * self.geom.volumes.len());
        mem::swap(&mut self.geom.curves, &mut curves);
        mem::swap(&mut self.geom.lines, &mut lines);
        mem::swap(&mut self.geom.areas, &mut tris);
        mem::swap(&mut self.geom.volumes, &mut tetras);

        // 2. Subdivide and obtain valence
        //
        // The order in which these passes are performed is important. We only want to
        // generate new geometrical elements when they're semantically important. Thus,
        // if we subdivide an edge of a square, it should only result in new lines if
        // that edge was already a line. Subdividing lines first gives us this property.
        for (line, parity) in lines.into_values() {
            self.interpolate_edge(line, Some(parity));
        }

        for (tri, parity) in tris.into_values() {
            self.interpolate_tri(tri, parity);
        }

        for (tetra, parity) in tetras.into_values() {
            self.interpolate_tetra(tetra, parity);
        }

        for curve in curves.into_values() {
            let mut verts = Vec::with_capacity(curve.verts.len() * 2);
            let mut parities = Vec::with_capacity(curve.parities.len() * 2);

            verts.push(curve.verts[0]);

            for i in 0..curve.verts.len() - 1 {
                let v0 = curve.verts[i];
                let v1 = curve.verts[i + 1];
                let parity = curve.parities[i];

                let line = if parity.is_even() { [v0, v1] } else { [v1, v0] };
                let interpolated = self.interpolate_edge(line, None);

                verts.push(interpolated);
                verts.push(v1);
                parities.push(parity);
                parities.push(parity);
            }

            self.geom.curves.push(CurveData {
                verts,
                parities,
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

        for tetra in self.geom.volumes.keys() {
            self.smooth_tetra(tetra);
        }

        for tri in self.geom.areas.keys() {
            self.smooth_tri(tri);
        }

        for line in self.geom.lines.keys() {
            self.smooth_line(line);
        }

        // 4. Update vertex positions and divide by valence
        for (vert, data) in self.smoothed.iter() {
            let valence = self.valence[vert];
            if valence > 0 {
                let vert = &mut self.geom.verts[vert];
                let new = *data / valence as f32;

                for i in 0..4 {
                    if !vert.boundary[i] && (i != 3 || self.smooth_time) {
                        vert.position[i] = new[i];
                    }
                }
            }
        }
    }
}

impl SimplicialGeometry {
    pub fn subdivide(&mut self, smooth_time: bool, depth: u8) {
        if depth == 0 {
            return;
        }

        let mut engine = Subdivider::new(self, smooth_time);
        for _ in 0..depth {
            engine.subdivide_once();
        }
    }
}
