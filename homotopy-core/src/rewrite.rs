use std::{
    cell::RefCell,
    cmp::Ordering,
    convert::{From, Into},
    fmt,
    hash::Hash,
    ops::Range,
};

use hashconsing::{HConsed, HConsign, HashConsign};
use once_cell::unsync::OnceCell;
// used for debugging only
use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Serialize,
};
use thiserror::Error;

use crate::{
    common::{DimensionError, Generator, Mode, RegularHeight, SingularHeight},
    diagram::Diagram,
    util::first_max_generator,
    Boundary, Height, SliceIndex,
};

thread_local! {
    static REWRITE_FACTORY: RefCell<HConsign<RewriteInternal>> =
        RefCell::new(HConsign::with_capacity(37));

    static CONE_FACTORY: RefCell<HConsign<ConeInternal>> =
        RefCell::new(HConsign::with_capacity(37));
}

#[derive(Clone, Debug, Error)]
pub enum CompositionError {
    #[error("can't compose rewrites of dimensions {0} and {1}")]
    Dimension(usize, usize),

    #[error("failed to compose incompatible rewrites")]
    Incompatible,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord, Serialize)]
pub struct Cospan {
    pub forward: Rewrite,
    pub backward: Rewrite,
}

#[derive(Clone, Serialize)]
pub struct RewriteInternal {
    dimension: usize,
    cones: Vec<Cone>,
    #[serde(skip_serializing)]
    max_generator_source: OnceCell<Option<Generator>>,
    #[serde(skip_serializing)]
    max_generator_target: OnceCell<Option<Generator>>,
}

#[derive(PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct Rewrite0(pub(crate) Option<(Generator, Generator, Label)>);

#[derive(PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct RewriteN(HConsed<RewriteInternal>);

impl Serialize for RewriteN {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct("RewriteN", self.0.get())
    }
}

#[derive(PartialEq, Eq, Clone, Hash, PartialOrd, Ord, Serialize)]
pub enum Rewrite {
    Rewrite0(Rewrite0),
    RewriteN(RewriteN),
}

type Coordinate<T> = Vec<T>;
pub type Label = (Generator, Coordinate<SliceIndex>);

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize)]
pub struct ConeInternal {
    pub(crate) source: Vec<Cospan>,
    pub(crate) target: Cospan,
    pub(crate) regular_slices: Vec<Rewrite>,
    pub(crate) singular_slices: Vec<Rewrite>,
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct Cone {
    pub(crate) index: usize,
    pub(crate) internal: HConsed<ConeInternal>,
}

impl Serialize for Cone {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Cone", 2)?;
        state.serialize_field("index", &self.index)?;
        state.serialize_field("internal", &self.internal.get())?;
        state.end()
    }
}

impl Cospan {
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

        first_max_generator(generators.iter().copied().flatten())
    }
}

impl fmt::Debug for Rewrite {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RewriteN(r) => r.fmt(f),
            Self::Rewrite0(r) => r.fmt(f),
        }
    }
}

impl From<RewriteN> for Rewrite {
    #[inline]
    fn from(r: RewriteN) -> Self {
        Self::RewriteN(r)
    }
}

impl From<Rewrite0> for Rewrite {
    #[inline]
    fn from(r: Rewrite0) -> Self {
        Self::Rewrite0(r)
    }
}

impl TryFrom<Rewrite> for RewriteN {
    type Error = DimensionError;

    #[inline]
    fn try_from(value: Rewrite) -> Result<Self, Self::Error> {
        match value {
            Rewrite::Rewrite0(_) => Err(DimensionError),
            Rewrite::RewriteN(r) => Ok(r),
        }
    }
}

impl<'a> TryFrom<&'a Rewrite> for &'a RewriteN {
    type Error = DimensionError;

    #[inline]
    fn try_from(value: &'a Rewrite) -> Result<Self, Self::Error> {
        match value {
            Rewrite::Rewrite0(_) => Err(DimensionError),
            Rewrite::RewriteN(r) => Ok(r),
        }
    }
}

impl TryFrom<Rewrite> for Rewrite0 {
    type Error = DimensionError;

    fn try_from(value: Rewrite) -> Result<Self, Self::Error> {
        match value {
            Rewrite::Rewrite0(r) => Ok(r),
            Rewrite::RewriteN(_) => Err(DimensionError),
        }
    }
}

