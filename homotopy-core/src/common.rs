use std::{
    cmp::Ordering,
    fmt,
    iter::FusedIterator,
    ops::{Index, IndexMut},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[derive(PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Generator {
    pub dimension: usize,
    pub id: usize,
    pub orientation: isize,
}

impl Generator {
    pub fn new(id: usize, dimension: usize) -> Self {
        Self {
            dimension,
            id,
            orientation: 1,
        }
    }

    pub fn inverse(self) -> Self {
        Self {
            orientation: -self.orientation,
            ..self
        }
    }
}

impl fmt::Debug for Generator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}:{}", self.id, self.dimension))
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Boundary {
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "target")]
    Target,
}

impl Boundary {
    #[must_use]
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

#[cfg(feature = "fuzz")]
impl<'a> arbitrary::Arbitrary<'a> for Height {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let h = u.int_in_range(0..=1023)?;
        u.choose(&[Height::Singular(h), Height::Regular(h)])
            .map(|s| s.clone())
    }
}

impl From<Height> for usize {
    fn from(h: Height) -> Self {
        match h {
            Height::Regular(i) => i * 2,
            Height::Singular(i) => i * 2 + 1,
        }
    }
}

impl From<usize> for Height {
    fn from(h: usize) -> Self {
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
        usize::from(*self).cmp(&usize::from(*other))
    }
}

impl Serialize for Height {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(usize::from(*self) as u32)
    }
}

impl<'de> Deserialize<'de> for Height {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok((u32::deserialize(deserializer)? as usize).into())
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
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
            Self::Interior(height) => usize::from(height) as isize,
        }
    }

    pub fn from_int(h: isize, size: usize) -> Self {
        if h < 0 {
            Boundary::Source.into()
        } else if h > 2 * size as isize {
            Boundary::Target.into()
        } else {
            Height::from(h as usize).into()
        }
    }

    /// The next slice in a diagram of a given size.
    ///
    /// # Examples
    ///
    /// ```
    /// # use homotopy_core::common::SliceIndex::*;
    /// # use homotopy_core::common::Height::*;
    /// # use homotopy_core::common::Boundary::*;
    /// assert_eq!(Boundary(Source).next(1), Some(Interior(Regular(0))));
    /// assert_eq!(Interior(Regular(0)).next(1), Some(Interior(Singular(0))));
    /// assert_eq!(Interior(Singular(0)).next(1), Some(Interior(Regular(1))));
    /// assert_eq!(Interior(Regular(1)).next(1), Some(Boundary(Target)));
    /// assert_eq!(Boundary(Target).next(1), None);
    /// ```
    pub fn next(self, size: usize) -> Option<Self> {
        use Height::{Regular, Singular};

        match self {
            Self::Boundary(Boundary::Source) => Some(Regular(0).into()),
            Self::Interior(Regular(i)) if i == size => Some(Boundary::Target.into()),
            Self::Interior(Regular(i)) => Some(Singular(i).into()),
            Self::Interior(Singular(i)) => Some(Regular(i + 1).into()),
            Self::Boundary(Boundary::Target) => None,
        }
    }

    /// The previous slice in a diagram of a given size.
    ///
    /// # Examples
    ///
    /// ```
    /// # use homotopy_core::common::SliceIndex::*;
    /// # use homotopy_core::common::Height::*;
    /// # use homotopy_core::common::Boundary::*;
    /// assert_eq!(Boundary(Source).prev(1), None);
    /// assert_eq!(Interior(Regular(0)).prev(1), Some(Boundary(Source)));
    /// assert_eq!(Interior(Singular(0)).prev(1), Some(Interior(Regular(0))));
    /// assert_eq!(Interior(Regular(1)).prev(1), Some(Interior(Singular(0))));
    /// assert_eq!(Boundary(Target).prev(1), Some(Interior(Regular(1))));
    /// ```
    pub fn prev(self, size: usize) -> Option<Self> {
        use Height::{Regular, Singular};

        match self {
            Self::Boundary(Boundary::Source) => None,
            Self::Interior(Regular(i)) if i == 0 => Some(Boundary::Source.into()),
            Self::Interior(Regular(i)) => Some(Singular(i - 1).into()),
            Self::Interior(Singular(i)) => Some(Regular(i).into()),
            Self::Boundary(Boundary::Target) => Some(Regular(size).into()),
        }
    }

    pub fn step(self, size: usize, direction: Direction) -> Option<Self> {
        match direction {
            Direction::Forward => self.next(size),
            Direction::Backward => self.prev(size),
        }
    }
}

/// ```
/// # use homotopy_core::common::Boundary::*;
/// # use homotopy_core::common::SliceIndex::*;
/// # use homotopy_core::common::Height::*;
/// assert!(Boundary(Source) < Interior(Regular(0)));
/// assert!(Interior(Regular(10)) < Boundary(Target));
/// ```
impl Ord for SliceIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use SliceIndex::{Boundary, Interior};

        use self::Boundary::{Source, Target};
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
    start: SliceIndex,
    stop: SliceIndex,
    done: bool,
}

impl SliceIndexIterator {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            start: Boundary::Source.into(),
            stop: Boundary::Target.into(),
            done: false,
        }
    }
}

impl Iterator for SliceIndexIterator {
    type Item = SliceIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.start == self.stop;
            let next = self
                .start
                .next(self.size)
                .unwrap_or(SliceIndex::Boundary(Boundary::Target));
            Some(std::mem::replace(&mut self.start, next))
        }
    }
}

impl DoubleEndedIterator for SliceIndexIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = self.start == self.stop;
            let next = self
                .stop
                .prev(self.size)
                .unwrap_or(SliceIndex::Boundary(Boundary::Source));
            Some(std::mem::replace(&mut self.stop, next))
        }
    }
}

impl FusedIterator for SliceIndexIterator {}

impl<T> Index<SliceIndex> for Vec<T> {
    type Output = T;

    fn index(&self, index: SliceIndex) -> &Self::Output {
        match index {
            SliceIndex::Boundary(Boundary::Source) => self.first(),
            SliceIndex::Boundary(Boundary::Target) => self.last(),
            SliceIndex::Interior(height) => self.get(usize::from(height) + 1),
        }
        .unwrap()
    }
}

impl<T> IndexMut<SliceIndex> for Vec<T> {
    fn index_mut(&mut self, index: SliceIndex) -> &mut Self::Output {
        match index {
            SliceIndex::Boundary(Boundary::Source) => self.first_mut(),
            SliceIndex::Boundary(Boundary::Target) => self.last_mut(),
            SliceIndex::Interior(height) => self.get_mut(usize::from(height) + 1),
        }
        .unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Debug, Error)]
#[error("invalid dimension")]
pub struct DimensionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mode {
    Deep,
    Shallow,
}
