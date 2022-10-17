use std::{
    fmt,
    hash::Hash,
    iter::{FromIterator, FusedIterator},
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use petgraph::graph::{EdgeIndex, IndexType, NodeIndex};

pub trait Idx: 'static + Copy + Eq + Hash + fmt::Debug {
    fn index(&self) -> usize;

    fn new(index: usize) -> Self;
}

impl Idx for u16 {
    fn index(&self) -> usize {
        *self as usize
    }

    fn new(index: usize) -> Self {
        index as Self
    }
}

impl Idx for u32 {
    fn index(&self) -> usize {
        *self as usize
    }

    fn new(index: usize) -> Self {
        index as Self
    }
}

impl Idx for u64 {
    fn index(&self) -> usize {
        *self as usize
    }

    fn new(index: usize) -> Self {
        index as Self
    }
}

impl<Ix: IndexType> Idx for NodeIndex<Ix> {
    fn index(&self) -> usize {
        Self::index(*self)
    }

    fn new(index: usize) -> Self {
        Self::new(index)
    }
}

impl<Ix: IndexType> Idx for EdgeIndex<Ix> {
    fn index(&self) -> usize {
        Self::index(*self)
    }

    fn new(index: usize) -> Self {
        Self::new(index)
    }
}

#[macro_export]
macro_rules! declare_idx {
    (
        $(
            $(#[$attrib:meta])*
            $vis:vis struct $name:ident = $ty:ident;
        )*
    ) => {
        $(
            $(#[$attrib])*
            #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
            $vis struct $name($ty);

            impl $crate::idx::Idx for $name {
                #[inline(always)]
                fn index(&self) -> usize {
                    self.0 as usize
                }

                #[inline(always)]
                fn new(index: usize) -> Self {
                    $name(index as $ty)
                }
            }

            #[cfg(feature = "fuzz")]
            impl<'a> arbitrary::Arbitrary<'a> for $name {
                fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
                    Ok($crate::idx::Idx::new(u.int_in_range(0..=4095)?))
                }
            }

            unsafe impl petgraph::graph::IndexType for $name {
                #[inline(always)]
                fn new(x: usize) -> Self {
                    $name(x as $ty)
                }

                #[inline(always)]
                fn index(&self) -> usize {
                    self.0 as usize
                }

                #[inline(always)]
                fn max() -> Self {
                    $name($ty::MAX)
                }
            }
        )*
    }
}

#[derive(Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct IdxVec<I, T> {
    raw: Vec<T>,
    #[serde(skip_serializing, skip_deserializing)]
    _phantom: PhantomData<fn(&I)>,
}

impl<I, T> IdxVec<I, T>
where
    I: Idx,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            raw: vec![],
            _phantom: PhantomData::default(),
        }
    }

    #[inline]
    pub fn splat(t: T, len: usize) -> Self
    where
        T: Clone,
    {
        Self {
            raw: vec![t; len],
            _phantom: PhantomData::default(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            raw: Vec::with_capacity(capacity),
            _phantom: PhantomData::default(),
        }
    }

    #[inline]
    pub fn push(&mut self, elem: T) -> I {
        let index = self.raw.len();
        self.raw.push(elem);
        I::new(index)
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.raw.pop()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    #[inline]
    pub fn map<F, U>(self, mut f: F) -> IdxVec<I, U>
    where
        F: FnMut(T) -> U,
    {
        #[allow(clippy::redundant_closure)]
        IdxVec {
            raw: self.raw.into_iter().map(|x| f(x)).collect(),
            _phantom: PhantomData::default(),
        }
    }

    #[inline]
    pub fn reindex<J>(self) -> IdxVec<J, T>
    where
        J: Idx,
    {
        IdxVec {
            raw: self.raw,
            _phantom: PhantomData::default(),
        }
    }

    #[inline]
    pub fn contains_key(&self, index: I) -> bool {
        self.raw.len() < index.index()
    }

    #[inline]
    pub fn get(&self, index: I) -> Option<&T> {
        self.raw.get(index.index())
    }

    #[inline]
    pub fn get_mut(&mut self, index: I) -> Option<&mut T> {
        self.raw.get_mut(index.index())
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (I, &T)> {
        self.keys().zip(self.values())
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (I, &mut T)> {
        self.keys().zip(self.values_mut())
    }

    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = I> {
        (0..self.raw.len()).map(I::new)
    }

    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.raw.iter()
    }

    #[inline]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.raw.iter_mut()
    }

    #[inline]
    pub fn into_values(self) -> impl Iterator<Item = T> {
        self.raw.into_iter()
    }

    #[inline]
    pub fn into_raw(self) -> Vec<T> {
        self.raw
    }

    #[inline]
    pub fn clear(&mut self) {
        self.raw.clear();
    }
}

impl<I, T> IdxVec<I, T>
where
    T: Eq,
{
    #[inline]
    pub fn contains(&self, t: &T) -> bool {
        self.raw.contains(t)
    }
}

impl<I, T> Default for IdxVec<I, T>
where
    I: Idx,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<I, T> fmt::Debug for IdxVec<I, T>
where
    T: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.raw.fmt(f)
    }
}

impl<I, T> FromIterator<T> for IdxVec<I, T>
where
    I: Idx,
{
    #[inline]
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        let mut idx_vec = Self::new();
        for t in iter {
            idx_vec.push(t);
        }
        idx_vec
    }
}

pub struct IdxVecIterator<I, T> {
    next_idx: usize,
    iter: <Vec<T> as IntoIterator>::IntoIter,
    _phantom: PhantomData<fn(&I)>,
}

impl<I, T> Iterator for IdxVecIterator<I, T>
where
    I: Idx,
{
    type Item = (I, T);

    #[inline]
    fn next(&mut self) -> Option<(I, T)> {
        let next = (I::new(self.next_idx), self.iter.next()?);
        self.next_idx += 1;
        Some(next)
    }
}

impl<I, T> IntoIterator for IdxVec<I, T>
where
    I: Idx,
{
    type IntoIter = IdxVecIterator<I, T>;
    type Item = (I, T);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IdxVecIterator {
            next_idx: 0,
            iter: self.raw.into_iter(),
            _phantom: PhantomData::default(),
        }
    }
}

impl<I, T> ExactSizeIterator for IdxVecIterator<I, T>
where
    I: Idx,
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I, T> FusedIterator for IdxVecIterator<I, T> where I: Idx {}

impl<I, T> Index<I> for IdxVec<I, T>
where
    I: Idx,
{
    type Output = T;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.raw[index.index()]
    }
}

impl<I, T> IndexMut<I> for IdxVec<I, T>
where
    I: Idx,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.raw[index.index()]
    }
}