impl Rewrite {
    pub fn cone_over_generator(
        generator: Generator,
        base: Diagram,
        prefix: Coordinate<SliceIndex>,
    ) -> Self {
        use Height::{Regular, Singular};
        use SliceIndex::{Boundary, Interior};

        use crate::Boundary::{Source, Target};

        match base {
            Diagram::Diagram0(base) => Rewrite0::new(base, generator, (generator, prefix)).into(),
            Diagram::DiagramN(base) => {
                let mut regular_slices: Vec<_> = Default::default();
                let mut singular_slices: Vec<_> = Default::default();
                base.slices()
                    .into_iter()
                    .enumerate()
                    .for_each(|(i, slice)| match Height::from(i) {
                        Singular(i) => singular_slices.push(Self::cone_over_generator(
                            generator,
                            slice.clone(),
                            [prefix.as_slice(), &[Interior(Singular(i))]].concat(),
                        )),
                        Regular(0) if base.size() > 0 => { /* omit first regular slice of non-unit cone */ }
                        Regular(i) if base.size() > 0 && base.size() == i => { /* omit last regular slice of non-unit cone */ }
                        Regular(i) => regular_slices.push(Self::cone_over_generator(
                            generator,
                            slice.clone(),
                            [prefix.as_slice(), &[Interior(Regular(i))]].concat(),
                        )),
                    });
                RewriteN::new(
                    base.dimension(),
                    vec![Cone::new(
                        0,
                        base.cospans().to_vec(),
                        Cospan {
                            forward: Self::cone_over_generator(
                                generator,
                                base.source(),
                                [
                                    &[Interior(Singular(0))],
                                    &prefix.as_slice()[..prefix.len() - 1],
                                    &[Boundary(Source)],
                                ]
                                .concat(),
                            ),
                            backward: Self::cone_over_generator(
                                generator,
                                base.target(),
                                [
                                    &[Interior(Singular(0))],
                                    &prefix.as_slice()[..prefix.len() - 1],
                                    &[Boundary(Target)],
                                ]
                                .concat(),
                            ),
                        },
                        regular_slices,
                        singular_slices,
                    )],
                )
                .into()
            }
        }
    }

    #[inline]
    pub fn identity(dimension: usize) -> Self {
        match dimension {
            0 => Rewrite0::identity().into(),
            _ => RewriteN::identity(dimension).into(),
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
        match self {
            Self::Rewrite0(r) => Self::Rewrite0(r.clone()),
            Self::RewriteN(r) => Self::RewriteN(r.pad(embedding)),
        }
    }
}

impl fmt::Debug for Rewrite0 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self(Some((s, t, l))) => f
                .debug_tuple("Rewrite0")
                .field(&s)
                .field(&t)
                .field(&l)
                .finish(),
            Self(None) => f.debug_struct("Rewrite0").finish(),
        }
    }
}

impl Serialize for Rewrite0 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.0 {
            Some((source, target, label)) => {
                let mut r0 = serializer.serialize_struct("Rewrite0", 3)?;
                r0.serialize_field("source", source)?;
                r0.serialize_field("target", target)?;

                struct Coord<'a>(&'a Coordinate<SliceIndex>);

                impl Serialize for Coord<'_> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: serde::Serializer,
                    {
                        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
                        for i in self.0 {
                            match i {
                                SliceIndex::Boundary(Boundary::Source) => {
                                    seq.serialize_element("source")?
                                }
                                SliceIndex::Boundary(Boundary::Target) => {
                                    seq.serialize_element("target")?
                                }
                                SliceIndex::Interior(Height::Regular(i)) => {
                                    seq.serialize_element(&format!("R{}", i))?
                                }
                                SliceIndex::Interior(Height::Singular(i)) => {
                                    seq.serialize_element(&format!("S{}", i))?
                                }
                            }
                        }
                        seq.end()
                    }
                }

                #[derive(Serialize)]
                struct Label<'a> {
                    generator: &'a Generator,
                    coordinate: &'a Coord<'a>,
                }

                r0.serialize_field(
                    "label",
                    &Label {
                        generator: &label.0,
                        coordinate: &Coord(&label.1),
                    },
                )?;
                r0.end()
            }
            None => serializer.serialize_unit_struct("Rewrite0"),
        }
    }
}

