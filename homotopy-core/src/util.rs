use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::BuildHasherDefault,
};

use rustc_hash::FxHasher;

use crate::common::Generator;

pub fn agreeing<T: Debug + Eq>(options: Vec<Option<T>>) -> Option<T> {
    options
        .into_iter()
        .reduce(|x, y| match (x, y) {
            (None, None) => None,
            (Some(a), Some(b)) => {
                assert_eq!(a, b);
                Some(a)
            }
            (Some(a), None) | (None, Some(a)) => Some(a),
        })
        .flatten()
}

pub fn consistent_assign<T: Eq>(lhs: &mut Option<T>, rhs: T) -> Option<()> {
    match lhs {
        Some(l) if l == &rhs => {}
        None => *lhs = Some(rhs),
        _ => return None,
    }
    Some(())
}

pub fn first_max_generator<I>(iterator: I, dimension_cutoff: Option<usize>) -> Option<Generator>
where
    I: IntoIterator<Item = Generator>,
{
    let mut max: Option<Generator> = None;

    for generator in iterator {
        if Some(generator.dimension) == dimension_cutoff {
            return Some(generator);
        }

        max = match max {
            Some(prev) if prev.dimension >= generator.dimension => Some(prev),
            _ => Some(generator),
        };
    }

    max
}

#[derive(Debug)]
pub(crate) struct CachedCell<T>(Cell<Option<T>>)
where
    T: Copy;

impl<T> CachedCell<T>
where
    T: Copy,
{
    pub fn new() -> Self {
        Self(Cell::new(None))
    }

    pub fn compute<F>(&self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.0.get().map_or_else(
            || {
                let value = f();
                self.0.set(Some(value));
                value
            },
            |cached| cached,
        )
    }
}

impl<T> Clone for CachedCell<T>
where
    T: Copy,
{
    fn clone(&self) -> Self {
        Self(Cell::new(self.0.get()))
    }
}

impl<T> Default for CachedCell<T>
where
    T: Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

pub type Hasher = BuildHasherDefault<FxHasher>;
pub type FastHashMap<K, V> = HashMap<K, V, Hasher>;
pub type FastHashSet<K> = HashSet<K, Hasher>;
