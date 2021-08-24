use std::convert::{From, Into, TryFrom};
use std::fmt;
use std::hash::Hash;
use std::ops::{Deref, Range};
use std::{cell::RefCell, cmp::Ordering};

use thiserror::Error;

use hashconsing::{HConsed, HConsign, HashConsign};

use crate::common::{DimensionError, Generator, SingularHeight};
use crate::diagram::Diagram;
use crate::util::{first_max_generator, CachedCell, Hasher};
use crate::Boundary;

thread_local! {
    static REWRITE_FACTORY: RefCell<HConsign<RewriteInternal<DefaultAllocator>, Hasher>> =
        RefCell::new(HConsign::with_capacity_and_hasher(37, Hasher::default()));

    static CONE_FACTORY: RefCell<HConsign<ConeInternal<DefaultAllocator>, Hasher>> =
        RefCell::new(HConsign::with_capacity_and_hasher(37, Hasher::default()));
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DefaultAllocator;

pub type Cospan = GenericCospan<DefaultAllocator>;

pub type Cone = GenericCone<DefaultAllocator>;

pub type RewriteN = GenericRewriteN<DefaultAllocator>;
pub type Rewrite = GenericRewrite<DefaultAllocator>;

#[derive(Debug, Error)]
pub enum CompositionError {
    #[error("can't compose rewrites of dimensions {0} and {1}")]
    Dimension(usize, usize),

    #[error("failed to compose incompatible rewrites")]
    Incompatible,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct GenericCospan<A>
where
    A: RewriteAllocator,
{
    pub forward: GenericRewrite<A>,
    pub backward: GenericRewrite<A>,
}

#[derive(Clone)]
pub struct RewriteInternal<A>
where
    A: RewriteAllocator,
{
    dimension: usize,
    cones: Vec<GenericCone<A>>,
    max_generator_source: CachedCell<Option<Generator>>,
    max_generator_target: CachedCell<Option<Generator>>,
    payload: A::Payload,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct Rewrite0(pub(crate) Option<(Generator, Generator)>);

#[derive(PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct GenericRewriteN<A>(A::RewriteCell)
where
    A: RewriteAllocator;

#[derive(PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub enum GenericRewrite<A>
where
    A: RewriteAllocator,
{
    Rewrite0(Rewrite0),
    RewriteN(GenericRewriteN<A>),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ConeInternal<A>
where
    A: RewriteAllocator,
{
    pub(crate) source: Vec<GenericCospan<A>>,
    pub(crate) target: GenericCospan<A>,
    pub(crate) slices: Vec<GenericRewrite<A>>,
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct GenericCone<A>
where
    A: RewriteAllocator,
{
    pub(crate) index: usize,
    pub(crate) internal: A::ConeCell,
}

pub trait RewriteAllocator: Copy + Eq + Hash + fmt::Debug + Sized {
    type Payload: Composable;

    type RewriteCell: Deref<Target = RewriteInternal<Self>> + Clone + Eq + Ord + Hash;
    type ConeCell: Deref<Target = ConeInternal<Self>> + Clone + Eq + Ord + Hash;

    fn mk_rewrite(internal: RewriteInternal<Self>) -> Self::RewriteCell;

    fn mk_cone(internal: ConeInternal<Self>) -> Self::ConeCell;

    fn collect_garbage();
}

pub trait Composable: Clone + Eq + Hash + fmt::Debug {
    fn compose<A>(f: &GenericRewriteN<A>, g: &GenericRewriteN<A>) -> Self
    where
        A: RewriteAllocator<Payload = Self>;
}

impl<A> GenericCospan<A>
where
    A: RewriteAllocator,
{
    #[inline]
    pub(crate) fn pad(&self, embedding: &[usize]) -> Self {
        let forward = self.forward.pad(embedding);
        let backward = self.backward.pad(embedding);
        Self { forward, backward }
    }

    #[inline]
    pub fn is_identity(&self) -> bool {
        self.forward.is_identity() && self.backward.is_identity()
    }

    pub(crate) fn max_generator(&self) -> Option<Generator> {
        let generators = [
            self.forward.max_generator(Boundary::Source),
            self.forward.max_generator(Boundary::Target),
            self.backward.max_generator(Boundary::Target),
            self.backward.max_generator(Boundary::Source),
        ];

        first_max_generator(generators.iter().copied().flatten(), None)
    }
}

impl<A> fmt::Debug for GenericRewrite<A>
where
    A: RewriteAllocator,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RewriteN(r) => r.fmt(f),
            Self::Rewrite0(r) => r.fmt(f),
        }
    }
}

impl<A> From<GenericRewriteN<A>> for GenericRewrite<A>
where
    A: RewriteAllocator,
{
    #[inline]
    fn from(r: GenericRewriteN<A>) -> Self {
        Self::RewriteN(r)
    }
}

impl<A> From<Rewrite0> for GenericRewrite<A>
where
    A: RewriteAllocator,
{
    #[inline]
    fn from(r: Rewrite0) -> Self {
        Self::Rewrite0(r)
    }
}

impl<A> TryFrom<GenericRewrite<A>> for GenericRewriteN<A>
where
    A: RewriteAllocator,
{
    type Error = DimensionError;

    #[inline]
    fn try_from(value: GenericRewrite<A>) -> Result<Self, Self::Error> {
        match value {
            GenericRewrite::Rewrite0(_) => Err(DimensionError),
            GenericRewrite::RewriteN(r) => Ok(r),
        }
    }
}

impl<'a, A> TryFrom<&'a GenericRewrite<A>> for &'a GenericRewriteN<A>
where
    A: RewriteAllocator,
{
    type Error = DimensionError;

    #[inline]
    fn try_from(value: &'a GenericRewrite<A>) -> Result<Self, Self::Error> {
        match value {
            GenericRewrite::Rewrite0(_) => Err(DimensionError),
            GenericRewrite::RewriteN(r) => Ok(r),
        }
    }
}

impl<A> TryFrom<GenericRewrite<A>> for Rewrite0
where
    A: RewriteAllocator,
{
    type Error = DimensionError;

    fn try_from(value: GenericRewrite<A>) -> Result<Self, Self::Error> {
        match value {
            GenericRewrite::Rewrite0(r) => Ok(r),
            GenericRewrite::RewriteN(_) => Err(DimensionError),
        }
    }
}

impl<A, T> GenericRewrite<A>
where
    A: RewriteAllocator<Payload = T>,
    T: Default,
{
    #[inline]
    pub fn identity(dimension: usize) -> Self {
        Self::identity_with_payload(dimension, &Default::default())
    }
}

impl GenericRewrite<DefaultAllocator> {
    pub fn cone_over_generator(generator: Generator, base: Diagram) -> Self {
        match base {
            Diagram::Diagram0(base) => Rewrite0::new(base, generator).into(),
            Diagram::DiagramN(base) => GenericRewriteN::new(
                base.dimension(),
                vec![GenericCone::new(
                    0,
                    base.cospans().to_vec(),
                    GenericCospan {
                        forward: Self::cone_over_generator(generator, base.source()),
                        backward: Self::cone_over_generator(generator, base.target()),
                    },
                    base.singular_slices()
                        .into_iter()
                        .map(|slice| Self::cone_over_generator(generator, slice))
                        .collect(),
                )],
            )
            .into(),
        }
    }
}

impl<A> GenericRewrite<A>
where
    A: RewriteAllocator,
{
    #[inline]
    pub fn identity_with_payload(dimension: usize, payload: &A::Payload) -> Self {
        match dimension {
            0 => Rewrite0::identity().into(),
            _ => GenericRewriteN::identity_with_payload(dimension, payload).into(),
        }
    }

    #[inline]
    pub fn dimension(&self) -> usize {
        match self {
            Self::Rewrite0(_) => 0,
            Self::RewriteN(r) => r.dimension(),
        }
    }

    #[inline]
    pub fn is_identity(&self) -> bool {
        match self {
            Self::Rewrite0(r) => r.is_identity(),
            Self::RewriteN(r) => r.is_identity(),
        }
    }

    #[inline]
    pub fn check_well_formed(&self) -> Result<(), MalformedRewrite> {
        match self {
            Self::Rewrite0(_) => Ok(()),
            Self::RewriteN(r) => r.check_well_formed(),
        }
    }

    #[inline]
    pub fn compose(&self, g: &Self) -> Result<Self, CompositionError> {
        match (self, g) {
            (Self::Rewrite0(ref f), Self::Rewrite0(ref g)) => Ok(f.compose(g)?.into()),
            (Self::RewriteN(ref f), Self::RewriteN(ref g)) => Ok(f.compose(g)?.into()),
            (f, g) => Err(CompositionError::Dimension(f.dimension(), g.dimension())),
        }
    }

    pub(crate) fn max_generator(&self, boundary: Boundary) -> Option<Generator> {
        match self {
            Self::Rewrite0(r) => r.max_generator(boundary),
            Self::RewriteN(r) => r.max_generator(boundary),
        }
    }

    pub(crate) fn pad(&self, embedding: &[usize]) -> Self {
        match *self {
            Self::Rewrite0(ref r) => Self::Rewrite0(*r),
            Self::RewriteN(ref r) => Self::RewriteN(r.pad(embedding)),
        }
    }
}

impl fmt::Debug for Rewrite0 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some((s, t)) => f.debug_tuple("Rewrite0").field(&s).field(&t).finish(),
            None => f.debug_struct("Rewrite0").finish(),
        }
    }
}

