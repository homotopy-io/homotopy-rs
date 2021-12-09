use std::{
    collections::{HashMap, HashSet},
    hash::BuildHasherDefault,
};

use rustc_hash::FxHasher;

pub type FastHasher = BuildHasherDefault<FxHasher>;
pub type FastHashMap<K, V> = HashMap<K, V, FastHasher>;
pub type FastHashSet<K> = HashSet<K, FastHasher>;
