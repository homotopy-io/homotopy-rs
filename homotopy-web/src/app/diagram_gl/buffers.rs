use std::{hash::Hash, mem};

use homotopy_common::{hash::FastHashMap, idx::IdxVec, parity};
use homotopy_core::Diagram0;
use homotopy_gl::{
    array::VAO_LIMIT,
    buffer::{Buffer, ElementBuffer, ElementKind},
    GlCtx, Result,
};
use homotopy_graphics::geom::{SimplicialGeometry, Vert};
use ultraviolet::{Vec3, Vec4};

trait BuffererState: Sized {
    type VertexData;

    fn alloc() -> Self;

    fn push_vert(&mut self, v: u16, data: Self::VertexData);
}

trait Bufferer: Sized {
    type Vertex: Eq + Hash;
    type Output;
    type State: BuffererState;
    type Key: Copy + Eq + Hash;

    fn new(geom: &SimplicialGeometry) -> Self;

    fn buffer(ctx: &mut BufferingCtx<Self>) -> Result<()>;

    fn commit(ctx: &GlCtx, key: Self::Key, completed: State<Self>) -> Result<Self::Output>;
}

type VertexData<B> = <<B as Bufferer>::State as BuffererState>::VertexData;

#[derive(Default)]
struct State<B: Bufferer> {
    mapping: FastHashMap<B::Vertex, u16>,
    inner: B::State,
}

struct BufferingCtx<'a, B: Bufferer> {
    ctx: &'a GlCtx,
    geom: &'a SimplicialGeometry,
    global_state: B,
    local_state: FastHashMap<B::Key, State<B>>,
    complete: Vec<B::Output>,
}

impl<B> State<B>
where
    B: Bufferer,
{
    fn new() -> Self {
        Self {
            mapping: FastHashMap::with_capacity_and_hasher(VAO_LIMIT, Default::default()),
            inner: B::State::alloc(),
        }
    }

    fn push_vert(&mut self, v: B::Vertex, data: VertexData<B>) -> u16 {
        if let Some(&idx) = self.mapping.get(&v) {
            return idx;
        }

        let idx = self.mapping.len() as u16;
        self.mapping.insert(v, idx);
        self.inner.push_vert(idx, data);
        idx
    }
}

impl<'a, B> BufferingCtx<'a, B>
where
    B: Bufferer,
{
    fn new(ctx: &'a GlCtx, geom: &'a SimplicialGeometry) -> Self {
        Self {
            ctx,
            geom,
            global_state: B::new(geom),
            local_state: Default::default(),
            complete: Default::default(),
        }
    }

    // FIXME(@doctorn) this shouldn't need to know how many vertices are required
    // up front - should handle 'overflow errors' gracefully and chop geometries
    // along these boundaries (copying any duplicated normal data?)
    fn with_state<F, U>(&mut self, key: B::Key, required: usize, f: F) -> Result<U>
    where
        F: FnOnce(&B, &mut State<B>) -> U,
    {
        let state = self.local_state.entry(key).or_insert_with(State::new);

        if state.mapping.len() + required > VAO_LIMIT {
            let mut completed = State::new();
            mem::swap(state, &mut completed);
            self.complete.push(B::commit(self.ctx, key, completed)?);
        }

        Ok(f(&self.global_state, state))
    }

    fn extract_buffers(mut self) -> Result<Vec<B::Output>> {
        B::buffer(&mut self)?;

        for (generator, state) in self.local_state {
            self.complete.push(B::commit(self.ctx, generator, state)?);
        }

        Ok(self.complete)
    }
}

struct TriBufferer {
    normals: IdxVec<Vert, Vec3>,
}

struct TriBufferingState {
    verts: IdxVec<u16, Vec3>,
    normals: IdxVec<u16, Vec3>,
    elements: Vec<u16>,
    wireframe_elements: Vec<u16>,
}

