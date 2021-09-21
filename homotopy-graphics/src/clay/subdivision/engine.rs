use std::{
    collections::HashMap,
    mem,
    ops::{Deref, DerefMut},
};

use homotopy_common::idx::IdxVec;
use ultraviolet::Vec4;

use crate::clay::geom::{ElementData, FromMesh, Mesh, MeshData, Vertex};

pub trait Subdivider: Sized {
    type Primitive: MeshData + FromMesh<ElementData>;

    fn new() -> Self;

    fn interpolate(ctx: &mut InterpolationCtx<Self>, primtive: Self::Primitive);

    fn smooth(ctx: &mut SmoothingCtx<Self>, primtive: <Self::Primitive as MeshData>::Idx);
}

pub struct InterpolationCtx<'ctx, 'a, T>
where
    T: Subdivider,
{
    engine: &'ctx mut SubdivisionEngine<'a, T>,
    valence: HashMap<Vertex, u32>,
}

pub struct SmoothingCtx<'ctx, 'a, T>
where
    T: Subdivider,
{
    engine: &'ctx mut SubdivisionEngine<'a, T>,
    smoothed: IdxVec<Vertex, Vec4>,
}

pub struct SubdivisionEngine<'a, T>
where
    T: Subdivider,
{
    pub mesh: &'a mut Mesh<<T as Subdivider>::Primitive>,
    subdivider: T,
}

impl<'a, T> SubdivisionEngine<'a, T>
where
    T: Subdivider,
{
    #[inline]
    pub(super) fn new(mesh: &'a mut Mesh<<T as Subdivider>::Primitive>) -> Self {
        Self {
            mesh,
            subdivider: T::new(),
        }
    }

    pub(super) fn subdivide_once(&mut self) {
        // (0. In debug, clone a copy of the original diagram for sanity checking)
        #[cfg(debug_assertions)]
        let unmodified = self.mesh.clone();

        // 1. Remove all elements from mesh
        let mut elements = IdxVec::new();
        mem::swap(&mut self.mesh.elements, &mut elements);

        // 2. Subdivide and obtain valence
        let valence = {
            let mut interpolation_ctx = InterpolationCtx {
                engine: self,
                valence: HashMap::new(),
            };

            for element in elements.into_values() {
                T::interpolate(&mut interpolation_ctx, element);
            }

            interpolation_ctx.valence
        };

        // 3. Smooth
        let smoothed = {
            let mut smoothing_ctx = SmoothingCtx {
                smoothed: IdxVec::splat(Vec4::zero(), self.mesh.vertices.len()),
                engine: self,
            };

            for element in smoothing_ctx.mesh.elements.keys() {
                T::smooth(&mut smoothing_ctx, element);
            }

            smoothing_ctx.smoothed
        };

        // 4. Update vertex positions and divide by valence
        for (vertex, data) in smoothed {
            let valence = valence[&vertex];
            self.mesh.vertices[vertex].vertex = data / (valence as f32);
        }

        // (5. In debug, sanity check the subdivided mesh)
        #[cfg(debug_assertions)]
        debug_assert!(self.check_bounds_preserved(&unmodified));
    }

    #[cfg(debug_assertions)]
    fn check_bounds_preserved(&self, unmodified: &Mesh<<T as Subdivider>::Primitive>) -> bool {
        let (min, max) = unmodified.vertices.values().fold(
            (
                Vec4::broadcast(f32::INFINITY),
                Vec4::broadcast(f32::NEG_INFINITY),
            ),
            |a, v| (a.0.min_by_component(**v), a.1.max_by_component(**v)),
        );

        self.mesh
            .vertices
            .values()
            .all(|v| v.clamped(min, max) == **v)
    }
}

impl<'ctx, 'a, T> InterpolationCtx<'ctx, 'a, T>
where
    T: Subdivider,
{
    #[inline]
    pub fn update_valence(&mut self, vertex: Vertex) {
        *self.valence.entry(vertex).or_insert(0) += 1;
    }
}

impl<'ctx, 'a, T> SmoothingCtx<'ctx, 'a, T>
where
    T: Subdivider,
{
    #[inline]
    pub fn update_smoothed(&mut self, vertex: Vertex, data: Vec4) {
        self.smoothed[vertex] += data;
    }
}

impl<'ctx, 'a, T> Deref for InterpolationCtx<'ctx, 'a, T>
where
    T: Subdivider,
{
    type Target = SubdivisionEngine<'a, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.engine
    }
}

impl<'ctx, 'a, T> DerefMut for InterpolationCtx<'ctx, 'a, T>
where
    T: Subdivider,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.engine
    }
}

impl<'ctx, 'a, T> Deref for SmoothingCtx<'ctx, 'a, T>
where
    T: Subdivider,
{
    type Target = SubdivisionEngine<'a, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.engine
    }
}

impl<'ctx, 'a, T> DerefMut for SmoothingCtx<'ctx, 'a, T>
where
    T: Subdivider,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.engine
    }
}

impl<'a, T> Deref for SubdivisionEngine<'a, T>
where
    T: Subdivider,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.subdivider
    }
}

impl<'a, T> DerefMut for SubdivisionEngine<'a, T>
where
    T: Subdivider,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.subdivider
    }
}
