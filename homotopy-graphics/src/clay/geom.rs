use std::ops::{Deref, DerefMut};

use homotopy_common::{
    declare_idx,
    idx::{Idx, IdxVec},
};
use homotopy_core::Generator;
use ultraviolet::Vec4;

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
pub struct VertData {
    pub vert: Vec4,
    pub stratum: usize,
    pub boundary: Boundary,
    pub generator: Generator,
}

impl Deref for VertData {
    type Target = Vec4;

    fn deref(&self) -> &Self::Target {
        &self.vert
    }
}

impl DerefMut for VertData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vert
    }
}

/// Represents a piece-wise linear curve in a 4-space
#[derive(Debug, Clone, PartialEq)]
pub struct CurveDataInner {
    pub verts: Vec<Vert>,
    pub generator: Generator,
}

impl Deref for CurveDataInner {
    type Target = Vec<Vert>;

    fn deref(&self) -> &Self::Target {
        &self.verts
    }
}

impl DerefMut for CurveDataInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.verts
    }
}

pub trait WithBoundaryAndGenerator<T> {
    fn with_boundary_and_generator(
        self,
        stratum: usize,
        boundary: Boundary,
        generator: Generator,
    ) -> T;
}

impl WithBoundaryAndGenerator<VertData> for Vec4 {
    #[inline]
    fn with_boundary_and_generator(
        self,
        stratum: usize,
        boundary: Boundary,
        generator: Generator,
    ) -> VertData {
        VertData {
            vert: self,
            stratum,
            boundary,
            generator,
        }
    }
}

pub trait WithGenerator<T> {
    fn with_generator(self, generator: Generator) -> T;
}

impl WithGenerator<CurveDataInner> for Vec<Vert> {
    #[inline]
    fn with_generator(self, generator: Generator) -> CurveDataInner {
        CurveDataInner {
            verts: self,
            generator,
        }
    }
}

declare_idx! {
    pub struct Vert = usize;
}

pub trait Mesh: Sized {
    fn verts(&self) -> &IdxVec<Vert, VertData>;

    fn verts_mut(&mut self) -> &mut IdxVec<Vert, VertData>;

    fn bounds(&self) -> (Vec4, Vec4) {
        self.verts().values().fold(
            (
                Vec4::broadcast(f32::INFINITY),
                Vec4::broadcast(f32::NEG_INFINITY),
            ),
            |a, v| (a.0.min_by_component(**v), a.1.max_by_component(**v)),
        )
    }
}

pub trait MeshData<T: Mesh>: Clone {
    type Idx: Idx;

    fn elements(mesh: &T) -> &IdxVec<Self::Idx, Self>;

    fn elements_mut(mesh: &mut T) -> &mut IdxVec<Self::Idx, Self>;

    #[inline]
    fn get(mesh: &T, idx: <Self as MeshData<T>>::Idx) -> &Self {
        &Self::elements(mesh)[idx]
    }

    #[inline]
    fn get_mut(mesh: &mut T, idx: <Self as MeshData<T>>::Idx) -> &mut Self {
        &mut Self::elements_mut(mesh)[idx]
    }
}

pub trait Carries<T>: Mesh
where
    T: MeshData<Self>,
{
    fn mk(&mut self, t: T) -> <T as MeshData<Self>>::Idx;
}

impl<T> MeshData<T> for VertData
where
    T: Mesh,
{
    type Idx = Vert;

    fn elements(mesh: &T) -> &IdxVec<Self::Idx, Self> {
        mesh.verts()
    }

    fn elements_mut(mesh: &mut T) -> &mut IdxVec<Self::Idx, Self> {
        mesh.verts_mut()
    }
}

impl<M, T> Carries<T> for M
where
    M: Mesh,
    T: MeshData<M>,
{
    fn mk(&mut self, t: T) -> <T as MeshData<Self>>::Idx {
        T::elements_mut(self).push(t)
    }
}

macro_rules! declare_mesh {
    (
        $mesh_vis:vis struct $mesh:ident {
            $($vis:vis type $name:ident ($idx:ty) = $ty:ty;)*
        }
    ) => {
        paste::paste! {
            $(
                homotopy_common::declare_idx! { $vis struct $name = $idx; }

                $vis type [<$name Data>] = $ty;
            )*

            #[derive(Clone)]
            pub struct $mesh {
                pub diagram: homotopy_core::Diagram,
                pub verts: homotopy_common::idx::IdxVec<
                    $crate::clay::geom::Vert,
                    $crate::clay::geom::VertData,
                >,
                $(
                    pub [<$name:lower s>]: homotopy_common::idx::IdxVec<
                        $name,
                        [<$name Data>],
                    >,
                )*
            }

            const _: () = {
                use homotopy_common::idx::IdxVec;
                use homotopy_core::Diagram;

                use $crate::clay::geom::{Mesh, MeshData, Vert, VertData};

                impl $mesh {
                    pub fn new(diagram: Diagram) -> Self {
                        Self {
                            diagram,
                            verts: Default::default(),
                            $(
                                [<$name:lower s>]: Default::default(),
                            )*
                        }
                    }
                }

                impl Mesh for $mesh {
                    #[inline]
                    fn verts(&self) -> &IdxVec<Vert, VertData> {
                        &self.verts
                    }

                    #[inline]
                    fn verts_mut(&mut self) -> &mut IdxVec<Vert, VertData> {
                        &mut self.verts
                    }
                }

                $(
                    impl MeshData<$mesh> for [<$name Data>] {
                        type Idx = $name;

                        #[inline]
                        fn elements(mesh: &$mesh) -> &IdxVec<Self::Idx, Self> {
                            &mesh.[<$name:lower s>]
                        }

                        #[inline]
                        fn elements_mut(mesh: &mut $mesh) -> &mut IdxVec<Self::Idx, Self> {
                            &mut mesh.[<$name:lower s>]
                        }
                    }
                )*
            };
        }
    }
}

