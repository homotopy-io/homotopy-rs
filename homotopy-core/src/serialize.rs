use highway::{HighwayHash, HighwayHasher};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash, marker::PhantomData};

use crate::{
    rewrite::Cone, typecheck::Signature, Cospan, Diagram, DiagramN, Generator, Rewrite, Rewrite0,
    RewriteN,
};

// Phantom key type
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct Key<K>(u128, PhantomData<K>);

impl<K> From<u128> for Key<K> {
    fn from(k: u128) -> Self {
        Key(k, PhantomData)
    }
}

/// Similar to `Hash`, except supposed to be deterministic and shouldn't collide
pub trait Keyed<K> {
    fn key(&self) -> K;
}

impl<K, H: Hash> Keyed<Key<K>> for H {
    fn key(&self) -> Key<K> {
        let mut h = HighwayHasher::default();
        self.hash(&mut h);
        let hash = h.finalize128();
        ((hash[1] as u128 + (hash[0] as u128)) << 64).into()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Serialization {
    pub version: usize,
    diagrams: HashMap<Key<Diagram>, DiagramSer>,
    rewrites: HashMap<Key<Rewrite>, RewriteSer>,
    cones: HashMap<Key<Cone>, ConeSer>,
    signature: HashMap<Generator, Key<Diagram>>,
}

impl Serialization {
    pub fn diagram(&self, key: Key<Diagram>) -> Diagram {
        self.diagrams[&key].rehydrate(self)
    }
}

impl Default for Serialization {
    fn default() -> Self {
        Serialization {
            version: 0,
            diagrams: Default::default(),
            rewrites: Default::default(),
            cones: Default::default(),
            signature: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
enum DiagramSer {
    D0(Generator),
    DN {
        source: Key<Diagram>,
        cospans: Vec<CospanSer>,
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
enum RewriteSer {
    R0(Option<(Generator, Generator)>),
    RN {
        dimension: usize,
        cones: Vec<Key<Cone>>,
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
struct CospanSer {
    forward: Key<Rewrite>,
    backward: Key<Rewrite>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
struct ConeSer {
    index: usize,
    source: Vec<CospanSer>,
    target: CospanSer,
    slices: Vec<Key<Rewrite>>,
}

trait Dehydrate {
    type Dehydrated;
    fn dehydrate(&self, serialization: &mut Serialization) -> Self::Dehydrated;
}

impl Dehydrate for Diagram {
    type Dehydrated = DiagramSer;

    fn dehydrate(&self, serialization: &mut Serialization) -> Self::Dehydrated {
        let k = self.key();
        if let Some(s) = serialization.diagrams.get(&k) {
            return s.clone();
        }
        match self {
            Diagram::Diagram0(g) => {
                if let Some(old) = serialization.diagrams.insert(k, DiagramSer::D0(*g)) {
                    assert_eq!(old, DiagramSer::D0(*g))
                }
                DiagramSer::D0(*g)
            }
            Diagram::DiagramN(n) => {
                n.source().dehydrate(serialization);
                let cospans = n
                    .cospans()
                    .iter()
                    .map(|c| c.dehydrate(serialization))
                    .collect();
                let ser = DiagramSer::DN {
                    source: n.source().key(),
                    cospans,
                };
                serialization.diagrams.insert(k, ser.clone());
                ser
            }
        }
    }
}

impl Dehydrate for Cospan {
    type Dehydrated = CospanSer;

    fn dehydrate(&self, serialization: &mut Serialization) -> Self::Dehydrated {
        self.forward.dehydrate(serialization);
        self.backward.dehydrate(serialization);
        CospanSer {
            forward: self.forward.key(),
            backward: self.backward.key(),
        }
    }
}

impl Dehydrate for Cone {
    type Dehydrated = ConeSer;

    fn dehydrate(&self, serialization: &mut Serialization) -> Self::Dehydrated {
        let k = self.key();
        let ser = ConeSer {
            index: self.index,
            source: self
                .source
                .iter()
                .map(|s| s.dehydrate(serialization))
                .collect(),
            target: self.target.dehydrate(serialization),
            slices: {
                self.slices
                    .iter()
                    .map(|s| {
                        s.dehydrate(serialization);
                        s.key()
                    })
                    .collect()
            },
        };
        serialization.cones.insert(k, ser.clone());
        ser
    }
}

impl Dehydrate for Rewrite {
    type Dehydrated = RewriteSer;

    fn dehydrate(&self, serialization: &mut Serialization) -> Self::Dehydrated {
        let k = self.key();
        if let Some(s) = serialization.rewrites.get(&k) {
            return s.clone();
        }
        match self {
            Rewrite::Rewrite0(Rewrite0(None)) => {
                if let Some(old) = serialization.rewrites.insert(k, RewriteSer::R0(None)) {
                    assert_eq!(old, RewriteSer::R0(None))
                }
                RewriteSer::R0(None)
            }
            Rewrite::Rewrite0(Rewrite0(Some((x, y)))) => {
                if let Some(old) = serialization
                    .rewrites
                    .insert(k, RewriteSer::R0(Some((*x, *y))))
                {
                    assert_eq!(old, RewriteSer::R0(Some((*x, *y))))
                }
                RewriteSer::R0(Some((*x, *y)))
            }
            Rewrite::RewriteN(n) => {
                if let Some(s) = serialization.rewrites.get(&k) {
                    return s.clone();
                }
                let cones = n
                    .cones()
                    .iter()
                    .map(|c| {
                        c.dehydrate(serialization);
                        c.key()
                    })
                    .collect();
                let rn = RewriteSer::RN {
                    dimension: n.dimension(),
                    cones,
                };
                serialization.rewrites.insert(k, rn.clone());
                rn
            }
        }
    }
}

impl From<Serialization> for Vec<u8> {
    fn from(ser: Serialization) -> Self {
        rmp_serde::to_vec(&ser).unwrap()
    }
}

impl From<Vec<u8>> for Serialization {
    fn from(bs: Vec<u8>) -> Self {
        rmp_serde::from_read_ref(&bs).unwrap()
    }
}

impl From<Signature> for Serialization {
    fn from(signature: Signature) -> Self {
        let mut serialization = Default::default();
        for (g, d) in signature.into_iter() {
            d.dehydrate(&mut serialization);
            serialization.signature.insert(g, d.key());
        }
        serialization
    }
}

// Serialization of a 'pointed' signature with a distinguished diagram
impl From<(Signature, Diagram)> for Serialization {
    fn from((sig, d): (Signature, Diagram)) -> Self {
        let mut serialization = Serialization::from(sig);
        d.dehydrate(&mut serialization);
        serialization
    }
}

trait Rehydrate<R> {
    fn rehydrate(&self, serialization: &Serialization) -> R;
}

impl Rehydrate<Diagram> for Key<Diagram> {
    fn rehydrate(&self, serialization: &Serialization) -> Diagram {
        serialization.diagrams[self].rehydrate(serialization)
    }
}

impl Rehydrate<Diagram> for DiagramSer {
    fn rehydrate(&self, serialization: &Serialization) -> Diagram {
        match self {
            DiagramSer::D0(g) => Diagram::from(*g),
            DiagramSer::DN { source, cospans } => DiagramN::new_unsafe(
                source.rehydrate(serialization),
                cospans
                    .iter()
                    .map(|cs| cs.rehydrate(serialization))
                    .collect(),
            )
            .into(),
        }
    }
}

impl Rehydrate<Rewrite> for Key<Rewrite> {
    fn rehydrate(&self, serialization: &Serialization) -> Rewrite {
        serialization.rewrites[self].rehydrate(serialization)
    }
}

impl Rehydrate<Rewrite> for RewriteSer {
    fn rehydrate(&self, serialization: &Serialization) -> Rewrite {
        match self {
            RewriteSer::R0(r) => Rewrite0(*r).into(),
            RewriteSer::RN { dimension, cones } => RewriteN::new(
                *dimension,
                cones.iter().map(|c| c.rehydrate(serialization)).collect(),
            )
            .into(),
        }
    }
}

impl Rehydrate<Cospan> for CospanSer {
    fn rehydrate(&self, serialization: &Serialization) -> Cospan {
        Cospan {
            forward: self.forward.rehydrate(serialization),
            backward: self.backward.rehydrate(serialization),
        }
    }
}

impl Rehydrate<Cone> for Key<Cone> {
    fn rehydrate(&self, serialization: &Serialization) -> Cone {
        serialization.cones[self].rehydrate(serialization)
    }
}

impl Rehydrate<Cone> for ConeSer {
    fn rehydrate(&self, serialization: &Serialization) -> Cone {
        Cone {
            index: self.index,
            source: self
                .source
                .iter()
                .map(|s| s.rehydrate(serialization))
                .collect(),
            target: self.target.rehydrate(serialization),
            slices: self
                .slices
                .iter()
                .map(|s| s.rehydrate(serialization))
                .collect(),
        }
    }
}

impl From<Serialization> for Signature {
    fn from(ser: Serialization) -> Self {
        let mut signature: Signature = Default::default();
        for (g, d) in ser.signature.iter() {
            signature.insert(*g, d.rehydrate(&ser));
        }
        signature
    }
}
