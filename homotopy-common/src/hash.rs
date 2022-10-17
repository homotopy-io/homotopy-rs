use std::hash::BuildHasherDefault;

use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

pub type FastHasher = BuildHasherDefault<FxHasher>;
pub type FastHashMap<K, V> = FxHashMap<K, V>;
pub type FastHashSet<K> = FxHashSet<K>;
