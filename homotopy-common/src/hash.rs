use std::hash::BuildHasherDefault;
use std::collections::{HashMap, HashSet};

use rustc_hash::FxHasher;

pub type FastHasher = BuildHasherDefault<FxHasher>;
pub type FastHashMap<K, V> = HashMap<K, V, FastHasher>;
pub type FastHashSet<K> = HashSet<K, FastHasher>;
