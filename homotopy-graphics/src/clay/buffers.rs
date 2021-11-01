use std::{
    cmp::Ordering,
    collections::HashMap,
    f32::consts::{PI, TAU},
    mem,
};

use homotopy_common::{
    declare_idx,
    idx::{Idx, IdxVec},
};
use homotopy_core::Generator;
use ultraviolet::{Mat3, Vec3, Vec4};

use super::geom::{Mesh, Vert};
use crate::gl::{
    array::VAO_LIMIT,
    buffer::{Buffer, ElementBuffer, ElementKind},
    GlCtx, Result,
};

const SPHERE_STACKS: u16 = 10;
const SPHERE_SECTORS: u16 = 10;
const SPHERE_RADIUS: f32 = 0.1;

const TUBE_RADIUS: f32 = 0.05;
const TUBE_SECTORS: u16 = 10;

pub struct SquareVertexArrayData {
    pub element_buffer: ElementBuffer,
    pub wireframe_element_buffer: ElementBuffer,
    pub vertex_buffer: Buffer<Vec3>,
    pub normal_buffer: Buffer<Vec3>,
    pub generator: Generator,
}

struct SquareBufferingState {
    elements: Vec<u16>,
    verts: Vec<Vec3>,
    mapping: HashMap<Vert, u16>,
    wireframe_elements: Vec<u16>,
    normals: Vec<Vec3>,
}

struct SquareBufferingCtx<'a> {
    ctx: &'a GlCtx,
    mesh: &'a Mesh,
    state: HashMap<Generator, SquareBufferingState>,
    complete_arrays: Vec<SquareVertexArrayData>,
}

impl<'a> SquareBufferingCtx<'a> {
    fn new(ctx: &'a GlCtx, mesh: &'a Mesh) -> Self {
        Self {
            ctx,
            mesh,
            state: Default::default(),
            complete_arrays: vec![],
        }
    }

    // FIXME(@doctorn) this shouldn't need to know how many vertices are required
    // up front - should handle 'overflow errors' gracefully and chop meshes
    // along these boundaries (copying any duplicated normal data?)
    fn with_state<F, U>(&mut self, generator: Generator, required: usize, f: F) -> Result<U>
    where
        F: FnOnce(&mut SquareBufferingState) -> U,
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

    fn push_square(&mut self, square: &[Vert; 4]) -> Result<()> {
        let generator = square
            .iter()
            .map(|v| self.mesh.verts[*v].generator)
            .min_by_key(|g| g.dimension)
            .unwrap();
        let mesh = self.mesh;

        self.with_state(generator, 4, |state| {
            let v_0 = state.push_vert(mesh, square[0]);
            let v_1 = state.push_vert(mesh, square[1]);
            let v_2 = state.push_vert(mesh, square[2]);
            let v_3 = state.push_vert(mesh, square[3]);

            // Bottom right triangle
            state.push_tri(v_0, v_1, v_3);
            state.push_tri(v_0, v_3, v_2);
        })
    }

    fn push_curve(&mut self, generator: Generator, curve: &[Vert]) -> Result<()> {
        let mesh = self.mesh;

        self.with_state(generator, curve.len() * TUBE_SECTORS as usize, |state| {
            // The direction of the curve in the previous segment
            let mut d_0 = (*mesh.verts[curve[1]] - *mesh.verts[curve[0]])
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

            state.push_tube_sector(mesh.verts[curve[0]].xyz(), n, d_0.cross(n), false);

            for i in 2..curve.len() {
                let v_0 = mesh.verts[curve[i - 1]].xyz();
                let v_1 = mesh.verts[curve[i]].xyz();

                let d_1 = (v_1 - v_0).normalized();
                let t = 0.5 * (d_1 + d_0);
                d_0 = d_1;

                n = t.cross(n).cross(t).normalized();
                let bn = t.cross(n).normalized();

                state.push_tube_sector(v_0, n, bn, true);

                if i == curve.len() - 1 {
                    state.push_tube_sector(v_1, n, bn, true);
                }
            }
        })
    }