impl Rewrite0 {
    pub fn new(source: Generator, target: Generator) -> Self {
        assert!(source.dimension <= target.dimension);
        if source == target {
            Self(None)
        } else {
            Self(Some((source, target)))
        }
    }

    pub fn identity() -> Self {
        Self(None)
    }

    pub fn is_identity(&self) -> bool {
        self.0.is_none()
    }

    pub fn compose(&self, g: &Self) -> Result<Self, CompositionError> {
        match (self.0, g.0) {
            (Some((f_s, f_t)), Some((g_s, g_t))) => {
                if f_t == g_s {
                    Ok(Self(Some((f_s, g_t))))
                } else {
                    Err(CompositionError::Incompatible)
                }
            }
            (Some(_), None) => Ok(*self),
            (None, Some(_)) => Ok(*g),
            (None, None) => Ok(Self(None)),
        }
    }

    pub fn source(&self) -> Option<Generator> {
        self.0.map(|(source, _)| source)
    }

    pub fn target(&self) -> Option<Generator> {
        self.0.map(|(_, target)| target)
    }

    pub(crate) fn max_generator(&self, boundary: Boundary) -> Option<Generator> {
        match boundary {
            Boundary::Source => self.source(),
            Boundary::Target => self.target(),
        }
    }
}

impl<A> fmt::Debug for GenericRewriteN<A>
where
    A: RewriteAllocator,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<A> PartialEq for RewriteInternal<A>
