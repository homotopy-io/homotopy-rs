use std::hash::{BuildHasherDefault, Hash, Hasher};

use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

pub type FastHasher = BuildHasherDefault<FxHasher>;
pub type FastHashMap<K, V> = FxHashMap<K, V>;
pub type FastHashSet<K> = FxHashSet<K>;

// backport unstable feature "build_hasher_simple_hash_one"
// https://github.com/rust-lang/rust/issues/86161
// !! use for debugging only !!
pub fn hash_one<T: Hash>(x: T) -> u64 {
    let mut hasher: FxHasher = Default::default();
    x.hash(&mut hasher);
    hasher.finish()
}
