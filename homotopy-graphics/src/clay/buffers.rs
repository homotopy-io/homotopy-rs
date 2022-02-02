use std::{
    cmp::Ordering,
    f32::consts::{PI, TAU},
    mem,
};

use homotopy_common::{
    declare_idx,
    hash::FastHashMap,
    idx::{Idx, IdxVec},
    parity,
};
use homotopy_core::Generator;
use ultraviolet::{Mat3, Vec3, Vec4};

use crate::{
    geom::{SimplicialGeometry, Vert},
    gl::{
        array::VAO_LIMIT,
        buffer::{Buffer, ElementBuffer, ElementKind},
        GlCtx, Result,
    },
};

const SPHERE_RADIUS: f32 = 0.1;
const TUBE_RADIUS: f32 = 0.05;

pub struct TriVertexArrayData {
    pub element_buffer: ElementBuffer,
    pub wireframe_element_buffer: ElementBuffer,
    pub vertex_buffer: Buffer<Vec3>,
    pub normal_buffer: Buffer<Vec3>,
    pub generator: Generator,
}

struct TriBufferingState {
    elements: Vec<u16>,
    verts: Vec<Vec3>,
    mapping: FastHashMap<Vert, u16>,
    wireframe_elements: Vec<u16>,
    normals: Vec<Vec3>,
}

struct TriBufferingCtx<'a> {
    ctx: &'a GlCtx,
    geom: &'a SimplicialGeometry,
    state: FastHashMap<Generator, TriBufferingState>,
    complete_arrays: Vec<TriVertexArrayData>,
    geometry_samples: u8,
}

impl<'a> TriBufferingCtx<'a> {
    fn new(ctx: &'a GlCtx, geom: &'a SimplicialGeometry, geometry_samples: u8) -> Self {
        Self {
            ctx,
            geom,
            state: Default::default(),
            complete_arrays: vec![],
            geometry_samples,
        }
    }

    // FIXME(@doctorn) this shouldn't need to know how many vertices are required
    // up front - should handle 'overflow errors' gracefully and chop geometries
    // along these boundaries (copying any duplicated normal data?)
    fn with_state<F, U>(&mut self, generator: Generator, required: usize, f: F) -> Result<U>
    where
        F: FnOnce(&mut TriBufferingState) -> U,
    {
        let state = self.state.entry(generator).or_default();

        if state.verts.len() + required > VAO_LIMIT {
            let mut completed = Default::default();
            mem::swap(state, &mut completed);
            self.complete_arrays
                .push(completed.extract_buffers(self.ctx, generator)?);
        }

        Ok(f(state))
    }

    fn push_tri(&mut self, tri: [Vert; 3]) -> Result<()> {
        let generator = tri
            .into_iter()
            .map(|v| &self.geom.verts[v])
            .min_by_key(|v| (v.stratum.floor() as usize, v.generator.dimension))
            .unwrap()
            .generator;
        let geom = self.geom;

        self.with_state(generator, 3, |state| {
            let v_0 = state.push_vert(geom, tri[0]);
            let v_1 = state.push_vert(geom, tri[1]);
            let v_2 = state.push_vert(geom, tri[2]);

            state.push_tri(v_0, v_1, v_2);
        })
    }

    fn push_curve(&mut self, generator: Generator, curve: &[Vert]) -> Result<()> {
        let geom = self.geom;
        let sectors = self.geometry_samples;

        self.with_state(generator, curve.len() * sectors as usize, |state| {
            // The direction of the curve in the previous segment
            let mut d_0 = (*geom.verts[curve[1]] - *geom.verts[curve[0]])
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

            state.push_tube_sector(geom.verts[curve[0]].xyz(), n, d_0.cross(n), false, sectors);

            for i in 2..curve.len() {
                let v_0 = geom.verts[curve[i - 1]].xyz();
                let v_1 = geom.verts[curve[i]].xyz();

                let d_1 = (v_1 - v_0).normalized();
                let t = 0.5 * (d_1 + d_0);
                d_0 = d_1;

                n = t.cross(n).cross(t).normalized();
                let bn = t.cross(n).normalized();

                state.push_tube_sector(v_0, n, bn, true, sectors);

                if i == curve.len() - 1 {
                    state.push_tube_sector(v_1, n, bn, true, sectors);
                }
            }
        })
    }