pub mod cubical {
    use super::{CurveDataInner, Vert};

    declare_mesh! {
        pub struct CubicalMesh {
            pub type Point(usize) = Vert;
            pub type Line(usize) = [Vert; 2];
            pub type Square(usize) = [Vert; 4];
            pub type Cube(usize) = [Vert; 8];
            pub type Curve(usize) = CurveDataInner;
        }
    }
}

pub mod simplicial {
    use std::cmp::Ordering;

    use homotopy_common::parity;

    use super::{cubical, Carries, CurveDataInner, Deref, DerefMut, IdxVec, Mesh, Vert};

    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum Orientation {
        Anticlockwise,
        Clockwise,
    }

    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub struct Oriented<T> {
        inner: T,
        pub orientation: Orientation,
    }

    impl<T> Oriented<T> {
        #[inline]
        pub fn anticlockwise(t: T) -> Self {
            Self {
                inner: t,
                orientation: Orientation::Anticlockwise,
            }
        }

        #[inline]
        pub fn clockwise(t: T) -> Self {
            Self {
                inner: t,
                orientation: Orientation::Clockwise,
            }
        }

        #[inline]
        pub fn from_parity(t: T, parity: bool) -> Self {
            if parity {
                Self::anticlockwise(t)
            } else {
                Self::clockwise(t)
            }
        }
    }

    impl<T> Deref for Oriented<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl<T> DerefMut for Oriented<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }

    declare_mesh! {
        pub struct SimplicialMesh {
            pub type Point(usize) = Vert;
            pub type Line(usize) = Oriented<[Vert; 2]>;
            pub type Tri(usize) = Oriented<[Vert; 3]>;
            pub type Tetra(usize) = Oriented<[Vert; 4]>;
            pub type Curve(usize) = CurveDataInner;
        }
    }

    impl From<cubical::CubicalMesh> for SimplicialMesh {
        fn from(cubical: cubical::CubicalMesh) -> Self {
            #[inline]
            fn time_order<M: Mesh>(mesh: &M, i: Vert, j: Vert) -> Ordering {
                let verts = mesh.verts();
                verts[i]
                    .w
                    .partial_cmp(&verts[j].w)
                    .unwrap_or(Ordering::Equal)
            }

            let mut simplicial = Self {
                diagram: cubical.diagram,
                verts: cubical.verts,
                points: cubical.points.reindex(),
                lines: IdxVec::with_capacity(cubical.lines.len()),
                tris: IdxVec::with_capacity(cubical.squares.len() * 2),
                tetras: IdxVec::with_capacity(cubical.cubes.len() * 5),
                curves: cubical.curves.reindex(),
            };

            for line in cubical.lines.into_values() {
                let mut verts = [line[0], line[1]];
                let parity = parity::sort_2(&mut verts, |i, j| time_order(&simplicial, i, j));
                simplicial.mk(Oriented::from_parity(verts, parity));
            }

            for mut square in cubical.squares.into_values() {
                const TRI_ASSEMBLY_ORDER: [[usize; 3]; 2] = [[0, 1, 2], [1, 3, 2]];

                // Simplices need to have unique strata at each vertex in order to be coloured properly.
                // If we see identical at the diagonal of the square, rotate counter-clockwise so we factor
                // along the correct diagonal
                if simplicial.verts[square[1]].stratum == simplicial.verts[square[2]].stratum {
                    square = [square[1], square[3], square[0], square[2]];
                }

                for [i, j, k] in TRI_ASSEMBLY_ORDER {
                    let mut verts @ [i, j, k] = [square[i], square[j], square[k]];

                    if i != j && j != k && k != i {
                        let parity =
                            parity::sort_3(&mut verts, |i, j| time_order(&simplicial, i, j));
                        simplicial.mk(Oriented::from_parity(verts, parity));
                    }
                }
            }

            for mut cube in cubical.cubes.into_values() {
                const TETRA_ASSEMBLY_ORDER: [[usize; 4]; 5] = [
                    [1, 4, 5, 7],
                    [0, 4, 1, 2],
                    [1, 7, 3, 2],
                    [4, 6, 7, 2],
                    [1, 7, 2, 4],
                ];

                // See above (but this could be wrong)
                if simplicial.verts[cube[0]].stratum == simplicial.verts[cube[7]].stratum {
                    cube = if simplicial.verts[cube[1]].stratum == simplicial.verts[cube[6]].stratum
                    {
                        [
                            cube[2], cube[0], cube[3], cube[1], cube[6], cube[4], cube[7], cube[5],
                        ]
                    } else {
                        [
                            cube[1], cube[5], cube[3], cube[7], cube[0], cube[4], cube[2], cube[6],
                        ]
                    };
                }

                for [i, j, k, l] in TETRA_ASSEMBLY_ORDER {
                    let mut verts @ [i, j, k, l] = [cube[i], cube[j], cube[k], cube[l]];

                    if i != j && j != k && k != l && l != i {
                        let parity =
                            parity::sort_4(&mut verts, |i, j| time_order(&simplicial, i, j));
                        simplicial.mk(Oriented::from_parity(verts, parity));
                    }
                }
            }

            simplicial
        }
    }
}
