use std::{
    collections::BTreeMap, convert::TryInto, hash::Hash, marker::PhantomData, num::NonZeroU32,
};

use bimap::BiHashMap;
use highway::{HighwayHash, HighwayHasher};
use serde::{Deserialize, Serialize};

use crate::{
    rewrite::{Cone, ConeInternal, Label, LabelNode},
    Cospan, Diagram, DiagramN, Generator, Rewrite, Rewrite0, RewriteN,
};

/// Similar to `Hash`, except supposed to be deterministic and shouldn't collide
trait Keyed<K> {
    fn key(&self) -> K;
}

impl<K, H: Hash> Keyed<Key<K>> for H {
    fn key(&self) -> Key<K> {
        let mut h = HighwayHasher::default();
        self.hash(&mut h);
        h.finalize128().into()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Store {
    #[serde(skip_serializing, skip_deserializing)]
    diagram_keys: BiHashMap<Diagram, Key<Diagram>>,
    diagrams: BTreeMap<Key<Diagram>, DiagramSer>,

    #[serde(skip_serializing, skip_deserializing)]
    rewrite_keys: BiHashMap<Rewrite, Key<Rewrite>>,
    rewrites: BTreeMap<Key<Rewrite>, RewriteSer>,

    #[serde(skip_serializing, skip_deserializing)]
    cone_keys: BiHashMap<Cone, Key<Cone>>,
    cones: BTreeMap<Key<Cone>, ConeSer>,

    #[serde(skip_serializing, skip_deserializing)]
    label_keys: BiHashMap<Label, Vec<Key<LabelNode>>>,
    label_nodes: BTreeMap<Key<LabelNode>, LabelNode>,
}

impl Store {
    pub fn pack_diagram(&mut self, diagram: &Diagram) -> Key<Diagram> {
        if let Some(key) = self.diagram_keys.get_by_left(diagram) {
            return *key;
        }

        let serialized = match diagram {
            Diagram::Diagram0(generator) => DiagramSer::D0 {
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
        if let Some(key) = self.rewrite_keys.get_by_left(rewrite) {
            return *key;
        }

        let serialized = match rewrite {
            Rewrite::Rewrite0(r0) => RewriteSer::R0 {
                source: r0.source(),
                target: r0.target(),
                label: r0.label().map(|l| self.pack_label(l)),
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
        if let Some(key) = self.cone_keys.get_by_left(cone) {
            return ConeWithIndexSer {
                index: cone.index as u32,
                cone: *key,
            };
        }

        let serialized = ConeSer {
            source: cone
                .source()
                .iter()
                .map(|cospan| self.pack_cospan(cospan))
                .collect(),
            target: self.pack_cospan(cone.target()),
            singular_slices: {
                cone.singular_slices()
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

    pub fn pack_label(&mut self, label: &Label) -> Vec<Key<LabelNode>> {
        if let Some(key) = self.label_keys.get_by_left(label) {
            return key.clone();
        }

        let keys: Vec<Key<LabelNode>> = label
            .0
            .iter()
            .map(|n| {
                let serialized = n.get().clone();

                let key: Key<LabelNode> = serialized.key();
                self.label_nodes.insert(key, serialized);
                key
            })
            .collect();
        self.label_keys.insert(label.clone(), keys.clone());
        keys
    }

    pub fn unpack_diagram(&mut self, key: Key<Diagram>) -> Option<Diagram> {
        self.diagram_keys.get_by_right(&key).cloned().or_else(|| {
            let diagram = match self.diagrams.get(&key)?.clone() {
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
            };
            diagram
                .as_ref()
                .cloned()
                .map(|r| self.diagram_keys.insert(r, key));
            diagram
        })
    }

    fn unpack_cospan(&mut self, serialized: &CospanSer) -> Option<Cospan> {
        let forward = self.unpack_rewrite(serialized.forward)?;
        let backward = self.unpack_rewrite(serialized.backward)?;
        Some(Cospan { forward, backward })
    }

    pub fn unpack_rewrite(&mut self, key: Key<Rewrite>) -> Option<Rewrite> {
        self.rewrite_keys.get_by_right(&key).cloned().or_else(|| {
            let rewrite = match self.rewrites.get(&key)?.clone() {
                RewriteSer::R0 {
                    source,
                    target,
                    label,
                } => match (source, target, label) {
                    (None, None, None) => Some(Rewrite0(None).into()),
                    (Some(source), Some(target), Some(label)) => {
                        Some(Rewrite0(Some((source, target, self.unpack_label(label)?))).into())
                    }
                    _ => None,
                },
                RewriteSer::Rn { dimension, cones } => {
                    let cones = cones
                        .into_iter()
                        .map(|cone| self.unpack_cone(cone))
                        .collect::<Option<_>>()?;
                    Some(RewriteN::new_unsafe(u32::from(dimension) as usize, cones).into())
                }
            };
            rewrite
                .as_ref()
                .cloned()
                .map(|r| self.rewrite_keys.insert(r, key));
            rewrite
        })
    }

    fn unpack_cone(&mut self, cone: ConeWithIndexSer) -> Option<Cone> {
        let key = cone.cone;
        self.cone_keys
            .get_by_right(&key)
            .cloned()
            .map(|c| {
                Cone::new(
                    cone.index as usize,
                    c.source().to_vec(),
                    c.target().clone(),
                    c.singular_slices().to_vec(),
                )
            })
            .or_else(|| {
                let serialized = self.cones.get(&cone.cone)?.clone();
                let source = serialized
                    .source
                    .into_iter()
                    .map(|cospan| self.unpack_cospan(&cospan))
                    .collect::<Option<_>>()?;
                let target = self.unpack_cospan(&serialized.target)?;
                let singular_slices = serialized
                    .singular_slices
                    .into_iter()
                    .map(|slice| self.unpack_rewrite(slice))
                    .collect::<Option<_>>()?;
                let cone = Some(Cone::new(
                    cone.index as usize,
                    source,
                    target,
                    singular_slices,
                ));
                cone.as_ref()
                    .cloned()
                    .map(|c| self.cone_keys.insert(c.clone(), key));
                cone
            })
    }

    pub fn unpack_label(&mut self, keys: Vec<Key<LabelNode>>) -> Option<Label> {
        self.label_keys.get_by_right(&keys).cloned().or_else(|| {
            let label = Some(Label::new(
                keys.iter()
                    .map(|key| self.label_nodes.get(key).cloned())
                    .collect::<Option<_>>()?,
            ));
            label
                .as_ref()
                .cloned()
                .map(|l| self.label_keys.insert(l, keys));
            label
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum DiagramSer {
    D0 {
        generator: Generator,
    },
    Dn {
        dimension: NonZeroU32,
        source: Key<Diagram>,
        cospans: Vec<CospanSer>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum RewriteSer {
    R0 {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        source: Option<Generator>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        target: Option<Generator>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        label: Option<Vec<Key<LabelNode>>>,
    },
    Rn {
        dimension: NonZeroU32,
        cones: Vec<ConeWithIndexSer>,
    },
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for RewriteSer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            RewriteSer::R0 {
                source,
                target,
                label,
            } => {
                source.hash(state);
                target.hash(state);
                label.hash(state);
            }
            RewriteSer::Rn { dimension, cones } => {
                dimension.hash(state);
                state.write_u32(cones.len() as u32);
                for cone in cones {
                    cone.hash(state);
                }
            }
        }
    }
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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
struct ConeSer {
    source: Vec<CospanSer>,
    target: CospanSer,
    singular_slices: Vec<Key<Rewrite>>,
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for ConeSer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.source.len() as u32);

        for source in &self.source {
            source.hash(state);
        }

        self.target.hash(state);

        for slice in &self.singular_slices {
            slice.hash(state);
        }
    }
}

// Phantom key type
#[derive(Debug)]
pub struct Key<K>([u64; 2], PhantomData<K>);

impl<K> Serialize for Key<K> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        <[u64; 2]>::serialize(&self.0, serializer)
    }
}

impl<'de, K> Deserialize<'de> for Key<K> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <[u64; 2]>::deserialize(deserializer).map(Self::from)
    }
}

impl<K> From<[u64; 2]> for Key<K> {
    fn from(k: [u64; 2]) -> Self {
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