impl Rewrite0 {
    pub fn new(source: Generator, target: Generator, label: Label) -> Self {
        assert!(source.dimension <= target.dimension);
        if source == target {
            Self(None)
        } else {
            Self(Some((source, target, label)))
        }
    }

    pub fn identity() -> Self {
        Self(None)
    }

    pub fn is_identity(&self) -> bool {
        self.0.is_none()
    }

    pub fn compose(&self, g: &Self) -> Result<Self, CompositionError> {
        match (&self.0, &g.0) {
            (Some(_), None) => Ok(self.clone()),
            (None, Some(_)) => Ok(g.clone()),
            (None, None) => Ok(Self::identity()),
            // TODO: check this is the correct notion of labelled composition
            (Some((f_s, f_t, f_l)), Some((g_s, g_t, g_l)))
                if f_t == g_s && f_l.0.dimension <= g_l.0.dimension =>
            {
                // to compute a regular slice `r` by composition of a source rewrite `b` and a
                // singular slice `s`:
                //     sⱼ
                //    ^ ^
                // r /  | s
                //  /   |
                // rᵢ → sᵢ
                //    b
                let result = Rewrite0::new(
                    *f_s,
                    *g_t,
                    (
                        g_l.0,
                        [
                            &g_l.1.as_slice()[..g_l.1.len() - f_l.1.len()],
                            &f_l.1.as_slice(),
                        ]
                        .concat(),
                    ),
                );
                Ok(result)
            }
            (f, g) => {
                log::error!("Failed to compose source: {:?}, target: {:?}", f, g);
                Err(CompositionError::Incompatible)
            }
        }
    }

    pub fn source(&self) -> Option<Generator> {
        self.0.as_ref().map(|(source, _, _)| *source)
    }

    pub fn target(&self) -> Option<Generator> {
        self.0.as_ref().map(|(_, target, _)| *target)
    }

    pub fn label(&self) -> Option<&Label> {
        self.0.as_ref().map(|(_, _, label)| label)
    }

    pub(crate) fn max_generator(&self, boundary: Boundary) -> Option<Generator> {
        match boundary {
            Boundary::Source => self.source(),
            Boundary::Target => self.target(),
        }
    }
}

impl fmt::Debug for RewriteN {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq for RewriteInternal {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.dimension == other.dimension && self.cones == other.cones
    }
}

impl Eq for RewriteInternal {}

impl Hash for RewriteInternal {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.dimension.hash(state);
        self.cones.hash(state);
    }
}

impl RewriteN {
    pub(crate) fn new(dimension: usize, cones: Vec<Cone>) -> Self {
        let rewrite = Self::new_unsafe(dimension, cones);
        if cfg!(feature = "safety-checks") {
            rewrite.check(Mode::Shallow).expect("Rewrite is malformed");
        }
        rewrite
    }

    /// Unsafe version of `new` which does not check if the rewrite is well-formed.
    pub(crate) fn new_unsafe(dimension: usize, mut cones: Vec<Cone>) -> Self {
        assert_ne!(dimension, 0, "Can not create RewriteN of dimension zero.");

        // Remove all identity cones. This is not only important to reduce memory consumption, but
        // it allows us the check if the rewrite is an identity by shallowly checking if it has any
        // cones.
        cones.retain(|cone| !cone.is_identity());

        Self(REWRITE_FACTORY.with(|factory| {
            factory.borrow_mut().mk(RewriteInternal {
                dimension,
                cones,
                max_generator_source: OnceCell::new(),
                max_generator_target: OnceCell::new(),
            })
        }))
    }

    #[inline]
    pub fn identity(dimension: usize) -> Self {
        Self::new(dimension, Vec::new())
    }

    pub(crate) fn make_degeneracy(dimension: usize, trivial_heights: &[SingularHeight]) -> Self {
        let cones = trivial_heights
            .iter()
            .enumerate()
            .map(|(i, height)| {
                Cone::new(
                    height - i,
                    vec![],
                    Cospan {
                        forward: Rewrite::identity(dimension - 1),
                        backward: Rewrite::identity(dimension - 1),
                    },
                    vec![Rewrite::identity(dimension - 1)],
                    vec![],
                )
            })
            .collect();

        Self::new(dimension, cones)
    }

