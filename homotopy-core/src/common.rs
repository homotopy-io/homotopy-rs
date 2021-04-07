use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::{cmp::Ordering, iter::FusedIterator};
use thiserror::Error;

#[derive(PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Generator {
    pub id: usize,
    pub dimension: usize,
}

impl Generator {
    pub fn new(id: usize, dimension: usize) -> Self {
        Self { id, dimension }
    }
}

impl fmt::Debug for Generator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}:{}", self.id, self.dimension))
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Boundary {
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "target")]
    Target,
}

impl Boundary {
    pub fn flip(self) -> Self {
        match self {
            Self::Source => Self::Target,
            Self::Target => Self::Source,
        }
    }
}

pub type SingularHeight = usize;

pub type RegularHeight = usize;

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub enum Height {
    Singular(SingularHeight),
    Regular(RegularHeight),
}

impl Height {
    pub fn to_int(self) -> usize {
        match self {
            Self::Regular(i) => i * 2,
            Self::Singular(i) => i * 2 + 1,
        }
    }

    pub fn from_int(h: usize) -> Self {
        if h % 2 == 0 {
            Self::Regular(h / 2)
        } else {
            Self::Singular((h - 1) / 2)
        }
    }
}

impl PartialOrd for Height {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Height {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_int().cmp(&other.to_int())
    }
}

impl Serialize for Height {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.to_int() as u32)
    }
}

impl<'de> Deserialize<'de> for Height {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::from_int(u32::deserialize(deserializer)? as usize))
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SliceIndex {
    Boundary(Boundary),
    Interior(Height),
}

impl SliceIndex {
    /// Create an iterator over all slice indices in a diagram of a specified size.
    pub fn for_size(size: usize) -> SliceIndexIterator {
        SliceIndexIterator::new(size)
    }

    pub fn to_int(self, size: usize) -> isize {
        match self {
            Self::Boundary(Boundary::Source) => -1,
            Self::Boundary(Boundary::Target) => size as isize * 2 + 1,
            Self::Interior(height) => height.to_int() as isize,
        }
    }

    pub fn from_int(h: isize, size: usize) -> Self {
        if h < 0 {
            Boundary::Source.into()
        } else if h > 2 * size as isize {
            Boundary::Target.into()
        } else {
            Height::from_int(h as usize).into()
        }
    }
}

impl Ord for SliceIndex {
    /// ```
    /// # use homotopy_core::common::Boundary::*;
    /// # use homotopy_core::common::SliceIndex::*;
    /// # use homotopy_core::common::Height::*;
    /// assert!(Boundary(Source) < Interior(Regular(0)));
    /// assert!(Interior(Regular(10)) < Boundary(Target));
    /// ```
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use self::Boundary::{Source, Target};
        use SliceIndex::{Boundary, Interior};
        match (self, other) {
            (Boundary(Source), Boundary(Source)) | (Boundary(Target), Boundary(Target)) => {
                Ordering::Equal
            }
            (Boundary(Source), _) | (Interior(_), Boundary(Target)) => Ordering::Less,
            (Interior(_), Boundary(Source)) | (Boundary(Target), _) => Ordering::Greater,
            (Interior(x), Interior(y)) => x.cmp(y),
        }
    }
}

impl PartialOrd for SliceIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<Height> for SliceIndex {
    fn from(height: Height) -> Self {
        Self::Interior(height)
    }
}

impl From<Boundary> for SliceIndex {
    fn from(boundary: Boundary) -> Self {
        Self::Boundary(boundary)
    }
}

pub struct SliceIndexIterator {
    size: usize,
    next: Option<SliceIndex>,
}

impl SliceIndexIterator {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            next: Some(SliceIndex::Boundary(Boundary::Source)),
        }
    }
}

impl Iterator for SliceIndexIterator {
    type Item = SliceIndex;

    fn next(&mut self) -> Option<Self::Item> {
        use self::Boundary::{Source, Target};
        use Height::{Regular, Singular};
        use SliceIndex::{Boundary, Interior};

        let next = match self.next {
            Some(Boundary(Source)) => Some(Height::Regular(0).into()),
            Some(Boundary(Target)) => None,
            Some(Interior(Regular(i))) if i >= self.size => Some(Boundary(Target)),
            Some(Interior(Regular(i))) => Some(Interior(Singular(i))),
            Some(Interior(Singular(i))) => Some(Interior(Regular(i + 1))),
            None => None,
        };

        std::mem::replace(&mut self.next, next)
    }
}

impl FusedIterator for SliceIndexIterator {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Debug, Error)]
#[error("invalid dimension")]
pub struct DimensionError;