where
    A: RewriteAllocator,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.dimension == other.dimension
            && self.cones == other.cones
            && self.payload == other.payload
    }
}

impl<A> Eq for RewriteInternal<A> where A: RewriteAllocator {}

impl<A> Hash for RewriteInternal<A>
where
    A: RewriteAllocator,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.dimension.hash(state);
        self.cones.hash(state);
        self.payload.hash(state);
    }
}

impl<A, T> GenericRewriteN<A>
where
    A: RewriteAllocator<Payload = T>,
    T: Default,
{
    pub(crate) fn new(dimension: usize, cones: Vec<GenericCone<A>>) -> Self {
        Self::new_with_payload(dimension, cones, &Default::default())
    }

    #[inline]
    pub fn identity(dimension: usize) -> Self {
        Self::new(dimension, Vec::new())
    }

    pub(crate) fn make_degeneracy(dimension: usize, trivial_heights: &[SingularHeight]) -> Self {
        Self::make_degeneracy_with_payloads(
            dimension,
            trivial_heights,
            &Default::default(),
            |_, _| Default::default(),
        )
    }

    #[inline]
    pub fn from_slices(
        dimension: usize,
        source_cospans: &[GenericCospan<A>],
        target_cospans: &[GenericCospan<A>],
        slices: Vec<Vec<GenericRewrite<A>>>,
    ) -> Self {
        Self::from_slices_with_payload(
            dimension,
            source_cospans,
            target_cospans,
            slices,
            &Default::default(),
        )
    }

    #[inline]
    pub fn slice(&self, height: usize) -> GenericRewrite<A> {
        self.slice_with_payload(height, &Default::default())
    }

    #[inline]
    pub fn payload(&self) -> &A::Payload {
        &self.0.payload
    }
}