    #[inline]
    pub fn from_slices(
        dimension: usize,
        source_cospans: &[Cospan],
        target_cospans: &[Cospan],
        regular_slices: Vec<Vec<Rewrite>>,
        singular_slices: Vec<Vec<Rewrite>>,
    ) -> Self {
        let rewrite = Self::from_slices_unsafe(
            dimension,
            source_cospans,
            target_cospans,
            regular_slices,
            singular_slices,
        );
        if cfg!(feature = "safety-checks") {
            rewrite.check(Mode::Shallow).expect("Rewrite is malformed");
        }
        rewrite
    }

    /// Unsafe version of `from_slices` which does not check if the rewrite is well-formed.
    #[inline]
    pub fn from_slices_unsafe(
        dimension: usize,
        source_cospans: &[Cospan],
        target_cospans: &[Cospan],
        regular_slices: Vec<Vec<Rewrite>>,
        singular_slices: Vec<Vec<Rewrite>>,
    ) -> Self {
        let mut cones = Vec::new();
        let mut index = 0;

        for (target, (rss, sss)) in regular_slices.into_iter().zip(singular_slices).enumerate() {
            let size = sss.len();
            cones.push(Cone::new(
                index,
                source_cospans[index..index + size].to_vec(),
                target_cospans[target].clone(),
                rss,
                sss,
            ));
            index += size;
        }

        Self::new_unsafe(dimension, cones)
    }

    #[inline]
    pub fn from_monotone(
        dimension: usize,
        source_cospans: &[Cospan],
        target_cospans: &[Cospan],
        mono: &[usize],
        regular_slices: &[Rewrite],
        singular_slices: &[Rewrite],
    ) -> Self {
        let rewrite = Self::from_monotone_unsafe(
            dimension,
            source_cospans,
            target_cospans,
            mono,
            regular_slices,
            singular_slices,
        );
        if cfg!(feature = "safety-checks") {
            rewrite.check(Mode::Shallow).expect("Rewrite is malformed");
        }
        rewrite
    }

    /// Unsafe version of `from_monotone` which does not check if the rewrite is well-formed.
    #[inline]
    pub fn from_monotone_unsafe(
        dimension: usize,
        source_cospans: &[Cospan],
        target_cospans: &[Cospan],
        mono: &[usize],
        regular_slices: &[Rewrite],
        singular_slices: &[Rewrite],
    ) -> Self {
        let mut cones_regular_slices: Vec<Vec<Rewrite>> = vec![vec![]; target_cospans.len() + 1];
        let mut cones_singular_slices: Vec<Vec<Rewrite>> = vec![vec![]; target_cospans.len()];
        for (i, &j) in mono.iter().enumerate() {
            cones_regular_slices[j].push(regular_slices[i].clone());
            cones_singular_slices[j].push(singular_slices[i].clone());
        }

        Self::from_slices_unsafe(
            dimension,
            source_cospans,
            target_cospans,
            cones_regular_slices,
            cones_singular_slices,
        )
    }

    pub(crate) fn collect_garbage() {
        REWRITE_FACTORY.with(|factory| factory.borrow_mut().collect_to_fit());
    }

    pub(crate) fn cones(&self) -> &[Cone] {
        &self.0.cones
    }

    pub(crate) fn pad(&self, embedding: &[usize]) -> Self {
        let cones = self
            .cones()
            .iter()
            .map(|cone| cone.pad(embedding))
            .collect();
        Self::new(self.dimension(), cones)
    }

