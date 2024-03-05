use std::{
    cell::RefCell,
    cmp::Ordering,
    fmt,
    iter::Product,
    ops::{Index, IndexMut, Mul},
};

use hashconsing::{HConsed, HConsign, HashConsign};
use homotopy_common::{hash::FastHashMap, idx::Idx};
use im::OrdSet;
use itertools::Either;
use serde::{Deserialize, Serialize};
use thiserror::Error;

thread_local! {
    static LABEL_FACTORY: RefCell<HConsign<LabelInternal>> =
        RefCell::new(HConsign::with_capacity(37));
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Generator {
    pub id: usize,
    pub dimension: usize,
}

impl Generator {
    /// Creates a new generator with the given id and dimension.
    pub const fn new(id: usize, dimension: usize) -> Self {
        Self { id, dimension }
    }

    /// Increments the dimension by one.
    #[must_use]
    pub const fn suspended(self) -> Self {
        Self {
            id: self.id,
            dimension: self.dimension + 1,
        }
    }
}

impl fmt::Debug for Generator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}:{}", self.id, self.dimension))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum Boundary {
    Source,
    Target,
}

impl Boundary {
    /// The opposite boundary.
    #[must_use]
    pub const fn flip(self) -> Self {
        match self {
            Self::Source => Self::Target,
            Self::Target => Self::Source,
        }
    }
}

impl fmt::Display for Boundary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source => f.write_str("S"),
            Self::Target => f.write_str("T"),
        }
    }
}

pub type RegularHeight = usize;
pub type SingularHeight = usize;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum Height {
    Regular(RegularHeight),
    Singular(SingularHeight),
}