impl<A> GenericRewriteN<A>
where
    A: RewriteAllocator,
{
    pub(crate) fn new_with_payload(
        dimension: usize,
        mut cones: Vec<GenericCone<A>>,
        payload: &A::Payload,
    ) -> Self {
        if dimension == 0 {
            panic!("Can not create RewriteN of dimension zero.");
        }

        // Remove all identity cones. This is not only important to reduce memory consumption, but
        // it allows us the check if the rewrite is an identity by shallowly checking if it has any
        // cones.
        cones.retain(|cone| !cone.is_identity());

        let rewrite = Self(A::mk_rewrite(RewriteInternal {
            dimension,
            cones,
            max_generator_source: CachedCell::new(),
            max_generator_target: CachedCell::new(),
            payload: payload.clone(),
        }));
        debug_assert!(rewrite.is_well_formed());
        rewrite
    }

    pub(crate) fn collect_garbage() {
        A::collect_garbage();
    }

    pub(crate) fn cones(&self) -> &[GenericCone<A>] {
        &self.0.cones
    }

    pub(crate) fn pad(&self, embedding: &[usize]) -> Self {
        let cones = self
            .cones()
            .iter()
            .map(|cone| cone.pad(embedding))
            .collect();
        Self::new_with_payload(self.dimension(), cones, &self.0.payload)
    }

    #[inline]
    pub fn identity_with_payload(dimension: usize, payload: &A::Payload) -> Self {
        Self::new_with_payload(dimension, Vec::new(), payload)
    }

    #[inline]
    pub fn is_identity(&self) -> bool {
        self.0.cones.is_empty()
    }

    #[inline]
    pub fn check_well_formed(&self) -> Result<(), MalformedRewrite> {
        for cone in &self.0.cones {
            if cone.len() == 0 {
                if cone.internal.target.forward != cone.internal.target.backward {
                    return Err(MalformedRewrite::NotSingularity(cone.index));
                }
            } else {
                // Check that the subslices are well-formed.
                for (i, slice) in cone.internal.slices.iter().enumerate() {
                    slice
                        .check_well_formed()
                        .map_err(|e| MalformedRewrite::Slice(i, Box::new(e)))?;
                }

                // Check that the squares commute.
                let len = cone.len();

                let f = cone.internal.source[0]
                    .forward
                    .compose(&cone.internal.slices[0]);
                if f.is_err() || f.unwrap() != cone.internal.target.forward {
                    return Err(MalformedRewrite::NotCommutativeLeft(cone.index));
                }

                for i in 0..len - 1 {
                    let f = cone.internal.source[i]
                        .backward
                        .compose(&cone.internal.slices[i]);
                    let g = cone.internal.source[i + 1]
                        .forward
                        .compose(&cone.internal.slices[i + 1]);
                    if f.is_err() || g.is_err() || f.unwrap() != g.unwrap() {
                        return Err(MalformedRewrite::NotCommutativeMiddle(
                            cone.index + i,
                            cone.index + i + 1,
                        ));
                    }
                }

                let f = cone.internal.source[len - 1]
                    .backward
                    .compose(&cone.internal.slices[len - 1]);
                if f.is_err() || f.unwrap() != cone.internal.target.backward {
                    return Err(MalformedRewrite::NotCommutativeRight(len - 1));
                }
            }
        }

        Ok(())
    }

    pub(crate) fn make_degeneracy_with_payloads(
        dimension: usize,
        trivial_heights: &[SingularHeight],
        payload: &A::Payload,
        payloads: impl Fn(usize, usize) -> A::Payload,
    ) -> Self {
        let cones = trivial_heights
            .iter()
            .enumerate()
            .map(|(i, height)| {
                GenericCone::new(
                    height - i,
                    vec![],
                    GenericCospan {
                        forward: GenericRewrite::identity_with_payload(
                            dimension - 1,
                            &payloads(i, *height),
                        ),
                        backward: GenericRewrite::identity_with_payload(
                            dimension - 1,
                            &payloads(i, *height),
                        ),
                    },
                    vec![],
                )
            })
            .collect();

        Self::new_with_payload(dimension, cones, payload)
    }

    pub fn from_slices_with_payload(
        dimension: usize,
        source_cospans: &[GenericCospan<A>],
        target_cospans: &[GenericCospan<A>],
        slices: Vec<Vec<GenericRewrite<A>>>,
        payload: &A::Payload,
    ) -> Self {
        let mut cones = Vec::new();
        let mut index = 0;

        for (target, cone_slices) in slices.into_iter().enumerate() {
            let size = cone_slices.len();
            cones.push(GenericCone::new(
                index,
                source_cospans[index..index + size].to_vec(),
                target_cospans[target].clone(),
                cone_slices,
            ));
            index += size;
        }

        Self::new_with_payload(dimension, cones, payload)
    }

    pub fn dimension(&self) -> usize {
        self.0.dimension
    }

    pub fn targets(&self) -> Vec<usize> {
        let mut targets = Vec::new();
        let mut offset: isize = 0;

        for cone in self.cones() {
            targets.push((cone.index as isize + offset) as usize);
            offset += 1 - cone.len() as isize;
        }

        targets
    }

    pub(crate) fn cone_over_target(&self, height: usize) -> Option<&GenericCone<A>> {
        let mut offset: isize = 0;

        for cone in self.cones() {
            let target = (cone.index as isize + offset) as usize;

            if target == height {
                return Some(cone);
            }

            offset += 1 - cone.len() as isize;
        }

        None
    }

    pub fn slice_with_payload(&self, height: usize, payload: &A::Payload) -> GenericRewrite<A> {
        self.cones()
            .iter()
            .find(|cone| cone.index <= height && height < cone.index + cone.len())
            .map_or(
                GenericRewrite::identity_with_payload(self.dimension() - 1, payload),
                |cone| cone.internal.slices[height - cone.index].clone(),
            )
    }

    pub fn compose(&self, g: &Self) -> Result<Self, CompositionError> {
        if self.dimension() != g.dimension() {
            return Err(CompositionError::Dimension(self.dimension(), g.dimension()));
        }

        let mut offset = 0;
        let mut delayed_offset = 0;

        let mut f_cones: Vec<GenericCone<A>> = self.cones().iter().rev().cloned().collect();
        let mut g_cones: Vec<GenericCone<A>> = g.cones().iter().rev().cloned().collect();
        let mut cones: Vec<GenericCone<A>> = Vec::new();

        loop {
            match (f_cones.pop(), g_cones.pop()) {
                (None, None) => break,
                (Some(f_cone), None) => cones.push(f_cone.clone()),
                (None, Some(g_cone)) => {
                    let mut cone: GenericCone<A> = g_cone.clone();
                    cone.index = (cone.index as isize + offset) as usize;
                    offset += delayed_offset;
                    delayed_offset = 0;
                    cones.push(cone);
                }
                (Some(f_cone), Some(g_cone)) => {
                    let index = f_cone.index as isize - g_cone.index as isize - offset;

                    if index >= g_cone.len() as isize {
                        let mut cone = g_cone.clone();
                        cone.index = (cone.index as isize + offset) as usize;
                        cones.push(cone);
                        offset += delayed_offset;
                        delayed_offset = 0;
                        f_cones.push(f_cone);
                    } else if index < 0 {
                        cones.push(f_cone.clone());
                        g_cones.push(g_cone);
                        offset -= 1 - f_cone.len() as isize;
                    } else {
                        let index = index as usize;

                        if f_cone.internal.target != g_cone.internal.source[index] {
                            return Err(CompositionError::Incompatible);
                        }

                        let mut source = vec![];
                        source.extend(g_cone.internal.source[..index].iter().cloned());
                        source.extend(f_cone.internal.source.iter().cloned());
                        source.extend(g_cone.internal.source[index + 1..].iter().cloned());

                        let g_slice = &g_cone.internal.slices[index];
                        let mut slices = vec![];
                        slices.extend(g_cone.internal.slices[..index].iter().cloned());
                        slices.extend(
                            f_cone
                                .internal
                                .slices
                                .iter()
                                .map(|f_slice| f_slice.compose(g_slice))
                                .collect::<Result<Vec<_>, _>>()?,
                        );
                        slices.extend(g_cone.internal.slices[index + 1..].iter().cloned());

                        delayed_offset -= 1 - f_cone.len() as isize;

                        g_cones.push(GenericCone::new(
                            g_cone.index,
                            source,
                            g_cone.internal.target.clone(),
                            slices,
                        ));
                    }
                }
            }
        }

        Ok(Self::new_with_payload(
            self.dimension(),
            cones,
            &A::Payload::compose::<A>(self, g),
        ))
    }

    pub fn singular_image(&self, index: usize) -> usize {
        let mut offset: isize = 0;

        for cone in self.cones() {
            if index < cone.index {
                return (index as isize + offset) as usize;
            } else if index < cone.index + cone.len() {
                return (cone.index as isize + offset) as usize;
            } else {
                offset += 1 - cone.len() as isize;
            }
        }

        (index as isize + offset) as usize
    }

    pub fn singular_preimage(&self, index: usize) -> Range<usize> {
        let mut offset: isize = 0;

        for cone in self.cones() {
            let adjusted = (index as isize - offset) as usize;
            match adjusted.cmp(&cone.index) {
                Ordering::Less => {
                    return adjusted..adjusted + 1;
                }
                Ordering::Equal => {
                    return cone.index..cone.index + cone.len();
                }
                Ordering::Greater => {
                    offset += 1 - cone.len() as isize;
                }
            }
        }

        let adjusted = (index as isize - offset) as usize;
        adjusted..adjusted + 1
    }

    pub fn regular_image(&self, index: usize) -> usize {
        let mut offset = 0;

        for cone in self.cones() {
            if index <= (cone.index as isize + offset) as usize {
                return (index as isize - offset) as usize;
            } else {
                offset += 1 - cone.len() as isize;
            }
        }

        (index as isize - offset) as usize
    }

    pub fn regular_preimage(&self, index: usize) -> Range<usize> {
        let mut offset = 0;

        for (cone_index, cone) in self.cones().iter().enumerate() {
            let start = (index as isize + offset) as usize;
            if cone.index > index || (cone.len() > 0 && cone.index == index) {
                return start..(start + 1);
            } else if cone.index == index && cone.len() == 0 {
                let length = self.cones()[cone_index..]
                    .iter()
                    .take_while(|cone| cone.index == index && cone.len() == 0)
                    .count();
                return start..(start + length + 1);
            } else if cone.index < index && index < cone.index + cone.len() {
                let start = (cone.index as isize + offset) as usize;
                return start..start;
            } else {
                offset += 1 - cone.len() as isize;
            }
        }

        let start = (index as isize + offset) as usize;
        start..(start + 1)
    }

    pub(crate) fn max_generator(&self, boundary: Boundary) -> Option<Generator> {
        match boundary {
            Boundary::Source => self.0.max_generator_source.compute(|| {
                first_max_generator(
                    self.cones()
                        .iter()
                        .flat_map(|cone| &cone.internal.source)
                        .filter_map(GenericCospan::max_generator),
                    None,
                )
            }),
            Boundary::Target => self.0.max_generator_target.compute(|| {
                first_max_generator(
                    self.cones()
                        .iter()
                        .filter_map(|cone| cone.internal.target.max_generator()),
                    None,
                )
            }),
        }
    }
}

