use std::ops::{Deref, DerefMut};

use homotopy_common::{
    declare_idx,
    idx::{Idx, IdxVec},
};
use homotopy_core::{Diagram, Generator};
use ultraviolet::{Vec3, Vec4};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Boundary {
    /// Corner - no freedom to move
    Zero = 0,
    /// Edge - free to move along line
    One = 1,
    /// Surface - free to move in space
    Two = 2,
    /// Volume - free to move in time and space
    Three = 3,
}

impl Boundary {
    /// Increase the boundary by 1.
    #[inline]
    pub fn inc(&mut self) {
        *self = match self {
            Self::Zero => Self::One,
            Self::One => Self::Two,
            _ => Self::Three,
        };
    }
}

/// Represents a vertex in a 4-space
#[derive(Debug, Clone, PartialEq)]
pub struct VertDataInner {
    pub vert: Vec4,
    pub boundary: Boundary,
    pub generator: Generator,
}

impl Deref for VertDataInner {
    type Target = Vec4;

    fn deref(&self) -> &Self::Target {
        &self.vert
    }
}

impl DerefMut for VertDataInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vert
    }
}

pub trait MeshData: Clone {
    type Idx: Idx;

    fn get(mesh: &Mesh, idx: Self::Idx) -> &Self;

    fn get_mut(mesh: &mut Mesh, idx: Self::Idx) -> &mut Self;

    fn elements(mesh: &Mesh) -> &IdxVec<Self::Idx, Self>;

    fn elements_mut(mesh: &mut Mesh) -> &mut IdxVec<Self::Idx, Self>;
}

macro_rules! declare_mesh_data {
    ($($vis:vis type $name:ident ($idx:ty) = $ty:ty;)*) => {
        paste::paste! {
            $(
                declare_idx! { $vis struct $name = $idx; }

                $vis type [<$name Data>] = $ty;

                impl MeshData for [<$name Data>] {
                    type Idx = $name;

                    #[inline]
                    fn get(mesh: &Mesh, idx: Self::Idx) -> &Self {
                        &mesh.[<$name:lower s>][idx]
                    }

                    #[inline]
                    fn get_mut(mesh: &mut Mesh, idx: Self::Idx) -> &mut Self {
                        &mut mesh.[<$name:lower s>][idx]
                    }

                    #[inline]
                    fn elements(mesh: &Mesh) -> &IdxVec<Self::Idx, Self> {
                        &mesh.[<$name:lower s>]
                    }

                    #[inline]
                    fn elements_mut(mesh: &mut Mesh) -> &mut IdxVec<Self::Idx, Self> {
                        &mut mesh.[<$name:lower s>]
                    }
                }
            )*

            #[derive(Clone)]
            pub struct Mesh {
                pub diagram: Diagram,
                $(
                    pub [<$name:lower s>]: IdxVec<$name, [<$name Data>]>,
                )*
            }

            impl Mesh {
                pub fn new(diagram: Diagram) -> Self {
                    Self {
                        diagram,
                        $(
                            [<$name:lower s>]: Default::default(),
                        )*
                    }
                }
            }
        }
    }
}

declare_mesh_data! {
    pub type Vert(usize) = VertDataInner;

    pub type Point(usize) = Vert;
    pub type Line(usize) = [Vert; 2];
    pub type Square(usize) = [Vert; 4];
    pub type Cube(usize) = [Vert; 8];

    pub type Curve(usize) = Vec<Vert>;
}

impl Mesh {
    #[allow(unused)]
    #[inline]
    pub fn elements<T>(&self) -> &IdxVec<T::Idx, T>
    where
        T: MeshData,
    {
        T::elements(self)
    }

    #[inline]
    pub fn elements_mut<T>(&mut self) -> &mut IdxVec<T::Idx, T>
    where
        T: MeshData,
    {
        T::elements_mut(self)
    }

    #[inline]
    pub fn mk<T>(&mut self, t: T) -> T::Idx
    where
        T: MeshData,
    {
        self.elements_mut().push(t)
    }

    pub fn bounds(&self) -> (Vec4, Vec4) {
        self.verts.values().fold(
            (
                Vec4::broadcast(f32::INFINITY),
                Vec4::broadcast(f32::NEG_INFINITY),
            ),
            |a, v| (a.0.min_by_component(**v), a.1.max_by_component(**v)),
        )
    }
}

pub trait VertExt {
    fn with_boundary_and_generator(self, boundary: Boundary, generator: Generator) -> VertData;
}

impl VertExt for Vec4 {
    #[inline]
    fn with_boundary_and_generator(self, boundary: Boundary, generator: Generator) -> VertData {
        VertData {
            vert: self,
            boundary,
            generator,
        }
    }
}