impl Height {
    /// Create an iterator over all heights in a diagram of a specified size.
    ///
    /// # Examples
    ///
    /// ```
    /// # use homotopy_core::common::Height;
    /// # use homotopy_core::common::Height::*;
    /// assert_eq!(
    ///     Height::for_size(1).collect::<Vec<_>>(),
    ///     vec![Regular(0), Singular(0), Regular(1)]
    /// )
    /// ```
    pub fn for_size(size: usize) -> impl DoubleEndedIterator<Item = Self> {
        (0..2 * size + 1).map(Self::from)
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

impl Idx for Height {
    fn index(&self) -> usize {
        usize::from(*self)
    }

    fn new(index: usize) -> Self {
        Self::from(index)
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

impl fmt::Display for Height {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Regular(i) => write!(f, "R{}", *i),
            Self::Singular(i) => write!(f, "S{}", *i),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SliceIndex {
    Boundary(Boundary),
    Interior(Height),
}

impl SliceIndex {
    /// Create an iterator over all slice indices in a diagram of a specified size.
    ///
    /// # Examples
    ///
    /// ```
    /// # use homotopy_core::common::Boundary::*;
    /// # use homotopy_core::common::Height::*;
    /// # use homotopy_core::common::SliceIndex;
    /// # use homotopy_core::common::SliceIndex::*;
    /// assert_eq!(
    ///     SliceIndex::for_size(1).collect::<Vec<_>>(),
    ///     vec![
    ///         Boundary(Source),
    ///         Interior(Regular(0)),
    ///         Interior(Singular(0)),
    ///         Interior(Regular(1)),
    ///         Boundary(Target)
    ///     ],
    /// )
    /// ```
    pub fn for_size(size: usize) -> impl DoubleEndedIterator<Item = Self> {
        use std::iter::once;
        once(Boundary::Source.into())
            .chain(Height::for_size(size).map(SliceIndex::Interior))
            .chain(once(Boundary::Target.into()))
    }

    /// The next slice in a diagram of a given size.
    ///
    /// # Examples
    ///
    /// ```
    /// # use homotopy_core::common::Boundary::*;
    /// # use homotopy_core::common::Height::*;
    /// # use homotopy_core::common::SliceIndex::*;
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
    /// # use homotopy_core::common::Boundary::*;
    /// # use homotopy_core::common::Height::*;
    /// # use homotopy_core::common::SliceIndex::*;
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
            Self::Interior(Regular(0)) => Some(Boundary::Source.into()),
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
/// # use homotopy_core::common::Height::*;
/// # use homotopy_core::common::SliceIndex::*;
/// assert!(Boundary(Source) < Interior(Regular(0)));
/// assert!(Interior(Regular(10)) < Boundary(Target));
/// ```
impl Ord for SliceIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use SliceIndex::{Boundary, Interior};

        use self::Boundary::{Source, Target};

        match (self, other) {
            (Boundary(x), Boundary(y)) => x.cmp(y),
            (Interior(x), Interior(y)) => x.cmp(y),
            (Boundary(Source), Interior(_)) | (Interior(_), Boundary(Target)) => Ordering::Less,
            (Boundary(Target), Interior(_)) | (Interior(_), Boundary(Source)) => Ordering::Greater,
        }
    }
}

impl PartialOrd for SliceIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for SliceIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boundary(b) => write!(f, "{b}"),
            Self::Interior(h) => write!(f, "{h}"),
        }
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

/// A boundary path represents a boundary of a diagram.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct BoundaryPath(pub Boundary, pub usize);

impl BoundaryPath {
    /// Given a path in a diagram, split it into a boundary path and an interior path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use homotopy_core::common::Boundary::*;
    /// # use homotopy_core::common::BoundaryPath;
    /// # use homotopy_core::common::Height::*;
    /// # use homotopy_core::common::SliceIndex::*;
    /// assert_eq!(
    ///     BoundaryPath::split(&[
    ///         Interior(Regular(0)),
    ///         Boundary(Source),
    ///         Interior(Singular(0))
    ///     ]),
    ///     (Some(BoundaryPath(Source, 1)), vec![Singular(0)]),
    /// )
    /// ```
    pub fn split(path: &[SliceIndex]) -> (Option<Self>, Vec<Height>) {
        use SliceIndex::{Boundary, Interior};

        let mut boundary_path: Option<Self> = None;
        let mut interior_path = Vec::new();

        for height in path.iter().rev() {
            match (&mut boundary_path, height) {
                (Some(bp), _) => bp.1 += 1,
                (None, Boundary(b)) => boundary_path = Some(Self(*b, 0)),
                (None, Interior(h)) => interior_path.insert(0, *h),
            }
        }

        (boundary_path, interior_path)
    }

    #[inline]
    pub const fn boundary(self) -> Boundary {
        self.0
    }

    #[inline]
    pub const fn depth(self) -> usize {
        self.1
    }
}

impl From<Boundary> for BoundaryPath {
    fn from(boundary: Boundary) -> Self {
        Self(boundary, 0)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum Direction {
    Forward,
    Backward,
}

pub trait WithDirection: DoubleEndedIterator {
    fn with_direction(self, direction: Direction) -> impl DoubleEndedIterator<Item = Self::Item>
    where
        Self: Sized,
    {
        match direction {
            Direction::Forward => Either::Left(self),
            Direction::Backward => Either::Right(self.rev()),
        }
    }
}

impl<T: DoubleEndedIterator> WithDirection for T {}

#[derive(Debug, Error)]
#[error("invalid dimension")]
pub struct DimensionError;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum Mode {
    Deep,
    Shallow,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Orientation {
    Negative,
    Zero,
    Positive,
}

impl Mul for Orientation {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        use Orientation::{Negative, Positive, Zero};
        match (self, rhs) {
            (Zero, _) | (_, Zero) => Zero,
            (Positive, _) => rhs,
            (_, Positive) => self,
            (Negative, Negative) => Positive,
        }
    }
}

impl Product for Orientation {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::Positive, Mul::mul)
    }
}

impl fmt::Debug for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negative => write!(f, "-"),
            Self::Zero => write!(f, "0"),
            Self::Positive => write!(f, "+"),
        }
    }
}

impl From<Direction> for Orientation {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Forward => Self::Positive,
            Direction::Backward => Self::Negative,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Label(HConsed<LabelInternal>);

impl Serialize for Label {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct("Label", self.0.get())
    }
}

impl<'de> Deserialize<'de> for Label {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer)
            .map(|l| Self(LABEL_FACTORY.with_borrow_mut(|factory| factory.mk(l))))
    }
}

impl fmt::Debug for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Label")
            .field(&self.0 .0)
            .field(&self.0 .1)
            .finish()
    }
}

impl Label {
    pub fn new(boundary_path: BoundaryPath, coords: OrdSet<Vec<Height>>) -> Self {
        Self(
            LABEL_FACTORY
                .with_borrow_mut(|factory| factory.mk(LabelInternal(boundary_path, coords))),
        )
    }

    pub fn boundary_path(&self) -> BoundaryPath {
        self.0 .0
    }

    pub fn coords(&self) -> OrdSet<Vec<Height>> {
        self.0 .1.clone()
    }

    pub(crate) fn collect_garbage() {
        LABEL_FACTORY.with_borrow_mut(|factory| factory.collect_to_fit());
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
struct LabelInternal(BoundaryPath, OrdSet<Vec<Height>>);

pub(crate) type LabelIdentifications = FastHashMap<Vec<Height>, OrdSet<Vec<Height>>>;
