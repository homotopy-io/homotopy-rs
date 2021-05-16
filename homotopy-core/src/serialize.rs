use crate::util::FastHashMap;
use crate::{rewrite::Cone, Cospan, Diagram, DiagramN, Generator, Rewrite, Rewrite0, RewriteN};
use highway::{HighwayHash, HighwayHasher};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap, convert::TryInto, hash::Hash, marker::PhantomData, num::NonZeroU32,
};

/// Similar to `Hash`, except supposed to be deterministic and shouldn't collide
trait Keyed<K> {
    fn key(&self) -> K;
}

impl<K, H: Hash> Keyed<Key<K>> for H {
    fn key(&self) -> Key<K> {
        let mut h = HighwayHasher::default();
        self.hash(&mut h);
        let hash = h.finalize128();
        (u128::from(hash[1]) + (u128::from(hash[0]) << 64)).into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    #[serde(skip_serializing, skip_deserializing)]
    diagram_keys: FastHashMap<Diagram, Key<Diagram>>,
    diagrams: BTreeMap<Key<Diagram>, DiagramSer>,

    #[serde(skip_serializing, skip_deserializing)]
    rewrite_keys: FastHashMap<Rewrite, Key<Rewrite>>,
    rewrites: BTreeMap<Key<Rewrite>, RewriteSer>,

    #[serde(skip_serializing, skip_deserializing)]
    cone_keys: FastHashMap<Cone, Key<Cone>>,
    cones: BTreeMap<Key<Cone>, ConeSer>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            diagram_keys: Default::default(),
            diagrams: Default::default(),
            rewrite_keys: Default::default(),
            rewrites: Default::default(),
            cone_keys: Default::default(),
            cones: Default::default(),
        }
    }

    pub fn pack_diagram(&mut self, diagram: &Diagram) -> Key<Diagram> {
        if let Some(key) = self.diagram_keys.get(diagram) {
            return *key;
        }

        let serialized = match diagram {
            Diagram::Diagram0(generator) => DiagramSer::D0 {
                dimension: ZeroU32::default(),
                generator: *generator,
            },
            Diagram::DiagramN(diagram) => {
                let source = self.pack_diagram(&diagram.source());
                let cospans = diagram
                    .cospans()
                    .iter()
                    .map(|cospan| self.pack_cospan(cospan))
                    .collect();
                DiagramSer::Dn {
                    dimension: (diagram.dimension() as u32).try_into().unwrap(),
                    source,
                    cospans,
                }
            }
        };

        let key: Key<Diagram> = serialized.key();
        self.diagram_keys.insert(diagram.clone(), key);
        self.diagrams.insert(key, serialized);
        key
    }

    fn pack_cospan(&mut self, cospan: &Cospan) -> CospanSer {
        let forward = self.pack_rewrite(&cospan.forward);
        let backward = self.pack_rewrite(&cospan.backward);
        CospanSer { forward, backward }
    }

    pub fn pack_rewrite(&mut self, rewrite: &Rewrite) -> Key<Rewrite> {
        if let Some(key) = self.rewrite_keys.get(&rewrite) {
            return *key;
        }

        let serialized = match rewrite {
            Rewrite::Rewrite0(Rewrite0(None)) => RewriteSer::R0 {
                dimension: ZeroU32::default(),
                source: None,
                target: None,
            },
            Rewrite::Rewrite0(Rewrite0(Some((x, y)))) => RewriteSer::R0 {
                dimension: ZeroU32::default(),
                source: Some(*x),
                target: Some(*y),
            },
            Rewrite::RewriteN(rewrite) => {
                let cones = rewrite
                    .cones()
                    .iter()
                    .map(|cone| self.pack_cone(cone))
                    .collect();
                RewriteSer::Rn {
                    dimension: (rewrite.dimension() as u32).try_into().unwrap(),
                    cones,
                }
            }
        };

        let key: Key<Rewrite> = serialized.key();
        self.rewrite_keys.insert(rewrite.clone(), key);
        self.rewrites.insert(key, serialized);
        key
    }

    fn pack_cone(&mut self, cone: &Cone) -> ConeWithIndexSer {
        if let Some(key) = self.cone_keys.get(cone) {
            return ConeWithIndexSer {
                index: cone.index as u32,
                cone: *key,
            };
        }

        let serialized = ConeSer {
            source: cone
                .internal
                .source
                .iter()
                .map(|cospan| self.pack_cospan(cospan))
                .collect(),
            target: self.pack_cospan(&cone.internal.target),
            slices: {
                cone.internal
                    .slices
                    .iter()
                    .map(|slice| self.pack_rewrite(slice))
                    .collect()
            },
        };

        let key: Key<Cone> = serialized.key();
        self.cone_keys.insert(cone.clone(), key);
        self.cones.insert(key, serialized);
        ConeWithIndexSer {
            index: cone.index as u32,
            cone: key,
        }
    }

    pub fn unpack_diagram(&self, key: Key<Diagram>) -> Option<Diagram> {
        match self.diagrams.get(&key)?.clone() {
            DiagramSer::D0 { generator, .. } => Some(Diagram::from(generator)),
            DiagramSer::Dn {
                source, cospans, ..
            } => {
                let source = self.unpack_diagram(source)?;
                let cospans = cospans
                    .into_iter()
                    .map(|cospan| self.unpack_cospan(&cospan))
                    .collect::<Option<_>>()?;
                Some(DiagramN::new_unsafe(source, cospans).into())
            }
        }
    }

    fn unpack_cospan(&self, serialized: &CospanSer) -> Option<Cospan> {
        let forward = self.unpack_rewrite(serialized.forward)?;
        let backward = self.unpack_rewrite(serialized.backward)?;
        Some(Cospan { forward, backward })
    }

    pub fn unpack_rewrite(&self, key: Key<Rewrite>) -> Option<Rewrite> {
        match self.rewrites.get(&key)?.clone() {
            RewriteSer::R0 { source, target, .. } => match (source, target) {
                (None, None) => Some(Rewrite0(None).into()),
                (Some(source), Some(target)) => Some(Rewrite0(Some((source, target))).into()),
                (None, Some(_)) | (Some(_), None) => None,
            },
            RewriteSer::Rn { dimension, cones } => {
                let cones = cones
                    .into_iter()
                    .map(|cone| self.unpack_cone(cone))
                    .collect::<Option<_>>()?;
                Some(RewriteN::new(u32::from(dimension) as usize, cones).into())
            }
        }
    }

    fn unpack_cone(&self, cone: ConeWithIndexSer) -> Option<Cone> {
        let serialized = self.cones.get(&cone.cone)?.clone();
        let source = serialized
            .source
            .into_iter()
            .map(|cospan| self.unpack_cospan(&cospan))
            .collect::<Option<_>>()?;
        let target = self.unpack_cospan(&serialized.target)?;
        let slices = serialized
            .slices
            .into_iter()
            .map(|slice| self.unpack_rewrite(slice))
            .collect::<Option<_>>()?;
        Some(Cone::new(cone.index as usize, source, target, slices))
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum DiagramSer {
    D0 {
        dimension: ZeroU32,
        generator: Generator,
    },
    Dn {
        dimension: NonZeroU32,
        source: Key<Diagram>,
        cospans: Vec<CospanSer>,
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum RewriteSer {
    R0 {
        dimension: ZeroU32,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        source: Option<Generator>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        target: Option<Generator>,
    },
    Rn {
        dimension: NonZeroU32,
        cones: Vec<ConeWithIndexSer>,
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
struct CospanSer {
    forward: Key<Rewrite>,
    backward: Key<Rewrite>,
}

#[derive(Debug, PartialEq, Copy, Eq, Hash, Clone, Serialize, Deserialize)]
struct ConeWithIndexSer {
    index: u32,
    cone: Key<Cone>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
struct ConeSer {
    source: Vec<CospanSer>,
    target: CospanSer,
    slices: Vec<Key<Rewrite>>,
}

// Phantom key type
#[derive(Debug)]
pub struct Key<K>(u128, PhantomData<K>);

impl<K> Serialize for Key<K> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:032x}", self.0))
    }
}

impl<'de, K> Deserialize<'de> for Key<K> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct HexVisitor;

        impl<'de> serde::de::Visitor<'de> for HexVisitor {
            type Value = u128;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a hex encoded key")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                u128::from_str_radix(v, 16).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_string(HexVisitor).map(Self::from)
    }
}

impl<K> From<u128> for Key<K> {
    fn from(k: u128) -> Self {
        Self(k, PhantomData)
    }
}

impl<K> Clone for Key<K> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<K> Copy for Key<K> {}

impl<K> PartialEq for Key<K> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<K> Eq for Key<K> {}

impl<K> PartialOrd for Key<K> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<K> Ord for Key<K> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<K> Hash for Key<K> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
struct ZeroU32;

impl Serialize for ZeroU32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(0)
    }
}

impl<'de> Deserialize<'de> for ZeroU32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let number = u32::deserialize(deserializer)?;

        if 0 == number {
            Ok(ZeroU32)
        } else {
            Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Unsigned(u64::from(number)),
                &"zero",
            ))
        }
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for ZeroU32 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(0);
    }
}