    #[inline]
    pub fn is_identity(&self) -> bool {
        self.0.cones.is_empty()
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

    /// Find a cone targeting a singular height
    pub(crate) fn cone_over_target(&self, height: SingularHeight) -> Option<&Cone> {
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

    /// Take a slice of a rewrite
    pub fn slice(&self, height: SingularHeight) -> Rewrite {
        self.cones()
            .iter()
            .find(|cone| cone.index <= height && height < cone.index + cone.len())
            .map_or(Rewrite::identity(self.dimension() - 1), |cone| {
                cone.internal.singular_slices[height - cone.index].clone()
            })
    }

    pub fn compose(&self, g: &Self) -> Result<Self, CompositionError> {
        if self.dimension() != g.dimension() {
            return Err(CompositionError::Dimension(self.dimension(), g.dimension()));
        }

        let mut offset = 0;
        let mut delayed_offset = 0;

        let mut f_cones: Vec<Cone> = self.cones().iter().rev().cloned().collect();
        let mut g_cones: Vec<Cone> = g.cones().iter().rev().cloned().collect();
        let mut cones: Vec<Cone> = Vec::new();

        loop {
            match (f_cones.pop(), g_cones.pop()) {
                (None, None) => break,
                (Some(f_cone), None) => cones.push(f_cone.clone()),
                (None, Some(g_cone)) => {
                    let mut cone: Cone = g_cone.clone();
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

                        let g_slice = &g_cone.internal.singular_slices[index];
                        let mut singular_slices = vec![];
                        singular_slices
                            .extend(g_cone.internal.singular_slices[..index].iter().cloned());
                        singular_slices.extend(
                            f_cone
                                .internal
                                .singular_slices
                                .iter()
                                .map(|f_slice| f_slice.compose(g_slice))
                                .collect::<Result<Vec<_>, _>>()?,
                        );
                        singular_slices
                            .extend(g_cone.internal.singular_slices[index + 1..].iter().cloned());

                        let g_slice_pre = &g_cone.internal.regular_slices[index];
                        let g_slice_post = &g_cone.internal.regular_slices[index + 1];
                        let mut regular_slices = vec![];
                        regular_slices
                            .extend(g_cone.internal.regular_slices[..index].iter().cloned());
                        regular_slices.extend(
                            f_cone
                                .internal
                                .regular_slices
                                .iter()
                                .map(|f_slice| {
                                    Ok([
                                        f_slice.compose(g_slice_pre)?,
                                        f_slice.compose(g_slice_post)?,
                                    ])
                                })
                                .collect::<Result<Vec<_>, _>>()?
                                .concat(),
                        );

                        delayed_offset -= 1 - f_cone.len() as isize;

                        g_cones.push(Cone::new(
                            g_cone.index,
                            source,
                            g_cone.internal.target.clone(),
                            regular_slices,
                            singular_slices,
                        ));
                    }
                }
            }
        }

        Ok(Self::new(self.dimension(), cones))
    }

    pub fn singular_image(&self, index: SingularHeight) -> SingularHeight {
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

    pub fn singular_preimage(&self, index: SingularHeight) -> Range<SingularHeight> {
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

    pub fn regular_image(&self, index: RegularHeight) -> RegularHeight {
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

    pub fn regular_preimage(&self, index: RegularHeight) -> Range<RegularHeight> {
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
            Boundary::Source => *self.0.max_generator_source.get_or_init(|| {
                first_max_generator(
                    self.cones()
                        .iter()
                        .flat_map(|cone| &cone.internal.source)
                        .filter_map(Cospan::max_generator),
                )
            }),
            Boundary::Target => *self.0.max_generator_target.get_or_init(|| {
                first_max_generator(
                    self.cones()
                        .iter()
                        .filter_map(|cone| cone.internal.target.max_generator()),
                )
            }),
        }
    }
}

impl fmt::Debug for RewriteInternal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RewriteN")
            .field("dimension", &self.dimension)
            .field("cones", &self.cones)
            .finish()
    }
}

impl fmt::Debug for Cone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cone")
            .field("index", &self.index)
            .field("internal", &*self.internal)
            .finish()
    }
}

impl Cone {
    pub(crate) fn new(
        index: usize,
        source: Vec<Cospan>,
        target: Cospan,
        regular_slices: Vec<Rewrite>,
        singular_slices: Vec<Rewrite>,
    ) -> Self {
        Self {
            index,
            internal: CONE_FACTORY.with(|factory| {
                factory.borrow_mut().mk(ConeInternal {
                    source,
                    target,
                    regular_slices,
                    singular_slices,
                })
            }),
        }
    }

    pub(crate) fn collect_garbage() {
        CONE_FACTORY.with(|factory| factory.borrow_mut().collect_to_fit());
    }

    pub(crate) fn is_identity(&self) -> bool {
        // TODO: do we care about regular slices here?
        self.internal.singular_slices.len() == 1
            && self.internal.source.len() == 1
            && self.internal.source[0] == self.internal.target
            && self.internal.singular_slices[0].is_identity()
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
                let regular_slices = self
                    .internal
                    .regular_slices
                    .iter()
                    .map(|r| r.pad(rest))
                    .collect();
                let singular_slices = self
                    .internal
                    .singular_slices
                    .iter()
                    .map(|r| r.pad(rest))
                    .collect();
                Self::new(index, source, target, regular_slices, singular_slices)
            }
            None => self.clone(),
        }
    }
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