    fn push_point(&mut self, point: Vert) -> Result<()> {
        let geom = self.geom;
        let generator = geom.verts[point].generator;

        let samples = u16::from(self.geometry_samples);
        // Algorithm works with these set independently.
        let stacks = samples;
        let sectors = samples;

        self.with_state(generator, (stacks * sectors) as usize + 2, |state| {
            let origin = geom.verts[point].xyz();
            let north_pole = state.push_unmapped_vert(origin + Vec3::unit_y() * SPHERE_RADIUS);
            let south_pole = state.push_unmapped_vert(origin - Vec3::unit_y() * SPHERE_RADIUS);

            for i in 1..stacks {
                let theta = 0.5 * PI - (f32::from(i) * PI / f32::from(stacks));
                let xz = SPHERE_RADIUS * f32::cos(theta);
                let y = SPHERE_RADIUS * f32::sin(theta);

                let len = state.verts.len() as u16;

                for j in 0..sectors {
                    let phi = f32::from(j) * TAU / f32::from(sectors);
                    let x = xz * f32::cos(phi);
                    let z = xz * f32::sin(phi);
                    state.push_unmapped_vert(origin + Vec3::new(x, y, z));
                }

                if i == 1 {
                    for j in 0..sectors {
                        let v_0 = len + j;
                        let v_1 = len + (j + 1) % sectors;
                        state.push_tri(north_pole, v_1, v_0);
                    }
                } else {
                    for j in 0..sectors {
                        let v_0 = len + j;
                        let v_1 = len + (j + 1) % sectors;
                        let v_2 = v_0 - sectors;
                        let v_3 = v_1 - sectors;

                        state.push_tri(v_0, v_2, v_1);
                        state.push_tri(v_1, v_2, v_3);
                    }
                }

                if i == stacks - 1 {
                    for j in 0..sectors {
                        let v_0 = len + j;
                        let v_1 = len + (j + 1) % sectors;
                        state.push_tri(south_pole, v_0, v_1);
                    }
                }
            }
        })
    }

    fn extract_buffers(mut self) -> Result<Vec<TriVertexArrayData>> {
        for tri in self.geom.tris() {
            self.push_tri(tri)?;
        }

        for curve in self.geom.curves.values() {
            self.push_curve(curve.generator, &*curve)?;
        }

        for point in self.geom.points() {
            self.push_point(point)?;
        }

        for (generator, state) in self.state {
            self.complete_arrays
                .push(state.extract_buffers(self.ctx, generator)?);
        }

        Ok(self.complete_arrays)
    }
}

impl TriBufferingState {
    fn push_wireframe_element(&mut self, i: u16, j: u16) {
        self.wireframe_elements.push(i);
        self.wireframe_elements.push(j);
    }

    fn push_element(&mut self, i: u16, j: u16, k: u16) {
        self.push_wireframe_element(i, j);
        self.push_wireframe_element(j, k);
        self.push_wireframe_element(k, i);

        self.elements.push(i);
        self.elements.push(j);
        self.elements.push(k);
    }

    fn push_vert(&mut self, geom: &SimplicialGeometry, v: Vert) -> u16 {
        if let Some(&idx) = self.mapping.get(&v) {
            return idx;
        }

        let idx = self.push_unmapped_vert(geom.verts[v].xyz());
        self.mapping.insert(v, idx);
        idx
    }

    fn push_unmapped_vert(&mut self, v: Vec3) -> u16 {
        let idx = self.verts.len() as u16;
        self.verts.push(v);
        self.normals.push(Vec3::zero());
        idx
    }

    fn push_tri(&mut self, i: u16, j: u16, k: u16) {
        if i != j && j != k && k != i {
            self.push_element(i, j, k);

            let v_1 = self.verts[i as usize];
            let v_2 = self.verts[j as usize];
            let v_3 = self.verts[k as usize];
            let n = (v_2 - v_1).cross(v_3 - v_1);

            self.normals[i as usize] += n;
            self.normals[j as usize] += n;
            self.normals[k as usize] += n;
        }
    }

    fn push_tube_sector(
        &mut self,
        vert: Vec3,
        normal: Vec3,
        binormal: Vec3,
        connect: bool,
        sectors: u8,
    ) {
        let len = self.verts.len() as u16;

        for j in 0..sectors {
            let theta = f32::from(j) * TAU / f32::from(sectors);
            self.push_unmapped_vert(
                vert + TUBE_RADIUS * f32::cos(theta) * normal
                    + TUBE_RADIUS * f32::sin(theta) * binormal,
            );
        }

        if connect {
            let sectors = u16::from(sectors);

            for j in 0..sectors {
                let v_0 = len + j;
                let v_1 = len + ((j + 1) % sectors);
                let v_2 = v_0 - sectors;
                let v_3 = v_1 - sectors;

                self.push_tri(v_0, v_2, v_1);
                self.push_tri(v_1, v_2, v_3);
            }
        }
    }

