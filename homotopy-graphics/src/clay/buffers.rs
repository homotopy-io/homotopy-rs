use std::{
    cmp::Ordering,
    collections::HashMap,
    f32::consts::{PI, TAU},
};

use homotopy_common::{
    declare_idx,
    idx::{Idx, IdxVec},
};
use ultraviolet::{Mat3, Vec3, Vec4};

use super::geom::{Mesh, Vert};
use crate::gl::{
    buffer::{Buffer, ElementBuffer, ElementKind},
    GlCtx, Result,
};

pub struct SquareBuffers {
    pub element_buffer: ElementBuffer,
    pub wireframe_element_buffer: ElementBuffer,
    pub vertex_buffer: Buffer<Vec3>,
    pub normal_buffer: Buffer<Vec3>,
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

struct SquareBufferer<'a> {
    mesh: &'a Mesh,
    elements: Vec<u16>,
    verts: Vec<Vec3>,
    wireframe_elements: Vec<u16>,
    normals: Vec<Vec3>,
}

impl<'a> SquareBufferer<'a> {
    fn new(mesh: &'a Mesh) -> Self {
        let len = mesh.squares.len();

        Self {
            mesh,
            verts: mesh.verts.values().map(|v| v.xyz()).collect::<Vec<_>>(),
            elements: Vec::with_capacity(len * 6),
            wireframe_elements: Vec::with_capacity(len * 12),
            normals: vec![Vec3::zero(); mesh.verts.len()],
        }
    }

    fn push_wireframe_element(&mut self, i: usize, j: usize) {
        self.wireframe_elements.push(i as u16);
        self.wireframe_elements.push(j as u16);
    }

    fn push_element(&mut self, i: usize, j: usize, k: usize) {
        self.push_wireframe_element(i, j);
        self.push_wireframe_element(j, k);
        self.push_wireframe_element(k, i);

        self.elements.push(i as u16);
        self.elements.push(j as u16);
        self.elements.push(k as u16);
    }

    fn push_tri(&mut self, i: usize, j: usize, k: usize) {
        if i != j && j != k && k != i {
            self.push_element(i, j, k);

            let v_1 = self.verts[i];
            let v_2 = self.verts[j];
            let v_3 = self.verts[k];
            let n = (v_2 - v_1).cross(v_3 - v_1);

            self.normals[i] += n;
            self.normals[j] += n;
            self.normals[k] += n;
        }
    }

    fn push_vert(&mut self, v: Vec3) -> usize {
        let idx = self.verts.len();
        self.verts.push(v);
        self.normals.push(Vec3::zero());
        idx
    }

    fn push_samples(&mut self, vert: Vert, normal: Vec3, binormal: Vec3, connect: bool) {
        const SAMPLES: usize = 10;
        const RADIUS: f32 = 0.05;

        let len = self.verts.len();

        for j in 0..SAMPLES {
            let theta = j as f32 * TAU / SAMPLES as f32;
            self.push_vert(
                self.verts[vert.index()]
                    + RADIUS * f32::cos(theta) * normal
                    + RADIUS * f32::sin(theta) * binormal,
            );
        }

        if connect {
            for j in 0..SAMPLES as usize {
                let v_0 = len + j;
                let v_1 = len + ((j + 1) % SAMPLES);
                let v_2 = v_0 - SAMPLES;
                let v_3 = v_1 - SAMPLES;

                self.push_tri(v_0, v_2, v_1);
                self.push_tri(v_1, v_2, v_3);
            }
        }
    }

    fn push_curve(&mut self, curve: &[Vert]) {
        // The direction of the curve in the previous segment
        let mut d_0 = (*self.mesh.verts[curve[1]] - *self.mesh.verts[curve[0]])
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

        self.push_samples(curve[0], n, d_0.cross(n), false);

        for i in 2..curve.len() {
            let v_0 = curve[i - 1];
            let v_1 = curve[i];

            let d_1 = (*self.mesh.verts[v_1] - *self.mesh.verts[v_0])
                .xyz()
                .normalized();
            let t = 0.5 * (d_1 + d_0);
            d_0 = d_1;

            n = t.cross(n).cross(t).normalized();
            let bn = t.cross(n).normalized();

            self.push_samples(v_0, n, bn, true);

            if i == curve.len() - 1 {
                self.push_samples(v_1, n, bn, true);
            }
        }
    }

    fn push_point(&mut self, point: Vert) {
        const STACKS: usize = 10;
        const SECTORS: usize = 10;
        const RADIUS: f32 = 0.1;

        let origin = self.mesh.verts[point].xyz();
        let north_pole = self.push_vert(origin + Vec3::unit_y() * RADIUS);
        let south_pole = self.push_vert(origin - Vec3::unit_y() * RADIUS);

        for i in 1..STACKS {
            let theta = 0.5 * PI - (i as f32 * PI / STACKS as f32);
            let xz = RADIUS * f32::cos(theta);
            let y = RADIUS * f32::sin(theta);

            let len = self.verts.len();

            for j in 0..SECTORS {
                let phi = j as f32 * TAU / SECTORS as f32;
                let x = xz * f32::cos(phi);
                let z = xz * f32::sin(phi);
                self.push_vert(origin + Vec3::new(x, y, z));
            }

            if i == 1 {
                for j in 0..SECTORS {
                    let v_0 = len + j;
                    let v_1 = len + (j + 1) % SECTORS;
                    self.push_tri(north_pole, v_1, v_0);
                }
            } else {
                for j in 0..SECTORS {
                    let v_0 = len + j;
                    let v_1 = len + (j + 1) % SECTORS;
                    let v_2 = v_0 - SECTORS;
                    let v_3 = v_1 - SECTORS;

                    self.push_tri(v_0, v_2, v_1);
                    self.push_tri(v_1, v_2, v_3);
                }
            }

            if i == STACKS - 1 {
                for j in 0..SECTORS {
                    let v_0 = len + j;
                    let v_1 = len + (j + 1) % SECTORS;
                    self.push_tri(south_pole, v_0, v_1);
                }
            }
        }
    }

    fn triangulate(&mut self) {
        // Triangulate mesh
        for square in self.mesh.squares.values() {
            // Bottom right triangle
            self.push_tri(square[0].index(), square[1].index(), square[3].index());
            // Top left triangle
            self.push_tri(square[0].index(), square[3].index(), square[2].index());
        }

        for curve in self.mesh.curves.values() {
            self.push_curve(curve);
        }

        for point in self.mesh.points.values() {
            self.push_point(*point);
        }
    }

    fn extract_buffers(mut self, ctx: &GlCtx) -> Result<SquareBuffers> {
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

        Ok(SquareBuffers {
            element_buffer,
            wireframe_element_buffer,
            vertex_buffer,
            normal_buffer,
        })
    }
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
    pub fn buffer_squares(&self, ctx: &GlCtx) -> Result<SquareBuffers> {
        let mut bufferer = SquareBufferer::new(self);
        bufferer.triangulate();
        bufferer.extract_buffers(ctx)
    }

    pub fn buffer_cubes(&self, ctx: &GlCtx) -> Result<CubeBuffers> {
        let mut bufferer = CubeBufferer::new(self);
        bufferer.triangulate();
        bufferer.extract_buffers(ctx)
    }
}