pub struct TriVertexArrayData {
    pub element_buffer: ElementBuffer,
    pub wireframe_element_buffer: ElementBuffer,
    pub vertex_buffer: Buffer<Vec3>,
    pub normal_buffer: Buffer<Vec3>,
    pub generator: Diagram0,
    pub k: usize,
}

impl BuffererState for TriBufferingState {
    type VertexData = (Vec3, Vec3);

    fn alloc() -> Self {
        Self {
            verts: IdxVec::with_capacity(VAO_LIMIT),
            normals: IdxVec::with_capacity(VAO_LIMIT),
            elements: Vec::with_capacity(VAO_LIMIT),
            wireframe_elements: Vec::with_capacity(VAO_LIMIT),
        }
    }

    fn push_vert(&mut self, v: u16, data: Self::VertexData) {
        let i = self.verts.push(data.0);
        let j = self.normals.push(data.1);

        debug_assert_eq!(i, v);
        debug_assert_eq!(j, v);
    }
}

impl Bufferer for TriBufferer {
    type Vertex = Vert;
    type Output = TriVertexArrayData;
    type State = TriBufferingState;
    type Key = (Diagram0, usize);

    fn new(geom: &SimplicialGeometry) -> Self {
        Self {
            normals: geom.compute_normals_3d(),
        }
    }

    fn buffer(ctx: &mut BufferingCtx<Self>) -> Result<()> {
        for (tri, parity) in ctx.geom.areas.values().copied() {
            let geom = ctx.geom;
            let generator = geom.verts[tri[0]].generator;
            let k = geom.verts[tri[0]].k;

            ctx.with_state((generator, k), 3, |global, local| {
                let v_0 = local.push_vert(
                    tri[0],
                    (geom.verts[tri[0]].position.xyz(), global.normals[tri[0]]),
                );
                let v_1 = local.push_vert(
                    tri[1],
                    (geom.verts[tri[1]].position.xyz(), global.normals[tri[1]]),
                );
                let v_2 = local.push_vert(
                    tri[2],
                    (geom.verts[tri[2]].position.xyz(), global.normals[tri[2]]),
                );

                if parity.is_even() {
                    local.inner.push_element(v_0, v_1, v_2);
                } else {
                    local.inner.push_element(v_2, v_1, v_0);
                }
            })?;
        }

        Ok(())
    }

    fn commit(
        ctx: &GlCtx,
        (generator, k): Self::Key,
        completed: State<Self>,
    ) -> Result<Self::Output> {
        let element_buffer =
            ctx.mk_element_buffer(&completed.inner.elements, ElementKind::Triangles)?;
        let wireframe_element_buffer =
            ctx.mk_element_buffer(&completed.inner.wireframe_elements, ElementKind::Lines)?;
        let vertex_buffer = ctx.mk_buffer(&completed.inner.verts.into_raw())?;
        let normal_buffer = ctx.mk_buffer(&completed.inner.normals.into_raw())?;

        Ok(TriVertexArrayData {
            element_buffer,
            wireframe_element_buffer,
            vertex_buffer,
            normal_buffer,
            generator,
            k,
        })
    }
}

impl TriBufferingState {
    fn push_wireframe_element(&mut self, i: u16, j: u16) {
        self.wireframe_elements.push(i);
        self.wireframe_elements.push(j);
    }

    fn push_element(&mut self, i: u16, j: u16, k: u16) {
        if i != j && j != k && k != i {
            self.push_wireframe_element(i, j);
            self.push_wireframe_element(j, k);
            self.push_wireframe_element(k, i);

            self.elements.push(i);
            self.elements.push(j);
            self.elements.push(k);
        }
    }
}

struct TetraBufferer {
    normals: IdxVec<Vert, Vec4>,
}

struct TetraBufferingState {
    elements: Vec<u16>,

    vert_starts: IdxVec<u16, Vec4>,
    vert_ends: IdxVec<u16, Vec4>,
    normal_starts: IdxVec<u16, Vec4>,
    normal_ends: IdxVec<u16, Vec4>,
}

pub struct TetraVertexArrayData {
    pub generator: Diagram0,
    pub element_buffer: ElementBuffer,
    pub k: usize,