impl<A> fmt::Debug for RewriteInternal<A>
where
    A: RewriteAllocator,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RewriteN")
            .field("dimension", &self.dimension)
            .field("cones", &self.cones)
            .field("payload", &self.payload)
            .finish()
    }
}

impl<A> fmt::Debug for GenericCone<A>
where
    A: RewriteAllocator,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cone")
            .field("index", &self.index)
            .field("internal", &*self.internal)
            .finish()
    }
}

impl<A> GenericCone<A>
where
    A: RewriteAllocator,
{
    pub(crate) fn new(
        index: usize,
        source: Vec<GenericCospan<A>>,
        target: GenericCospan<A>,
        slices: Vec<GenericRewrite<A>>,
    ) -> Self {
        Self {
            index,
            internal: A::mk_cone(ConeInternal {
                source,
                target,
                slices,
            }),
        }
    }

    pub(crate) fn collect_garbage() {
        A::collect_garbage();
    }

    pub(crate) fn is_identity(&self) -> bool {
        self.internal.slices.len() == 1
            && self.internal.source.len() == 1
            && self.internal.source[0] == self.internal.target
            && self.internal.slices[0].is_identity()
    }

    pub(crate) fn len(&self) -> usize {
        self.internal.source.len()
    }

    pub(crate) fn pad(&self, embedding: &[usize]) -> Self {
        match embedding.split_first() {
            Some((offset, rest)) => {
                let index = self.index + offset;
                let source = self.internal.source.iter().map(|c| c.pad(rest)).collect();
                let target = self.internal.target.pad(rest);
                let slices = self.internal.slices.iter().map(|r| r.pad(rest)).collect();
                Self::new(index, source, target, slices)
            }
            None => self.clone(),
        }
    }
}

