use std::{cmp::Ordering, collections::HashMap};

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

// TODO(@doctorn) remove line buffers (by making tubes)
// TODO(@doctorn) chunk meshes

pub struct LineBuffers {
    pub element_buffer: ElementBuffer,
    pub vertex_buffer: Buffer<Vec3>,
}

pub struct SquareBuffers {
    pub element_buffer: ElementBuffer,
    pub wireframe_element_buffer: ElementBuffer,
    pub wireframe_color_buffer: Buffer<Vec3>,
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

struct LineBufferer<'a> {
    mesh: &'a Mesh,
    elements: Vec<u16>,
}

impl<'a> LineBufferer<'a> {
    fn new(mesh: &'a Mesh) -> Self {
        Self {
            mesh,
            elements: Vec::with_capacity(mesh.lines.len()),
        }
    }

    fn triangulate(&mut self) {
        for line in self.mesh.lines.values() {
            self.elements.push(line[0].index() as u16);
            self.elements.push(line[1].index() as u16);
        }
    }

    fn extract_buffers(self, ctx: &GlCtx) -> Result<LineBuffers> {
        let vertices = self
            .mesh
            .verts
            .values()
            .map(|v| v.xyz())
            .collect::<Vec<_>>();

        let element_buffer = ctx.mk_element_buffer(&self.elements, ElementKind::Lines)?;
        let vertex_buffer = ctx.mk_buffer(&vertices)?;

        Ok(LineBuffers {
            element_buffer,
            vertex_buffer,
        })
    }
}

struct SquareBufferer<'a> {
    mesh: &'a Mesh,
    elements: Vec<u16>,
    wireframe_elements: Vec<u16>,
    normals: IdxVec<Vert, Vec3>,
}

impl<'a> SquareBufferer<'a> {
    fn new(mesh: &'a Mesh) -> Self {
        let len = mesh.squares.len();

        Self {
            mesh,
            elements: Vec::with_capacity(len * 6),
            wireframe_elements: Vec::with_capacity(len * 12),
            normals: IdxVec::splat(Vec3::zero(), mesh.verts.len()),
        }
    }

    fn push_wireframe_element(&mut self, i: Vert, j: Vert) {
        self.wireframe_elements.push(i.index() as u16);
        self.wireframe_elements.push(j.index() as u16);
    }

    fn push_element(&mut self, i: Vert, j: Vert, k: Vert) {
        self.push_wireframe_element(i, j);
        self.push_wireframe_element(j, k);
        self.push_wireframe_element(k, i);

        self.elements.push(i.index() as u16);
        self.elements.push(j.index() as u16);
        self.elements.push(k.index() as u16);
    }

    fn push_tri(&mut self, i: Vert, j: Vert, k: Vert) {
        if i != j && j != k && k != i {
            self.push_element(i, j, k);

            let v_1 = self.mesh.verts[i].xyz();
            let v_2 = self.mesh.verts[j].xyz();
            let v_3 = self.mesh.verts[k].xyz();
            let n = (v_2 - v_1).cross(v_3 - v_1);

            self.normals[i] += n;
            self.normals[j] += n;
            self.normals[k] += n;
        }
    }

    fn triangulate(&mut self) {
        // Triangulate mesh
        for square in self.mesh.squares.values() {
            // Bottom right triangle
            self.push_tri(square[0], square[1], square[3]);
            // Top left triangle
            self.push_tri(square[0], square[3], square[2]);
        }
    }

    fn extract_buffers(mut self, ctx: &GlCtx) -> Result<SquareBuffers> {
        // Average normals
        for normal in self.normals.values_mut() {
            normal.normalize();
        }

        let verts = self
            .mesh
            .verts
            .values()
            .map(|v| v.xyz())
            .collect::<Vec<_>>();
        let boundary_colors = self
            .mesh
            .verts
            .values()
            .map(|v| v.boundary.debug_color())
            .collect::<Vec<_>>();

        // Buffer data
        let element_buffer = ctx.mk_element_buffer(&self.elements, ElementKind::Triangles)?;
        let wireframe_element_buffer =
            ctx.mk_element_buffer(&self.wireframe_elements, ElementKind::Lines)?;
        let wireframe_color_buffer = ctx.mk_buffer(&boundary_colors)?;
        let vertex_buffer = ctx.mk_buffer(&verts)?;
        let normal_buffer = ctx.mk_buffer(&self.normals.into_raw())?;

        Ok(SquareBuffers {
            element_buffer,
            wireframe_element_buffer,
            wireframe_color_buffer,
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
    pub fn buffer_lines(&self, ctx: &GlCtx) -> Result<LineBuffers> {
        let mut bufferer = LineBufferer::new(self);
        bufferer.triangulate();
        bufferer.extract_buffers(ctx)
    }

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
