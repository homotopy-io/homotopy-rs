use std::collections::HashMap;
use std::default::Default;
use std::fmt;
use std::hash::{BuildHasher, Hash, Hasher};
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::mem;

/// Represents a cubical surface mesh using an indexed list of vertices and an
/// indexed list of cubical surface elements that organise the vertices.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Mesh {
    vertices : IdxVec<SimpleIdx, Vertex>,
    elements : IdxVec<SimpleIdx, Element>, 
}

impl Mesh
{
    pub fn new() -> Mesh {
        Mesh {
            vertices: IdxVec::new(),
            elements: IdxVec::new(), 
        }
    }

    pub fn push_vertex(&mut self, v: Vertex) -> SimpleIdx {
        return self.vertices.push(v);
    }

    pub fn push_element(&mut self, e: Element) -> SimpleIdx {
        return self.elements.push(e);
    }
}

/// Represents cubical surface elements (points, lines, squares, cubes, ...) 
/// that have the cubical property (composed of exactly opposite subfaces).
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Element {
    Cube0(SimpleIdx),
    CubeN(CubeN),
}

impl Element {
    pub fn order(&self) -> u8 {
        use Element::{Cube0, CubeN};
        match self {
            Cube0(_) => 0,
            CubeN(c) => c.n,
        }
    }

    pub fn from_list(n: u8, list : Vec<SimpleIdx>) -> Element {
        if n == 0 {
            return Element::Cube0(list[0]);
        }
        let subcube0 = Element::from_list(n-1, list[..list.len()/2].to_vec());
        let subcube1 = Element::from_list(n-1, list[list.len()/2..].to_vec());
        return Element::CubeN(
            CubeN::new(n, subcube0, subcube1)
        );
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CubeN {
    n: u8,
    subcubes: Vec<Element>, //How to represent it correctly? there should be exactly two subcubes of n-1
}

impl CubeN {
    pub fn new(n : u8, subcube0 : Element, subcube1 : Element) -> CubeN {
        assert!(n == subcube0.order() + 1);
        assert!(n == subcube1.order() + 1);
        let mut subcubes = Vec::new();
        subcubes.push(subcube0);
        subcubes.push(subcube1);
        CubeN {
            n: n,
            subcubes: subcubes, 
        }
    }
}

/// Represents a Vertex in a 4-space
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Vertex {
    x: Coordinate,
    y: Coordinate,
    z: Coordinate,
    t: Coordinate,
    boundary: u8,
    //generator: Generator, should this be represented as an id? 
}

impl Vertex
{
    pub fn new(x : f64, y : f64, z : f64, t : f64, boundary: u8) -> Vertex {
        Vertex {
            x: Coordinate::new(x),
            y: Coordinate::new(y),
            z: Coordinate::new(z),
            t: Coordinate::new(t),
            boundary
        }
    }
}

#[derive(Clone)]
struct Coordinate {
    val : f64,
}

impl Coordinate
{
    pub fn new(val: f64) -> Coordinate {
        Coordinate {
            val,
        }
    }
}

fn integer_decode(val: f64) -> (u64, i16, i8) {
    let bits: u64 = unsafe { mem::transmute(val) };
    let sign: i8 = if bits >> 63 == 0 { 1 } else { -1 };
    let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0xfffffffffffff) << 1
    } else {
        (bits & 0xfffffffffffff) | 0x10000000000000
    };

    exponent -= 1023 + 52;
    (mantissa, exponent, sign)
}

impl PartialEq for Coordinate {
    fn eq(&self, other: &Self) -> bool {
        integer_decode(self.val) == integer_decode(other.val) 
    }
}

impl Eq for Coordinate {}

impl Hash for Coordinate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        integer_decode(self.val).hash(state);
    }
}

pub trait Idx: 'static + Copy + Eq + Hash + fmt::Debug {
    fn index(&self) -> usize;

    fn new(index: usize) -> Self;
}

#[derive(Clone, PartialEq, Eq, Hash, Copy, fmt::Debug)]
pub struct SimpleIdx {
    index: usize,
}

impl Idx for SimpleIdx {
    fn index(&self) -> usize {
        self.index
    }

    fn new(index: usize) -> Self {
        SimpleIdx {
            index: index,
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
    pub fn new() -> IdxVec<I, T> {
        IdxVec {
            raw: vec![],
            _phantom: PhantomData::default(),
        }
    }

    pub fn with_capacity(capacity: usize) -> IdxVec<I, T> {
        IdxVec {
            raw: Vec::with_capacity(capacity),
            _phantom: PhantomData::default(),
        }
    }

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

pub trait Indexable<K, T> {
    fn reindex<I: Idx>(self) -> (HashMap<K, I>, IdxVec<I, T>);
}

impl<K, T, S> Indexable<K, T> for HashMap<K, T, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    fn reindex<I: Idx>(self) -> (HashMap<K, I>, IdxVec<I, T>) {
        let mut idx_vec = IdxVec::with_capacity(self.len());
        let mut mapping = HashMap::new();
        for (k, v) in self.into_iter() {
            let i = idx_vec.push(v);
            mapping.insert(k, i);
        }
        (mapping, idx_vec)
    }
}

pub trait IntoIdxVec<I, T> {
    fn into_idx_vec(self) -> Option<IdxVec<I, T>>;
}

impl<I, T, S> IntoIdxVec<I, T> for HashMap<I, T, S>
where
    I: Idx,
    S: BuildHasher,
{
    fn into_idx_vec(mut self) -> Option<IdxVec<I, T>> {
        let mut idx_vec = IdxVec::with_capacity(self.len());
        for i in 0..self.len() {
            let idx = I::new(i);
            match self.remove(&idx) {
                Some(v) => {
                    idx_vec.push(v);
                }
                _ => return None,
            }
        }
        Some(idx_vec)
    }
}

impl<I, T> fmt::Debug for IdxVec<I, T>
where
    I: Idx,
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
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