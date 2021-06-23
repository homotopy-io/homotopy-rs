use std::fmt;
use std::iter::{FromIterator, FusedIterator};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

use std::hash::Hash;

pub trait Idx: 'static + Copy + Eq + Hash + fmt::Debug {
    fn index(&self) -> usize;

    fn new(index: usize) -> Self;
}

#[macro_export]
macro_rules! declare_idx {
    ($vis:vis struct $name:ident = $ty:ident;) => {
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        $vis struct $name($ty);

        impl $crate::idx::Idx for $name {
            #[inline(always)]
            fn index(&self) -> usize {
                self.0.into()
            }

            #[inline(always)]
            fn new(index: usize) -> Self {
                $name($ty::from(index))
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct IdxVec<I, T> {
    raw: Vec<T>,
    _phantom: PhantomData<fn(&I)>,
}

impl<I, T> IdxVec<I, T>
where
    I: Idx,
{
    #[inline]
    pub fn new() -> IdxVec<I, T> {
        IdxVec {
            raw: vec![],
            _phantom: PhantomData::default(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> IdxVec<I, T> {
        IdxVec {
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
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.raw.len()
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
    pub fn clear(&mut self) {
        self.raw.clear()
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
    fn default() -> IdxVec<I, T> {
        IdxVec::new()
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
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        let mut idx_vec = IdxVec::new();
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
    type Item = (I, T);
    type IntoIter = IdxVecIterator<I, T>;

    fn into_iter(self) -> Self::IntoIter {
        IdxVecIterator {
            next_idx: 0,
            iter: self.raw.into_iter(),
            _phantom: PhantomData::default(),
        }
    }
}

impl<I, T> FusedIterator for IdxVecIterator<I, T> where I: Idx {}

impl<I, T> Index<I> for IdxVec<I, T>
where
    I: Idx,
{
    type Output = T;

    fn index(&self, index: I) -> &Self::Output {
        &self.raw[index.index()]
    }
}

impl<I, T> IndexMut<I> for IdxVec<I, T>
where
    I: Idx,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.raw[index.index()]
    }
}
