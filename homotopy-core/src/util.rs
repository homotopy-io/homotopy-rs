use std::{
    collections::{HashMap, HashSet},
    hash::BuildHasherDefault,
};

use rustc_hash::FxHasher;

use crate::common::Generator;

pub mod rayon;

pub(crate) fn first_max_generator<I>(iterator: I) -> Option<Generator>
where
    I: IntoIterator<Item = Generator>,
{
    let mut max: Option<Generator> = None;

    for generator in iterator {
        max = match max {
            Some(prev) if prev.dimension >= generator.dimension => Some(prev),
            _ => Some(generator),
        };
    }

    max
}

pub(crate) type Hasher = BuildHasherDefault<FxHasher>;
pub(crate) type FastHashMap<K, V> = HashMap<K, V, Hasher>;
pub(crate) type FastHashSet<K> = HashSet<K, Hasher>;