    fn extract_buffers(mut self, ctx: &GlCtx, generator: Generator) -> Result<TriVertexArrayData> {
        // Average normals
        for normal in &mut self.normals {
            normal.normalize();
        }

        // Buffer data
        let element_buffer = ctx.mk_element_buffer(&self.elements, ElementKind::Triangles)?;
        let wireframe_element_buffer =
            ctx.mk_element_buffer(&self.wireframe_elements, ElementKind::Lines)?;
        let vertex_buffer = ctx.mk_buffer(&self.verts)?;
        let normal_buffer = ctx.mk_buffer(&self.normals)?;

        Ok(TriVertexArrayData {
            element_buffer,
            wireframe_element_buffer,
            vertex_buffer,
            normal_buffer,
            generator,
        })
    }
}

impl Default for TriBufferingState {
    fn default() -> Self {
        Self {
            elements: Vec::with_capacity(VAO_LIMIT),
            verts: Vec::with_capacity(VAO_LIMIT),
            mapping: FastHashMap::with_capacity_and_hasher(VAO_LIMIT, Default::default()),
            wireframe_elements: Vec::with_capacity(VAO_LIMIT),
            normals: Vec::with_capacity(VAO_LIMIT),
        }
    }
}

// TODO(@doctorn) remove and replace with partitioned set of buffers
pub struct TetraBuffers {
    pub element_buffer: ElementBuffer,
    pub projected_wireframe_element_buffer: ElementBuffer,
    pub animated_wireframe_element_buffer: ElementBuffer,
    pub vertex_start_buffer: Buffer<Vec4>,
    pub vertex_end_buffer: Buffer<Vec4>,
    pub wireframe_vertex_buffer: Buffer<Vec3>,
    pub normal_start_buffer: Buffer<Vec4>,
    pub normal_end_buffer: Buffer<Vec4>,
}

declare_idx! { struct Segment = u16; }

struct TetraBufferer<'a> {
    geom: &'a SimplicialGeometry,

    segment_cache: FastHashMap<(Vert, Vert), Segment>,
    elements: Vec<u16>,
    animated_wireframe_elements: Vec<u16>,

    segment_starts: IdxVec<Segment, Vec4>,
    segment_ends: IdxVec<Segment, Vec4>,
    normals: IdxVec<Vert, Vec4>,

    projected_wireframe_elements: Vec<u16>,
    wireframe_vertices: Vec<Vec3>,
}

impl<'a> TetraBufferer<'a> {
    fn new(geom: &'a SimplicialGeometry) -> Self {
        let len = geom.elements.len();
        let segments = 6 * len;

        Self {
            geom,

            segment_cache: FastHashMap::with_capacity_and_hasher(segments, Default::default()),
            elements: Vec::with_capacity(segments),
            animated_wireframe_elements: Vec::with_capacity(segments),

            segment_starts: IdxVec::with_capacity(segments),
            segment_ends: IdxVec::with_capacity(segments),
            normals: IdxVec::splat(Vec4::zero(), geom.verts.len()),

            projected_wireframe_elements: Vec::with_capacity(len * 8),
            wireframe_vertices: geom.verts.values().map(|v| v.xyz()).collect(),
        }
    }

    fn mk_segment_uncached(&mut self, i: Vert, j: Vert) -> Segment {
        self.projected_wireframe_elements.push(i.index() as u16);
        self.projected_wireframe_elements.push(j.index() as u16);

        let segment = self.segment_starts.push(*self.geom.verts[i]);
        self.segment_ends.push(*self.geom.verts[j]);
        self.segment_cache.insert((i, j), segment);
        segment
    }

    fn mk_segment(&mut self, i: Vert, j: Vert) -> Segment {
        self.segment_cache
            .get(&(i, j))
            .copied()
            .unwrap_or_else(|| self.mk_segment_uncached(i, j))
    }

    fn push_tri(&mut self, i: Segment, j: Segment, k: Segment) {
        if i != j && j != k && k != i {
            let i = i.index() as u16;
            let j = j.index() as u16;
            let k = k.index() as u16;

            self.elements.push(i);
            self.elements.push(j);
            self.elements.push(k);
        }
    }

    fn push_wireframe_tri(&mut self, mut tri: [Vert; 3]) {
        parity::sort_3(&mut tri, |i, j| self.time_order(i, j));
        let [i, j, k] = tri;

        let ij = self.mk_segment(i, j);
        let ik = self.mk_segment(i, k);
        let jk = self.mk_segment(j, k);

        if ij != ik && ik != jk {
            let ij = ij.index() as u16;
            let ik = ik.index() as u16;
            let jk = jk.index() as u16;

            self.animated_wireframe_elements.push(ij);
            self.animated_wireframe_elements.push(ik);
            self.animated_wireframe_elements.push(jk);
            self.animated_wireframe_elements.push(ik);
        }
    }

