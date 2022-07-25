use std::{
    cell::RefCell,
    cmp::Ordering,
    convert::{From, Into},
    fmt,
    hash::Hash,
    ops::{Add, Range},
};

use hashconsing::{HConsed, HConsign, HashConsign};
use once_cell::unsync::OnceCell;
// used for debugging only
use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Serialize,
};
use thiserror::Error;

use crate::{
    common::{DimensionError, Generator, MaxByDimension, Mode, RegularHeight, SingularHeight},
    diagram::Diagram,
    Boundary, Height, SliceIndex,
};

thread_local! {
    static REWRITE_FACTORY: RefCell<HConsign<RewriteInternal>> =
        RefCell::new(HConsign::with_capacity(37));

    static CONE_FACTORY: RefCell<HConsign<ConeInternal>> =
        RefCell::new(HConsign::with_capacity(37));

    static LABEL_FACTORY: RefCell<HConsign<LabelNode>> =
        RefCell::new(HConsign::with_capacity(37));
}

#[derive(Clone, Debug, Error)]
pub enum CompositionError {
    #[error("can't compose rewrites of dimensions {0} and {1}")]
    Dimension(usize, usize),

    #[error("failed to compose incompatible rewrites")]
    Incompatible,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Cospan {
    pub forward: Rewrite,
    pub backward: Rewrite,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RewriteInternal {
    dimension: usize,
    cones: Vec<Cone>,
    #[serde(skip)]
    max_generator_source: OnceCell<Option<Generator>>,
    #[serde(skip)]
    max_generator_target: OnceCell<Option<Generator>>,
}

#[derive(PartialEq, Eq, Clone, Hash, PartialOrd, Ord, Deserialize)]
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

impl<'de> Deserialize<'de> for RewriteN {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer)
            .map(|r| RewriteN(REWRITE_FACTORY.with(|factory| factory.borrow_mut().mk(r))))
    }
}

#[derive(PartialEq, Eq, Clone, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rewrite {
    Rewrite0(Rewrite0),
    RewriteN(RewriteN),
}

type Coordinate<T> = Vec<T>;
pub(crate) type LabelNode = (usize, Coordinate<SliceIndex>);

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct Label(pub(crate) Vec<HConsed<LabelNode>>);

impl Add for Label {
    type Output = Self;

    fn add(mut self, mut rhs: Self) -> Self::Output {
        Self({
            self.0.append(&mut rhs.0);
            self.0
        })
    }
}

impl Label {
    pub fn new(nodes: Vec<LabelNode>) -> Self {
        Self(
            nodes
                .into_iter()
                .map(|node| LABEL_FACTORY.with(|factory| factory.borrow_mut().mk(node)))
                .collect(),
        )
    }
}

impl From<LabelNode> for Label {
    fn from(node: LabelNode) -> Self {
        Self::new(vec![node])
    }
}