impl Composable for () {
    #[inline]
    fn compose<A>(_: &GenericRewriteN<A>, _: &GenericRewriteN<A>) -> Self
    where
        A: RewriteAllocator<Payload = Self>,
    {
    }
}

impl RewriteAllocator for DefaultAllocator {
    type Payload = ();
    type RewriteCell = HConsed<RewriteInternal<Self>>;
    type ConeCell = HConsed<ConeInternal<Self>>;

    #[inline]
    fn mk_rewrite(internal: RewriteInternal<Self>) -> Self::RewriteCell {
        REWRITE_FACTORY.with(|factory| factory.borrow_mut().mk(internal))
    }

    #[inline]
    fn mk_cone(internal: ConeInternal<Self>) -> Self::ConeCell {
        CONE_FACTORY.with(|factory| factory.borrow_mut().mk(internal))
    }

    #[inline]
    fn collect_garbage() {
        REWRITE_FACTORY.with(|factory| factory.borrow_mut().collect_to_fit());
        CONE_FACTORY.with(|factory| factory.borrow_mut().collect_to_fit());
    }
}

#[derive(Debug, Error)]
pub enum MalformedRewrite {
    #[error("slice {0} is malformed: {1}")]
    Slice(usize, Box<MalformedRewrite>),

    #[error("slice {0} of target cannot be a singularity.")]
    NotSingularity(usize),