    fn push_tetra(&mut self, mut tetra: [Vert; 4]) {
        let parity = parity::sort_4(&mut tetra, |i, j| self.time_order(i, j));
        let [i, j, k, l] = tetra;

        let ij = self.mk_segment(i, j);
        let ik = self.mk_segment(i, k);
        let il = self.mk_segment(i, l);
        let jk = self.mk_segment(j, k);
        let jl = self.mk_segment(j, l);
        let kl = self.mk_segment(k, l);

        {
            let origin = *self.geom.verts[i];
            let v_0 = *self.geom.verts[j] - origin;
            let v_1 = *self.geom.verts[k] - origin;
            let v_2 = *self.geom.verts[l] - origin;

            let xs = Vec3::new(v_0.x, v_1.x, v_2.x);
            let ys = Vec3::new(v_0.y, v_1.y, v_2.y);
            let zs = Vec3::new(v_0.z, v_1.z, v_2.z);
            let ws = Vec3::new(v_0.w, v_1.w, v_2.w);

            let m_0 = Mat3::new(ys, zs, ws);
            let m_1 = Mat3::new(xs, zs, ws);
            let m_2 = Mat3::new(xs, ys, ws);
            let m_3 = Mat3::new(xs, ys, zs);

            let mut normal = Vec4::new(
                -m_0.determinant(),
                m_1.determinant(),
                -m_2.determinant(),
                m_3.determinant(),
            );

            if !parity {
                normal = -normal;
            }

            self.normals[i] += normal;
            self.normals[j] += normal;
            self.normals[k] += normal;
            self.normals[l] += normal;
        }

        if parity {
            self.push_tri(ij, il, ik);
            self.push_tri(jl, ik, jk);
            self.push_tri(jl, il, ik);
            self.push_tri(jl, il, kl);
        } else {
            self.push_tri(il, ij, ik);
            self.push_tri(ik, jl, jk);
            self.push_tri(il, jl, ik);
            self.push_tri(il, jl, kl);
        }
    }

    fn time_order(&self, i: Vert, j: Vert) -> Ordering {
        self.geom.verts[i]
            .w
            .partial_cmp(&self.geom.verts[j].w)
            .unwrap_or(Ordering::Equal)
    }

    fn extract_buffers(mut self, ctx: &GlCtx) -> Result<TetraBuffers> {
        for tetra in self.geom.tetras() {
            self.push_tetra(tetra);
        }

        for tri in self.geom.tris() {
            self.push_wireframe_tri(tri);
        }

        // Average normals
        for normal in self.normals.values_mut() {
            normal.normalize();
        }

        let mut normal_starts = IdxVec::splat(Vec4::zero(), self.segment_cache.len());
        let mut normal_ends = IdxVec::splat(Vec4::zero(), self.segment_cache.len());

        for ((i, j), segment) in &self.segment_cache {
            normal_starts[*segment] = self.normals[*i];
            normal_ends[*segment] = self.normals[*j];
        }

        // Buffer data
        let element_buffer = ctx.mk_element_buffer(&self.elements, ElementKind::Triangles)?;
        let projected_wireframe_element_buffer =
            ctx.mk_element_buffer(&self.projected_wireframe_elements, ElementKind::Lines)?;
        let animated_wireframe_element_buffer =
            ctx.mk_element_buffer(&self.animated_wireframe_elements, ElementKind::Lines)?;
        let vertex_start_buffer = ctx.mk_buffer(&self.segment_starts.into_raw())?;
        let vertex_end_buffer = ctx.mk_buffer(&self.segment_ends.into_raw())?;
        let wireframe_vertex_buffer = ctx.mk_buffer(&self.wireframe_vertices)?;
        let normal_start_buffer = ctx.mk_buffer(&normal_starts.into_raw())?;
        let normal_end_buffer = ctx.mk_buffer(&normal_ends.into_raw())?;

        Ok(TetraBuffers {
            element_buffer,
            projected_wireframe_element_buffer,
            animated_wireframe_element_buffer,
            vertex_start_buffer,
            vertex_end_buffer,
            wireframe_vertex_buffer,
            normal_start_buffer,
            normal_end_buffer,
        })
    }
}

impl SimplicialGeometry {
    #[inline]
    pub fn buffer_tris(
        &self,
        ctx: &GlCtx,
        segments_per_cyllinder: u8,
    ) -> Result<Vec<TriVertexArrayData>> {
        TriBufferingCtx::new(ctx, self, segments_per_cyllinder).extract_buffers()
    }

    #[inline]
    pub fn buffer_tetras(&self, ctx: &GlCtx) -> Result<TetraBuffers> {
        TetraBufferer::new(self).extract_buffers(ctx)
    }
}