    fn push_point(&mut self, point: Vert) -> Result<()> {
        let mesh = self.mesh;
        let generator = mesh.verts[point].generator;

        self.with_state(
            generator,
            (SPHERE_STACKS * SPHERE_SECTORS) as usize + 2,
            |state| {
                let origin = mesh.verts[point].xyz();
                let north_pole = state.push_unmapped_vert(origin + Vec3::unit_y() * SPHERE_RADIUS);
                let south_pole = state.push_unmapped_vert(origin - Vec3::unit_y() * SPHERE_RADIUS);

                for i in 1..SPHERE_STACKS {
                    let theta = 0.5 * PI - (f32::from(i) * PI / f32::from(SPHERE_STACKS));
                    let xz = SPHERE_RADIUS * f32::cos(theta);
                    let y = SPHERE_RADIUS * f32::sin(theta);

                    let len = state.verts.len() as u16;

                    for j in 0..SPHERE_SECTORS {
                        let phi = f32::from(j) * TAU / f32::from(SPHERE_SECTORS);
                        let x = xz * f32::cos(phi);
                        let z = xz * f32::sin(phi);
                        state.push_unmapped_vert(origin + Vec3::new(x, y, z));
                    }

                    if i == 1 {
                        for j in 0..SPHERE_SECTORS {
                            let v_0 = len + j;
                            let v_1 = len + (j + 1) % SPHERE_SECTORS;
                            state.push_tri(north_pole, v_1, v_0);
                        }
                    } else {
                        for j in 0..SPHERE_SECTORS {
                            let v_0 = len + j;
                            let v_1 = len + (j + 1) % SPHERE_SECTORS;
                            let v_2 = v_0 - SPHERE_SECTORS;
                            let v_3 = v_1 - SPHERE_SECTORS;

                            state.push_tri(v_0, v_2, v_1);
                            state.push_tri(v_1, v_2, v_3);
                        }
                    }

                    if i == SPHERE_STACKS - 1 {
                        for j in 0..SPHERE_SECTORS {
                            let v_0 = len + j;
                            let v_1 = len + (j + 1) % SPHERE_SECTORS;
                            state.push_tri(south_pole, v_0, v_1);
                        }
                    }
                }
            },
        )
    }

    fn triangulate(&mut self) -> Result<()> {
        // Triangulate mesh
        for square in self.mesh.squares.values() {
            self.push_square(square)?;
        }

        for curve in self.mesh.curves.values() {
            self.push_curve(curve.generator, &*curve)?;
        }

        for point in self.mesh.points.values() {
            self.push_point(*point)?;
        }

        Ok(())
    }

    fn extract_buffers(mut self) -> Result<Vec<SquareVertexArrayData>> {
        for (generator, state) in self.state {
            self.complete_arrays
                .push(state.extract_buffers(self.ctx, generator)?);
        }

        Ok(self.complete_arrays)
    }
}

impl SquareBufferingState {
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

    fn push_vert(&mut self, mesh: &Mesh, v: Vert) -> u16 {
        if let Some(&idx) = self.mapping.get(&v) {
            return idx;
        }

        let idx = self.push_unmapped_vert(mesh.verts[v].xyz());
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

    fn push_tube_sector(&mut self, vert: Vec3, normal: Vec3, binormal: Vec3, connect: bool) {
        let len = self.verts.len() as u16;

        for j in 0..TUBE_SECTORS {
            let theta = f32::from(j) * TAU / f32::from(TUBE_SECTORS);
            self.push_unmapped_vert(
                vert + TUBE_RADIUS * f32::cos(theta) * normal
                    + TUBE_RADIUS * f32::sin(theta) * binormal,
            );
        }

        if connect {
            for j in 0..TUBE_SECTORS {
                let v_0 = len + j;
                let v_1 = len + ((j + 1) % TUBE_SECTORS);
                let v_2 = v_0 - TUBE_SECTORS;
                let v_3 = v_1 - TUBE_SECTORS;

                self.push_tri(v_0, v_2, v_1);
                self.push_tri(v_1, v_2, v_3);
            }
        }
    }

    fn extract_buffers(
        mut self,
        ctx: &GlCtx,
        generator: Generator,
    ) -> Result<SquareVertexArrayData> {
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

        Ok(SquareVertexArrayData {
            element_buffer,
            wireframe_element_buffer,
            vertex_buffer,
            normal_buffer,
            generator,
        })
    }
}

impl Default for SquareBufferingState {
    fn default() -> Self {
        Self {
            elements: Vec::with_capacity(VAO_LIMIT),
            verts: Vec::with_capacity(VAO_LIMIT),
            mapping: Default::default(),
            wireframe_elements: Vec::with_capacity(VAO_LIMIT),
            normals: Vec::with_capacity(VAO_LIMIT),
        }
    }
}

pub struct CubeBuffers {
    pub element_buffer: ElementBuffer,
    pub wireframe_element_buffer: ElementBuffer,
    pub vertex_start_buffer: Buffer<Vec4>,
    pub vertex_end_buffer: Buffer<Vec4>,
    pub wireframe_vertex_buffer: Buffer<Vec3>,
    pub normal_start_buffer: Buffer<Vec4>,
    pub normal_end_buffer: Buffer<Vec4>,
}

declare_idx! { struct Segment = u16; }

struct CubeBufferer<'a> {
    mesh: &'a Mesh,

    segment_cache: HashMap<(Vert, Vert), Segment>,
    elements: Vec<u16>,
    wireframe_elements: Vec<u16>,

    segment_starts: IdxVec<Segment, Vec4>,
    segment_ends: IdxVec<Segment, Vec4>,
    normals: IdxVec<Vert, Vec4>,