impl Serialize for Label {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let nodes: Vec<LabelNode> = self.0.iter().map(|node| node.get().clone()).collect();
        nodes.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Label {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let nodes: Vec<LabelNode> = Deserialize::deserialize(deserializer)?;
        Ok(Label::new(nodes))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub(crate) enum ConeInternal {
    Cone0 {
        target: Cospan,
        regular_slice: Rewrite,
    },
    ConeN {
        source: Vec<Cospan>,
        target: Cospan,
        regular_slices: Vec<Rewrite>,
        singular_slices: Vec<Rewrite>,
    },
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

impl<'de> Deserialize<'de> for Cone {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ConeUnshared {
            index: usize,
            internal: ConeInternal,
        }
        Deserialize::deserialize(deserializer).map(|c: ConeUnshared| Cone {
            index: c.index,
            internal: CONE_FACTORY.with(|factory| factory.borrow_mut().mk(c.internal)),
        })
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

        generators.iter().copied().flatten().max_by_dimension()
    }

    pub fn map<F>(&self, f: F) -> Self
    where
        F: Fn(&Rewrite) -> Rewrite,
    {
        Self {
            forward: f(&self.forward),
            backward: f(&self.backward),
        }
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

impl<'a> From<&'a RewriteN> for &'a Rewrite {
    fn from(r: &'a RewriteN) -> Self {
        r.into()
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

    #[inline]
    fn try_from(value: Rewrite) -> Result<Self, Self::Error> {
        match value {
            Rewrite::Rewrite0(r) => Ok(r),
            Rewrite::RewriteN(_) => Err(DimensionError),
        }
    }
}

impl<'a> TryFrom<&'a Rewrite> for &'a Rewrite0 {
    type Error = DimensionError;

    fn try_from(value: &'a Rewrite) -> Result<Self, Self::Error> {
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
            Diagram::Diagram0(base) => {
                Rewrite0::new(base, generator, Label::new(vec![(generator.id, prefix)])).into()
            }
            Diagram::DiagramN(base) => {
                let mut regular_slices: Vec<_> = Default::default();
                let mut singular_slices: Vec<_> = Default::default();
                base.slices()
                    .into_iter()
                    .enumerate()
                    .for_each(|(i, slice)| match Height::from(i) {
                        Singular(i) => singular_slices.push(Self::cone_over_generator(
                            generator,
                            slice,
                            [prefix.as_slice(), &[Interior(Singular(i))]].concat(),
                        )),
                        Regular(i) => regular_slices.push(Self::cone_over_generator(
                            generator,
                            slice,
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

    pub fn invert_targets(&self) -> Self {
        use Rewrite::{Rewrite0, RewriteN};
        match self {
            Rewrite0(r) => Rewrite0(r.invert_targets()),
            RewriteN(r) => RewriteN(r.invert_targets()),
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

    pub fn remove_framing(&self, id: usize) -> Self {
        match self {
            Self::Rewrite0(r) => Self::Rewrite0(r.remove_framing(id)),
            Self::RewriteN(r) => Self::RewriteN(r.remove_framing(id)),
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
                                    seq.serialize_element("source")?;
                                }
                                SliceIndex::Boundary(Boundary::Target) => {
                                    seq.serialize_element("target")?;
                                }
                                SliceIndex::Interior(Height::Regular(i)) => {
                                    seq.serialize_element(&format!("R{}", i))?;
                                }
                                SliceIndex::Interior(Height::Singular(i)) => {
                                    seq.serialize_element(&format!("S{}", i))?;
                                }
                            }
                        }
                        seq.end()
                    }
                }

                #[derive(Serialize)]
                struct Label<'a> {
                    generator: usize,
                    coordinate: Coord<'a>,
                }

                let mut r0 = serializer.serialize_struct("Rewrite0", 3)?;
                r0.serialize_field("source", source)?;
                r0.serialize_field("target", target)?;
                r0.serialize_field(
                    "label",
                    &label
                        .0
                        .iter()
                        .map(|ln| {
                            let (generator, coordinate) = ln.get();
                            Label {
                                generator: *generator,
                                coordinate: Coord(coordinate),
                            }
                        })
                        .collect::<Vec<_>>(),
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

    pub fn invert_targets(&self) -> Self {
        match &self.0 {
            None => Self(None),
            Some((source, target, label)) => Self::new(*source, target.inverse(), label.clone()),
        }
    }

    pub fn compose(&self, g: &Self) -> Result<Self, CompositionError> {
        match (&self.0, &g.0) {
            (Some(_), None) => Ok(self.clone()),
            (None, Some(_)) => Ok(g.clone()),
            (None, None) => Ok(Self::identity()),
            (Some((f_s, f_t, f_l)), Some((g_s, g_t, g_l))) if f_t == g_s => {
                Ok(Self::new(*f_s, *g_t, f_l.clone() + g_l.clone()))
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

    pub fn remove_framing(&self, id: usize) -> Self {
        match &self.0 {
            None => Self(None),
            Some((source, target, label)) => {
                let new_label = if target.id == id {
                    Label::new(vec![])
                } else {
                    label.clone()
                };
                Self::new(*source, *target, new_label)
            }
        }
    }
}

impl fmt::Debug for RewriteN {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
            .get() // important for hashcons formatting
            .fmt(f)
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
        assert_ne!(dimension, 0, "Cannot create RewriteN of dimension zero.");

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
        mono: &Monotone,
        singular_slices: &[Rewrite],
    ) -> Self {
        let rewrite = Self::from_monotone_unsafe(
            dimension,
            source_cospans,
            target_cospans,
            mono,
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
        mono: &Monotone,
        singular_slices: &[Rewrite],
    ) -> Self {
        // try to determine regular slices by pulling back from target cospans
        let mut cones_regular_slices: Vec<Vec<Rewrite>> = vec![vec![]; target_cospans.len()];
        let mut cones_singular_slices: Vec<Vec<Rewrite>> = vec![vec![]; target_cospans.len()];
        for (i, Split { source, target }) in mono.cones().enumerate() {
            for j in source.clone() {
                cones_singular_slices[i].push(singular_slices[j].clone());
            }
            for j in source.start..=source.end {
                cones_regular_slices[i].push(if j % 2 == source.start % 2 {
                    target_cospans[target].forward.clone()
                } else {
                    target_cospans[target].backward.clone()
                });
            }
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

    pub fn invert_targets(&self) -> Self {
        let cones = self
            .cones()
            .iter()
            .map(|cone| {
                Cone::new_untrimmed(
                    cone.index,
                    cone.source().to_vec(),
                    Cospan {
                        forward: cone.target().forward.invert_targets(),
                        backward: cone.target().backward.invert_targets(),
                    },
                    cone.regular_slices()
                        .iter()
                        .map(Rewrite::invert_targets)
                        .collect(),
                    cone.singular_slices()
                        .iter()
                        .map(Rewrite::invert_targets)
                        .collect(),
                )
            })
            .collect();

        Self::new(self.dimension(), cones)
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

    /// Take a singular slice of a rewrite
    pub fn slice(&self, height: SingularHeight) -> Rewrite {
        self.cones()
            .iter()
            .find(|cone| cone.index <= height && height < cone.index + cone.len())
            .map_or(Rewrite::identity(self.dimension() - 1), |cone| {
                cone.singular_slices()[height - cone.index].clone()
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

                        if f_cone.target() != &g_cone.source()[index] {
                            return Err(CompositionError::Incompatible);
                        }

                        let mut source = vec![];
                        source.extend(g_cone.source()[..index].iter().cloned());
                        source.extend(f_cone.source().iter().cloned());
                        source.extend(g_cone.source()[index + 1..].iter().cloned());

                        let g_slice = &g_cone.singular_slices()[index];
                        let mut singular_slices = vec![];
                        singular_slices.extend(g_cone.singular_slices()[..index].iter().cloned());
                        singular_slices.extend(
                            f_cone
                                .singular_slices()
                                .iter()
                                .map(|f_slice| f_slice.compose(g_slice))
                                .collect::<Result<Vec<_>, _>>()?,
                        );
                        singular_slices
                            .extend(g_cone.singular_slices()[index + 1..].iter().cloned());

                        let mut regular_slices = vec![];
                        regular_slices.extend(g_cone.regular_slices()[..index].iter().cloned());
                        regular_slices.extend(
                            f_cone
                                .regular_slices()
                                .iter()
                                .map(|f_slice| f_slice.compose(g_slice))
                                .collect::<Result<Vec<_>, _>>()?,
                        );
                        regular_slices
                            .extend(g_cone.singular_slices()[index + 1..].iter().cloned());

                        delayed_offset -= 1 - f_cone.len() as isize;

                        g_cones.push(Cone::new_untrimmed(
                            g_cone.index,
                            source,
                            g_cone.target().clone(),
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
                self.cones()
                    .iter()
                    .flat_map(Cone::source)
                    .filter_map(Cospan::max_generator)
                    .max_by_dimension()
            }),
            Boundary::Target => *self.0.max_generator_target.get_or_init(|| {
                self.cones()
                    .iter()
                    .filter_map(|cone| cone.target().max_generator())
                    .max_by_dimension()
            }),
        }
    }

    pub fn remove_framing(&self, id: usize) -> Self {
        let cones = self
            .cones()
            .iter()
            .map(|cone| {
                let regular_slices = cone
                    .regular_slices()
                    .into_iter()
                    .map(|slice| slice.remove_framing(id))
                    .collect::<Vec<_>>();
                let singular_slices = cone
                    .singular_slices()
                    .into_iter()
                    .map(|slice| slice.remove_framing(id))
                    .collect::<Vec<_>>();
                let source = cone
                    .source()
                    .iter()
                    .map(|cs| cs.map(|r| r.remove_framing(id)))
                    .collect();
                let target = cone.target().map(|r| r.remove_framing(id));
                Cone::new_untrimmed(cone.index, source, target, regular_slices, singular_slices)
            })
            .collect();

        Self::new_unsafe(self.dimension(), cones)
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
        debug_assert_eq!(source.len(), singular_slices.len());
        debug_assert_eq!(singular_slices.len() + 1, regular_slices.len());

        if regular_slices.len() > 1 {
            // remove flanges
            Self::new_untrimmed(
                index,
                source,
                target,
                regular_slices[1..regular_slices.len() - 1].to_vec(),
                singular_slices,
            )
        } else {
            Self::new_untrimmed(index, source, target, regular_slices, singular_slices)
        }
    }

    #[inline]
    pub(crate) fn new_untrimmed(
        index: usize,
        source: Vec<Cospan>,
        target: Cospan,
        regular_slices: Vec<Rewrite>,
        singular_slices: Vec<Rewrite>,
    ) -> Self {
        if source.is_empty() {
            Self {
                index,
                internal: CONE_FACTORY.with(|factory| {
                    factory.borrow_mut().mk(ConeInternal::Cone0 {
                        target,
                        regular_slice: regular_slices.into_iter().next().unwrap(),
                    })
                }),
            }
        } else {
            Self {
                index,
                internal: CONE_FACTORY.with(|factory| {
                    factory.borrow_mut().mk(ConeInternal::ConeN {
                        source,
                        target,
                        regular_slices,
                        singular_slices,
                    })
                }),
            }
        }
    }

    pub(crate) fn source(&self) -> &[Cospan] {
        match self.internal.get() {
            ConeInternal::Cone0 { .. } => &[],
            ConeInternal::ConeN { source, .. } => source,
        }
    }

    pub(crate) fn target(&self) -> &Cospan {
        match self.internal.get() {
            ConeInternal::Cone0 { target, .. } | ConeInternal::ConeN { target, .. } => target,
        }
    }

    pub(crate) fn regular_slices(&self) -> &[Rewrite] {
        match self.internal.get() {
            ConeInternal::Cone0 { regular_slice, .. } => std::slice::from_ref(regular_slice),
            ConeInternal::ConeN { regular_slices, .. } => regular_slices,
        }
    }

    pub(crate) fn singular_slices(&self) -> &[Rewrite] {
        match self.internal.get() {
            ConeInternal::Cone0 { .. } => &[],
            ConeInternal::ConeN {
                singular_slices, ..
            } => singular_slices,
        }
    }

    pub(crate) fn collect_garbage() {
        CONE_FACTORY.with(|factory| factory.borrow_mut().collect_to_fit());
    }

    #[allow(dead_code)]
    pub(crate) fn is_unit(&self) -> bool {
        match self.internal.get() {
            ConeInternal::Cone0 { .. } => true,
            ConeInternal::ConeN { .. } => false,
        }
    }

    pub(crate) fn is_identity(&self) -> bool {
        match self.internal.get() {
            ConeInternal::Cone0 { .. } => false,
            ConeInternal::ConeN {
                source,
                target,
                singular_slices,
                ..
            } => {
                debug_assert_eq!(singular_slices.len(), source.len());
                singular_slices.len() == 1
                    && &source[0] == target
                    && singular_slices[0].is_identity()
            }
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.source().len()
    }

    pub(crate) fn pad(&self, embedding: &[usize]) -> Self {
        match embedding.split_first() {
            Some((offset, rest)) => {
                let index = self.index + offset;
                let source = self.source().iter().map(|c| c.pad(rest)).collect();
                let target = self.target().pad(rest);
                let regular_slices = self.regular_slices().iter().map(|r| r.pad(rest)).collect();
                let singular_slices = self.singular_slices().iter().map(|r| r.pad(rest)).collect();
                Self::new_untrimmed(index, source, target, regular_slices, singular_slices)
            }
            None => self.clone(),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::{
        examples::{scalar, two_monoid},
        Boundary::*,
        Height::*,
        SliceIndex::*,
    };

    #[test]
    fn trim_flanges() {
        let x = Generator::new(0, 0);
        let f = Generator::new(1, 1);
        let internal = |gen: Generator| -> Cospan {
            Cospan {
                forward: Rewrite0::new(x, gen, (gen.id, vec![Boundary(Source)]).into()).into(),
                backward: Rewrite0::new(x, gen, (gen.id, vec![Boundary(Target)]).into()).into(),
            }
        };
        let up = |gen: Generator, r: usize| -> Rewrite {
            Rewrite0::new(
                x,
                gen,
                (
                    Generator::new(gen.id + 1, 2).id,
                    vec![Boundary(Source), Interior(Regular(r))],
                )
                    .into(),
            )
            .into()
        };

        let unit_cone = Cone::new(0, vec![], internal(f), vec![up(f, 0)], vec![]);
        assert_eq!(unit_cone.is_unit(), true);

        let unit_cone_untrimmed =
            Cone::new_untrimmed(0, vec![], internal(f), vec![up(f, 0)], vec![]);
        assert_eq!(unit_cone, unit_cone_untrimmed);

        let cone = Cone::new(
            0,
            vec![internal(f)],
            internal(f),
            vec![up(f, 0), up(f, 1)],
            vec![Rewrite::identity(1)],
        );
        assert_eq!(cone.is_unit(), false);

        let cone_untrimmed = Cone::new_untrimmed(
            0,
            vec![internal(f)],
            internal(f),
            vec![],
            vec![Rewrite::identity(1)],
        );
        assert_eq!(cone, cone_untrimmed);
    }

    fn correct_number_of_slices(rewrite: &RewriteN) {
        for cone in rewrite.cones() {
            if cone.is_unit() {
                assert_eq!(cone.regular_slices().len(), 1);
                assert_eq!(cone.singular_slices().len(), 0);
            } else {
                assert_eq!(
                    cone.regular_slices().len() + 1,
                    cone.singular_slices().len()
                );
            }
        }
    }

    #[test]
    fn monoid_correct_number_of_slices() {
        let (_sig, monoid) = two_monoid();
        for Cospan { forward, backward } in monoid.cospans() {
            correct_number_of_slices(forward.try_into().unwrap());
            correct_number_of_slices(backward.try_into().unwrap());
        }
    }

    #[test]
    fn scalar_correct_number_of_slices() {
        let (_sig, scalar) = scalar();
        for Cospan { forward, backward } in scalar.cospans() {
            correct_number_of_slices(forward.try_into().unwrap());
            correct_number_of_slices(backward.try_into().unwrap());
        }
    }
    #[test]
    fn zero_rewrite_compose() {
        let x = Generator::new(0, 0);
        let y = Generator::new(1, 0);
        let z = Generator::new(2, 0);

        let f = Generator::new(3, 1);
        let g = Generator::new(4, 1);

        let first = Rewrite0::new(x, y, (f.id, vec![Boundary(Source)]).into());
        let second = Rewrite0::new(y, z, (g.id, vec![Boundary(Source)]).into());

        let actual = first.compose(&second).unwrap();
        let expected = Rewrite0::new(
            x,
            z,
            Label::new(vec![
                (f.id, vec![Boundary(Source)]),
                (g.id, vec![Boundary(Source)]),
            ]),
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn rewrite_compose() {
        let x = Generator::new(0, 0);
        let f = Generator::new(1, 1);
        let g = Generator::new(2, 1);
        let h = Generator::new(3, 1);
        let x_to_g = Generator::new(5, 2);
        let g_to_h = Generator::new(7, 2);

        let internal = |gen: Generator| -> Cospan {
            Cospan {
                forward: Rewrite0::new(x, gen, (gen.id, vec![Boundary(Source)]).into()).into(),
                backward: Rewrite0::new(x, gen, (gen.id, vec![Boundary(Target)]).into()).into(),
            }
        };
        let up = |gen: Generator, r: usize| -> Rewrite {
            Rewrite0::new(
                x,
                gen,
                (gen.id + 3, vec![Boundary(Source), Interior(Regular(r))]).into(),
            )
            .into()
        };

        let first = RewriteN::from_slices(
            1,
            &[],
            &[internal(f), internal(g)],
            vec![vec![up(f, 0)], vec![up(g, 0)]],
            vec![vec![], vec![]],
        );

        let second = RewriteN::from_slices(
            1,
            &[internal(f), internal(g)],
            &[internal(f), internal(h)],
            vec![vec![up(f, 0), up(f, 1)], vec![up(h, 0), up(h, 1)]],
            vec![
                vec![Rewrite0::identity().into()],
                vec![Rewrite0::new(
                    g,
                    h,
                    (g_to_h.id, vec![Boundary(Source), Interior(Singular(0))]).into(),
                )
                .into()],
            ],
        );

        let expected = RewriteN::from_slices(
            1,
            &[],
            &[internal(f), internal(h)],
            vec![
                vec![up(f, 0)],
                vec![Rewrite0::new(
                    x,
                    h,
                    Label::new(vec![
                        (x_to_g.id, vec![Boundary(Source), Interior(Regular(0))]),
                        (g_to_h.id, vec![Boundary(Source), Interior(Singular(0))]),
                    ]),
                )
                .into()],
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
        let x_to_f = Generator::new(4, 2);
        let f_to_g = Generator::new(7, 2);

        let internal = |gen: Generator| -> Cospan {
            Cospan {
                forward: Rewrite0::new(x, gen, (gen.id, vec![Boundary(Source)]).into()).into(),
                backward: Rewrite0::new(x, gen, (gen.id, vec![Boundary(Target)]).into()).into(),
            }
        };
        let up = |gen: Generator, r: usize| -> Rewrite {
            Rewrite0::new(
                x,
                gen,
                (gen.id + 3, vec![Boundary(Source), Interior(Regular(r))]).into(),
            )
            .into()
        };

        let first =
            RewriteN::from_slices(1, &[], &[internal(f)], vec![vec![up(f, 0)]], vec![vec![]]);

        let second = RewriteN::from_slices(
            1,
            &[internal(f)],
            &[internal(g), internal(h)],
            vec![vec![up(g, 0), up(g, 1)], vec![up(h, 0)]],
            vec![
                vec![Rewrite0::new(
                    f,
                    g,
                    (f_to_g.id, vec![Boundary(Source), Interior(Singular(0))]).into(),
                )
                .into()],
                vec![],
            ],
        );

        let expected = RewriteN::from_slices(
            1,
            &[],
            &[internal(g), internal(h)],
            vec![
                vec![Rewrite0::new(
                    x,
                    g,
                    Label::new(vec![
                        (x_to_f.id, vec![Boundary(Source), Interior(Regular(0))]),
                        (f_to_g.id, vec![Boundary(Source), Interior(Singular(0))]),
                    ]),
                )
                .into()],
                vec![up(h, 0)],
            ],
            vec![vec![], vec![]],
        );

        let actual = RewriteN::compose(&first, &second).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn rewrite_compose_3() {
        let x = Generator::new(0, 0);
        let f = Generator::new(1, 1);
        let g = Generator::new(2, 1);
        let internal = |gen: Generator| -> Cospan {
            Cospan {
                forward: Rewrite0::new(x, gen, (gen.id, vec![Boundary(Source)]).into()).into(),
                backward: Rewrite0::new(x, gen, (gen.id, vec![Boundary(Target)]).into()).into(),
            }
        };
        let up = |gen: Generator, r: usize| -> Rewrite {
            Rewrite0::new(
                x,
                gen,
                (
                    Generator::new(gen.id + 2, 2).id,
                    vec![Boundary(Source), Interior(Regular(r))],
                )
                    .into(),
            )
            .into()
        };
        let f_to_g: Rewrite = Rewrite0::new(
            f,
            g,
            (7, vec![Boundary(Source), Interior(Singular(0))]).into(),
        )
        .into();
        let g_to_f: Rewrite = Rewrite0::new(
            g,
            f,
            (4, vec![Boundary(Source), Interior(Singular(0))]).into(),
        )
        .into();

        let first = RewriteN::from_slices(
            1,
            &[internal(g), internal(g), internal(f), internal(f)],
            &[internal(g), internal(g), internal(f)],
            vec![
                vec![up(g, 0), up(g, 1), up(g, 2)],
                vec![up(g, 0), up(g, 1)],
                vec![up(f, 0), up(f, 1)],
            ],
            vec![
                vec![Rewrite::identity(0), Rewrite::identity(0)],
                vec![f_to_g.clone()],
                vec![Rewrite::identity(0)],
            ],
        );
        assert_eq!(
            first,
            RewriteN::new(
                1,
                vec![
                    Cone::new(
                        0,
                        vec![internal(g), internal(g)],
                        internal(g),
                        vec![up(g, 0), up(g, 1), up(g, 2)],
                        vec![Rewrite::identity(0), Rewrite::identity(0)]
                    ),
                    Cone::new(
                        2,
                        vec![internal(f)],
                        internal(g),
                        vec![up(g, 0), up(g, 1)],
                        vec![f_to_g.clone()]
                    ),
                    Cone::new(
                        3,
                        vec![internal(f)],
                        internal(f),
                        vec![up(f, 0), up(f, 1)],
                        vec![Rewrite::identity(0)]
                    )
                ]
            )
        );
        let second = RewriteN::from_slices(
            1,
            &[internal(g), internal(g), internal(f)],
            &[internal(f), internal(f), internal(g)],
            vec![
                vec![up(f, 0), up(f, 1)],
                vec![up(f, 0), up(f, 1)],
                vec![up(g, 0), up(g, 1)],
            ],
            vec![
                vec![g_to_f.clone()],
                vec![g_to_f.clone()],
                vec![f_to_g.clone()],
            ],
        );

        let actual = RewriteN::compose(&first, &second);

        assert!(actual.is_ok());
    }
}