    pub vert_start_buffer: Buffer<Vec4>,
    pub vert_end_buffer: Buffer<Vec4>,
    pub normal_start_buffer: Buffer<Vec4>,
    pub normal_end_buffer: Buffer<Vec4>,
}

struct PseudoVertData {
    vert_start: Vec4,
    vert_end: Vec4,
    normal_start: Vec4,
    normal_end: Vec4,
}

impl BuffererState for TetraBufferingState {
    type VertexData = PseudoVertData;

    fn alloc() -> Self {
        Self {
            elements: Vec::with_capacity(VAO_LIMIT),

            vert_starts: IdxVec::with_capacity(VAO_LIMIT),
            vert_ends: IdxVec::with_capacity(VAO_LIMIT),
            normal_starts: IdxVec::with_capacity(VAO_LIMIT),
            normal_ends: IdxVec::with_capacity(VAO_LIMIT),
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn push_vert(&mut self, v: u16, data: Self::VertexData) {
        let i = self.vert_starts.push(data.vert_start);
        let j = self.vert_ends.push(data.vert_end);
        let k = self.normal_starts.push(data.normal_start);
        let l = self.normal_ends.push(data.normal_end);

        debug_assert_eq!(i, v);
        debug_assert_eq!(j, v);
        debug_assert_eq!(k, v);
        debug_assert_eq!(l, v);
    }
}

impl Bufferer for TetraBufferer {
    type Vertex = (Vert, Vert);
    type Output = TetraVertexArrayData;
    type State = TetraBufferingState;
    type Key = (Diagram0, usize);

    fn new(geom: &SimplicialGeometry) -> Self {
        Self {
            normals: geom.compute_normals_4d(),
        }
    }

    fn buffer(ctx: &mut BufferingCtx<Self>) -> Result<()> {
        for (mut tetra, parity) in ctx.geom.volumes.values().copied() {
            let geom = ctx.geom;
            let generator = geom.verts[tetra[0]].generator;
            let k = geom.verts[tetra[0]].k;

            ctx.with_state((generator, k), 6, |global, local| {
                let parity =
                    parity * parity::sort_4(&mut tetra, |i, j| geom.time_order(i, j)).into();
                let [i, j, k, l] = tetra;

                let mut push_vert = |i: Vert, j: Vert| {
                    local.push_vert(
                        (i, j),
                        PseudoVertData {
                            vert_start: geom.verts[i].position,
                            vert_end: geom.verts[j].position,
                            normal_start: global.normals[i],
                            normal_end: global.normals[j],
                        },
                    )
                };

                let ij = push_vert(i, j);
                let ik = push_vert(i, k);
                let il = push_vert(i, l);
                let jk = push_vert(j, k);
                let jl = push_vert(j, l);
                let kl = push_vert(k, l);

                if parity.is_even() {
                    local.inner.push_tri(ij, il, ik);
                    local.inner.push_tri(jl, ik, jk);
                    local.inner.push_tri(jl, il, ik);
                    local.inner.push_tri(jl, il, kl);
                } else {
                    local.inner.push_tri(il, ij, ik);
                    local.inner.push_tri(ik, jl, jk);
                    local.inner.push_tri(il, jl, ik);
                    local.inner.push_tri(il, jl, kl);
                }
            })?;
        }

        Ok(())
    }

    fn commit(
        ctx: &GlCtx,
        (generator, k): Self::Key,
        completed: State<Self>,
    ) -> Result<Self::Output> {
        let element_buffer =
            ctx.mk_element_buffer(&completed.inner.elements, ElementKind::Triangles)?;
        let vert_start_buffer = ctx.mk_buffer(&completed.inner.vert_starts.into_raw())?;
        let vert_end_buffer = ctx.mk_buffer(&completed.inner.vert_ends.into_raw())?;
        let normal_start_buffer = ctx.mk_buffer(&completed.inner.normal_starts.into_raw())?;
        let normal_end_buffer = ctx.mk_buffer(&completed.inner.normal_ends.into_raw())?;

        Ok(TetraVertexArrayData {
            generator,
            element_buffer,
            k,
            vert_start_buffer,
            vert_end_buffer,
            normal_start_buffer,
            normal_end_buffer,
        })
    }
}

impl TetraBufferingState {
    fn push_tri(&mut self, i: u16, j: u16, k: u16) {
        if i != j && j != k && k != i {
            self.elements.push(i);
            self.elements.push(j);
            self.elements.push(k);
        }
    }
}

struct ProjectedWireBufferer;

struct ProjectedWireBufferingState {
    elements: Vec<u16>,
    verts: IdxVec<u16, Vec3>,
}

pub struct ProjectedWireArrayData {
    pub element_buffer: ElementBuffer,
    pub vert_buffer: Buffer<Vec3>,
}

impl BuffererState for ProjectedWireBufferingState {
    type VertexData = Vec3;

    fn alloc() -> Self {
        Self {
            verts: IdxVec::with_capacity(VAO_LIMIT),
            elements: Vec::with_capacity(VAO_LIMIT),
        }
    }

    fn push_vert(&mut self, v: u16, data: Self::VertexData) {
        let i = self.verts.push(data);

        debug_assert_eq!(i, v);
    }
}

impl Bufferer for ProjectedWireBufferer {
    type Vertex = Vert;
    type Output = ProjectedWireArrayData;
    type State = ProjectedWireBufferingState;
    type Key = ();

    fn new(_geom: &SimplicialGeometry) -> Self {
        Self
    }

    fn buffer(ctx: &mut BufferingCtx<Self>) -> Result<()> {
        for (tri, _) in ctx.geom.areas.values().copied() {
            let geom = ctx.geom;

            ctx.with_state((), 3, |_, local| {
                let v_0 = local.push_vert(tri[0], geom.verts[tri[0]].position.xyz());
                let v_1 = local.push_vert(tri[1], geom.verts[tri[1]].position.xyz());
                let v_2 = local.push_vert(tri[2], geom.verts[tri[2]].position.xyz());

                local.inner.push_tri(v_0, v_1, v_2);
            })?;
        }

        for (tetra, _) in ctx.geom.volumes.values().copied() {
            let geom = ctx.geom;

            ctx.with_state((), 4, |_, local| {
                let v_0 = local.push_vert(tetra[0], geom.verts[tetra[0]].position.xyz());
                let v_1 = local.push_vert(tetra[1], geom.verts[tetra[1]].position.xyz());
                let v_2 = local.push_vert(tetra[2], geom.verts[tetra[2]].position.xyz());
                let v_3 = local.push_vert(tetra[3], geom.verts[tetra[3]].position.xyz());

                local.inner.push_tetra(v_0, v_1, v_2, v_3);
            })?;
        }

        Ok(())
    }

    fn commit(ctx: &GlCtx, (): Self::Key, completed: State<Self>) -> Result<Self::Output> {
        let element_buffer =
            ctx.mk_element_buffer(&completed.inner.elements, ElementKind::Lines)?;
        let vert_buffer = ctx.mk_buffer(&completed.inner.verts.into_raw())?;

        Ok(ProjectedWireArrayData {
            element_buffer,
            vert_buffer,
        })
    }
}

impl ProjectedWireBufferingState {
    fn push_tri(&mut self, i: u16, j: u16, k: u16) {
        self.elements.push(i);
        self.elements.push(j);

        self.elements.push(j);
        self.elements.push(k);

        self.elements.push(k);
        self.elements.push(i);
    }

    fn push_tetra(&mut self, i: u16, j: u16, k: u16, l: u16) {
        self.elements.push(i);
        self.elements.push(j);

        self.elements.push(j);
        self.elements.push(k);

        self.elements.push(k);
        self.elements.push(l);

        self.elements.push(i);
        self.elements.push(l);
    }
}

struct CylinderWireBufferer;

struct CylinderWireBufferingState {
    elements: Vec<u16>,
    vert_starts: IdxVec<u16, Vec4>,
    vert_ends: IdxVec<u16, Vec4>,
}

pub struct CylinderWireArrayData {
    pub generator: Diagram0,
    pub element_buffer: ElementBuffer,
    pub vert_start_buffer: Buffer<Vec4>,
    pub vert_end_buffer: Buffer<Vec4>,
}

impl BuffererState for CylinderWireBufferingState {
    type VertexData = (Vec4, Vec4);

    fn alloc() -> Self {
        Self {
            vert_starts: IdxVec::with_capacity(VAO_LIMIT),
            vert_ends: IdxVec::with_capacity(VAO_LIMIT),
            elements: Vec::with_capacity(VAO_LIMIT),
        }
    }

    fn push_vert(&mut self, v: u16, data: Self::VertexData) {
        let i = self.vert_starts.push(data.0);
        let j = self.vert_ends.push(data.1);

        debug_assert_eq!(i, v);
        debug_assert_eq!(j, v);
    }
}

impl Bufferer for CylinderWireBufferer {
    type Vertex = (Vert, Vert);
    type Output = CylinderWireArrayData;
    type State = CylinderWireBufferingState;
    type Key = Diagram0;

    fn new(_geom: &SimplicialGeometry) -> Self {
        Self
    }

    fn buffer(ctx: &mut BufferingCtx<Self>) -> Result<()> {
        for (mut tri, _) in ctx.geom.areas.values().copied() {
            let geom = ctx.geom;
            let generator = geom.verts[tri[0]].generator;

            ctx.with_state(generator, 3, |_, local| {
                parity::sort_3(&mut tri, |i, j| geom.time_order(i, j));
                let [i, j, k] = tri;

                let mut push_vert = |i: Vert, j: Vert| {
                    local.push_vert((i, j), (geom.verts[i].position, geom.verts[j].position))
                };

                let ij = push_vert(i, j);
                let ik = push_vert(i, k);
                let jk = push_vert(j, k);

                if ij != ik && ik != jk {
                    local.inner.elements.push(ij);
                    local.inner.elements.push(ik);
                    local.inner.elements.push(jk);
                    local.inner.elements.push(ik);
                }
            })?;
        }

        Ok(())
    }

    fn commit(ctx: &GlCtx, generator: Self::Key, completed: State<Self>) -> Result<Self::Output> {
        let element_buffer =
            ctx.mk_element_buffer(&completed.inner.elements, ElementKind::Lines)?;
        let vert_start_buffer = ctx.mk_buffer(&completed.inner.vert_starts.into_raw())?;
        let vert_end_buffer = ctx.mk_buffer(&completed.inner.vert_ends.into_raw())?;

        Ok(CylinderWireArrayData {
            generator,
            element_buffer,
            vert_start_buffer,
            vert_end_buffer,
        })
    }
}

#[inline]
pub fn buffer_tris(g: &SimplicialGeometry, ctx: &GlCtx) -> Result<Vec<TriVertexArrayData>> {
    BufferingCtx::<TriBufferer>::new(ctx, g).extract_buffers()
}

#[inline]
pub fn buffer_projected_wireframe(
    g: &SimplicialGeometry,
    ctx: &GlCtx,
) -> Result<Vec<ProjectedWireArrayData>> {
    BufferingCtx::<ProjectedWireBufferer>::new(ctx, g).extract_buffers()
}

#[inline]
pub fn buffer_cylinder_wireframe(
    g: &SimplicialGeometry,
    ctx: &GlCtx,
) -> Result<Vec<CylinderWireArrayData>> {
    BufferingCtx::<CylinderWireBufferer>::new(ctx, g).extract_buffers()
}

#[inline]
pub fn buffer_tetras(g: &SimplicialGeometry, ctx: &GlCtx) -> Result<Vec<TetraVertexArrayData>> {
    BufferingCtx::<TetraBufferer>::new(ctx, g).extract_buffers()
}