    wireframe_vertices: Vec<Vec3>,
}

impl<'a> CubeBufferer<'a> {
    fn new(mesh: &'a Mesh) -> Self {
        let len = mesh.cubes.len();
        let segments = 30 * len;

        Self {
            mesh,

            segment_cache: HashMap::new(),
            elements: Vec::with_capacity(segments),
            wireframe_elements: Vec::with_capacity(len * 8),

            segment_starts: IdxVec::with_capacity(segments),
            segment_ends: IdxVec::with_capacity(segments),
            normals: IdxVec::splat(Vec4::zero(), mesh.verts.len()),

            wireframe_vertices: mesh.verts.values().map(|v| v.xyz()).collect(),
        }
    }

    fn mk_segment_uncached(&mut self, i: Vert, j: Vert) -> Segment {
        self.wireframe_elements.push(i.index() as u16);
        self.wireframe_elements.push(j.index() as u16);

        let segment = self.segment_starts.push(*self.mesh.verts[i]);
        self.segment_ends.push(*self.mesh.verts[j]);

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
            self.elements.push(i.index() as u16);
            self.elements.push(j.index() as u16);
            self.elements.push(k.index() as u16);
        }
    }

    fn push_tetra(&mut self, i: Vert, j: Vert, k: Vert, l: Vert) {
        #[inline]
        fn parity_sort<F>(vertices: &mut [Vert; 4], f: F) -> bool
        where
            F: Fn(Vert, Vert) -> Ordering,
        {
            let mut parity = true;

            macro_rules! bubble_pass  {
                ($($i:literal),*$(,)*) => {
                    $(if f(vertices[$i + 1], vertices[$i]) == Ordering::Less {
                        vertices.swap($i, $i + 1);
                        parity = !parity;
                    })*
                };
            }

            bubble_pass!(
                0, 1, 2, // First pass finds top element
                0, 1, // Second pass finds next element
                0, // Final pass orders remaining two elements
            );

            parity
        }

        let ([i, j, k, l], parity) = {
            let mut verts = [i, j, k, l];
            let parity = parity_sort(&mut verts, |i, j| {
                self.mesh.verts[i]
                    .w
                    .partial_cmp(&self.mesh.verts[j].w)
                    .unwrap()
            });
            (verts, parity)
        };

        let ij = self.mk_segment(i, j);
        let ik = self.mk_segment(i, k);
        let il = self.mk_segment(i, l);
        let jk = self.mk_segment(j, k);
        let jl = self.mk_segment(j, l);
        let kl = self.mk_segment(k, l);

        {
            let origin = *self.mesh.verts[i];
            let v_0 = *self.mesh.verts[j] - origin;
            let v_1 = *self.mesh.verts[k] - origin;
            let v_2 = *self.mesh.verts[l] - origin;

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

    fn triangulate(&mut self) {
        // Triangulate mesh
        for cube in self.mesh.cubes.values() {
            self.push_tetra(cube[1], cube[4], cube[5], cube[7]);
            self.push_tetra(cube[0], cube[4], cube[1], cube[2]);
            self.push_tetra(cube[1], cube[7], cube[3], cube[2]);
            self.push_tetra(cube[4], cube[6], cube[7], cube[2]);
            self.push_tetra(cube[1], cube[7], cube[2], cube[4]);
        }
    }

    fn extract_buffers(mut self, ctx: &GlCtx) -> Result<CubeBuffers> {
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
        let wireframe_element_buffer =
            ctx.mk_element_buffer(&self.wireframe_elements, ElementKind::Lines)?;
        let vertex_start_buffer = ctx.mk_buffer(&self.segment_starts.into_raw())?;
        let vertex_end_buffer = ctx.mk_buffer(&self.segment_ends.into_raw())?;
        let wireframe_vertex_buffer = ctx.mk_buffer(&self.wireframe_vertices)?;
        let normal_start_buffer = ctx.mk_buffer(&normal_starts.into_raw())?;
        let normal_end_buffer = ctx.mk_buffer(&normal_ends.into_raw())?;

        Ok(CubeBuffers {
            element_buffer,
            wireframe_element_buffer,
            vertex_start_buffer,
            vertex_end_buffer,
            wireframe_vertex_buffer,
            normal_start_buffer,
            normal_end_buffer,
        })
    }
}

impl Mesh {
    pub fn buffer_squares(&self, ctx: &GlCtx) -> Result<Vec<SquareVertexArrayData>> {
        let mut ctx = SquareBufferingCtx::new(ctx, self);
        ctx.triangulate()?;
        ctx.extract_buffers()
    }

    pub fn buffer_cubes(&self, ctx: &GlCtx) -> Result<CubeBuffers> {
        let mut bufferer = CubeBufferer::new(self);
        bufferer.triangulate();
        bufferer.extract_buffers(ctx)
    }
}