    #[error("square to the left of slice {0} does not commute.")]
    NotCommutativeLeft(usize),

    #[error("square to the right of slice {0} does not commute.")]
    NotCommutativeRight(usize),

    #[error("square between slices {0} and {1} does not commute.")]
    NotCommutativeMiddle(usize, usize),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rewrite_compose() {
        let x = Generator::new(0, 0);
        let f = Generator::new(1, 1);
        let g = Generator::new(2, 1);
        let h = Generator::new(3, 1);

        let first = RewriteN::from_slices(
            1,
            &[],
            &[
                Cospan {
                    forward: Rewrite0::new(x, f).into(),
                    backward: Rewrite0::new(x, f).into(),
                },
                Cospan {
                    forward: Rewrite0::new(x, g).into(),
                    backward: Rewrite0::new(x, g).into(),
                },
            ],
            vec![vec![], vec![]],
        );

        let second = RewriteN::from_slices(
            1,
            &[
                Cospan {
                    forward: Rewrite0::new(x, f).into(),
                    backward: Rewrite0::new(x, f).into(),
                },
                Cospan {
                    forward: Rewrite0::new(x, g).into(),
                    backward: Rewrite0::new(x, g).into(),
                },
            ],
            &[
                Cospan {
                    forward: Rewrite0::new(x, f).into(),
                    backward: Rewrite0::new(x, f).into(),
                },
                Cospan {
                    forward: Rewrite0::new(x, h).into(),
                    backward: Rewrite0::new(x, h).into(),
                },
            ],
            vec![
                vec![Rewrite0::identity().into()],
                vec![Rewrite0::new(g, h).into()],
            ],
        );

        let expected = RewriteN::from_slices(
            1,
            &[],
            &[
                Cospan {
                    forward: Rewrite0::new(x, f).into(),
                    backward: Rewrite0::new(x, f).into(),
                },
                Cospan {
                    forward: Rewrite0::new(x, h).into(),
                    backward: Rewrite0::new(x, h).into(),
                },
            ],
            vec![vec![], vec![]],
        );

        let actual = RewriteN::compose(&first, &second).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn rewrite_compose_2() {
        let x = Generator::new(0, 0);
        let f = Generator::new(1, 1);
        let g = Generator::new(2, 1);
        let h = Generator::new(3, 1);

        let first = RewriteN::from_slices(
            1,
            &[],
            &[Cospan {
                forward: Rewrite0::new(x, f).into(),
                backward: Rewrite0::new(x, f).into(),
            }],
            vec![vec![]],
        );

        let second = RewriteN::from_slices(
            1,
            &[Cospan {
                forward: Rewrite0::new(x, f).into(),
                backward: Rewrite0::new(x, f).into(),
            }],
            &[
                Cospan {
                    forward: Rewrite0::new(x, g).into(),
                    backward: Rewrite0::new(x, g).into(),
                },
                Cospan {
                    forward: Rewrite0::new(x, h).into(),
                    backward: Rewrite0::new(x, h).into(),
                },
            ],
            vec![vec![Rewrite0::new(f, g).into()], vec![]],
        );

        let expected = RewriteN::from_slices(
            1,
            &[],
            &[
                Cospan {
                    forward: Rewrite0::new(x, g).into(),
                    backward: Rewrite0::new(x, g).into(),
                },
                Cospan {
                    forward: Rewrite0::new(x, h).into(),
                    backward: Rewrite0::new(x, h).into(),
                },
            ],
            vec![vec![], vec![]],
        );

        let actual = RewriteN::compose(&first, &second).unwrap();

        assert_eq!(actual, expected);
    }
}
