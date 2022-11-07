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
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use thiserror::Error;

use crate::{
    common::{
        BoundaryPath, DimensionError, Generator, Mode, Orientation, RegularHeight, SingularHeight,
    },
    diagram::{Diagram, Diagram0},
    label::{Label, Neighbourhood},
    signature::Signature,
    Boundary, Height,
};

thread_local! {
    static REWRITE_FACTORY: RefCell<HConsign<RewriteInternal>> =
        RefCell::new(HConsign::with_capacity(37));

    static CONE_FACTORY: RefCell<HConsign<ConeInternal>> =
        RefCell::new(HConsign::with_capacity(37));
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Cospan {
    pub forward: Rewrite,
    pub backward: Rewrite,
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

    pub(crate) fn max_generator(&self) -> Option<Diagram0> {
        [
            self.forward.max_generator(Boundary::Source),
            self.forward.max_generator(Boundary::Target),
            self.backward.max_generator(Boundary::Target),
            self.backward.max_generator(Boundary::Source),
        ]
        .into_iter()
        .flatten()
        .rev()
        .max_by_key(|d| d.generator.dimension)
    }

    #[must_use]
    pub fn map<F>(&self, f: F) -> Self
    where
        F: Fn(&Rewrite) -> Rewrite,
    {
        Self {
            forward: f(&self.forward),
            backward: f(&self.backward),
        }
    }

    #[must_use]
    pub fn inverse(&self) -> Self {
        use Orientation::Negative;
        Self {
            forward: self.backward.orientation_transform(Negative),
            backward: self.forward.orientation_transform(Negative),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Rewrite {
    Rewrite0(Rewrite0),
    RewriteN(RewriteN),
}

impl Rewrite {
    pub fn cone_over_generator(
        generator: Generator,
        base: Diagram,
        boundary_path: BoundaryPath,
        depth: usize,
        prefix: &[Height],
        signature: &impl Signature,
        neighbourhood: &mut Neighbourhood,
    ) -> Self {
        use Height::{Regular, Singular};

        use crate::Boundary::{Source, Target};

        // Collapse the base to identify labels.
        if prefix.is_empty() {
            neighbourhood.insert(boundary_path, &base, signature);
        }

        match base {
            Diagram::Diagram0(base) => Rewrite0::new(
                base,
                generator,
                Some((generator, boundary_path, prefix.to_vec())),
            )
            .into(),
            Diagram::DiagramN(base) => {
                let target_cospan = Cospan {
                    forward: Self::cone_over_generator(
                        generator,
                        base.source(),
                        BoundaryPath(Source, depth + 1),
                        depth + 1,
                        &[],
                        signature,
                        neighbourhood,
                    ),
                    backward: Self::cone_over_generator(
                        generator,
                        base.target(),
                        BoundaryPath(Target, depth + 1),
                        depth + 1,
                        &[],
                        signature,
                        neighbourhood,
                    ),
                };
                let mut regular_slices: Vec<_> = Default::default();
                let mut singular_slices: Vec<_> = Default::default();
                base.slices()
                    .into_iter()
                    .enumerate()
                    .for_each(|(i, slice)| match Height::from(i) {
                        Regular(i) => regular_slices.push(Self::cone_over_generator(
                            generator,
                            slice,
                            boundary_path,
                            depth + 1,
                            &[prefix, &[Regular(i)]].concat(),
                            signature,
                            neighbourhood,
                        )),
                        Singular(i) => singular_slices.push(Self::cone_over_generator(
                            generator,
                            slice,
                            boundary_path,
                            depth + 1,
                            &[prefix, &[Singular(i)]].concat(),
                            signature,
                            neighbourhood,
                        )),
                    });
                RewriteN::new(
                    base.dimension(),
                    vec![Cone::new(
                        0,
                        base.cospans().to_vec(),
                        target_cospan,
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

    #[must_use]
    pub fn orientation_transform(&self, k: Orientation) -> Self {
        self.orientation_transform_above(k, self.dimension())
    }

    #[must_use]
    fn orientation_transform_above(&self, k: Orientation, dim: usize) -> Self {
        use Rewrite::{Rewrite0, RewriteN};
        match self {
            Rewrite0(r) => Rewrite0(r.orientation_transform_above(k, dim)),
            RewriteN(r) => RewriteN(r.orientation_transform_above(k, dim)),
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
    pub fn is_homotopy(&self) -> bool {
        self.max_generator(Boundary::Target)
            .map_or(true, |d| d.generator.dimension <= self.dimension())
    }

    #[inline]
    pub fn compose(&self, g: &Self) -> Result<Self, CompositionError> {
        match (self, g) {
            (Self::Rewrite0(ref f), Self::Rewrite0(ref g)) => Ok(f.compose(g)?.into()),
            (Self::RewriteN(ref f), Self::RewriteN(ref g)) => Ok(f.compose(g)?.into()),
            (f, g) => Err(CompositionError::Dimension(f.dimension(), g.dimension())),
        }
    }

    pub(crate) fn max_generator(&self, boundary: Boundary) -> Option<Diagram0> {
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

    #[must_use]
    pub fn remove_framing(&self, generator: Generator) -> Self {
        match self {
            Self::Rewrite0(r) => Self::Rewrite0(r.remove_framing(generator)),
            Self::RewriteN(r) => Self::RewriteN(r.remove_framing(generator)),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Rewrite0(pub(crate) Option<(Diagram0, Diagram0, Label)>);

impl Rewrite0 {
    pub fn new(source: impl Into<Diagram0>, target: impl Into<Diagram0>, label: Label) -> Self {
        let source: Diagram0 = source.into();
        let target: Diagram0 = target.into();
        assert!(source.generator.dimension <= target.generator.dimension);
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

    #[must_use]
    fn orientation_transform_above(&self, k: Orientation, dim: usize) -> Self {
        match &self.0 {
            None => Self(None),
            Some((source, mut target, label)) => {
                if dim < target.generator.dimension {
                    target = target.orientation_transform(k);
                }
                Self::new(*source, target, label.clone())
            }
        }
    }

    pub fn compose(&self, g: &Self) -> Result<Self, CompositionError> {
        match (&self.0, &g.0) {
            (Some(_), None) => Ok(self.clone()),
            (None, Some(_)) => Ok(g.clone()),
            (None, None) => Ok(Self::identity()),
            (Some((f_s, f_t, f_l)), Some((g_s, g_t, g_l))) if f_t == g_s => {
                assert!(
                    f_l.is_none() && g_l.is_none(),
                    "Composition of labelled rewrites is illegal"
                );
                Ok(Self::new(*f_s, *g_t, None))
            }
            (f, g) => {
                log::error!("Failed to compose source: {:?}, target: {:?}", f, g);
                Err(CompositionError::Incompatible)
            }
        }
    }

    pub fn source(&self) -> Option<Diagram0> {
        self.0.as_ref().map(|(source, _, _)| *source)
    }

    pub fn target(&self) -> Option<Diagram0> {
        self.0.as_ref().map(|(_, target, _)| *target)
    }

    pub fn label(&self) -> Label {
        self.0.as_ref().and_then(|(_, _, label)| label.clone())
    }

    pub(crate) fn max_generator(&self, boundary: Boundary) -> Option<Diagram0> {
        match boundary {
            Boundary::Source => self.source(),
            Boundary::Target => self.target(),
        }
    }

    #[must_use]
    pub fn remove_framing(&self, generator: Generator) -> Self {
        match &self.0 {
            None => Self(None),
            Some((source, target, label)) => {
                let new_label = if target.generator == generator {
                    None
                } else {
                    label.clone()
                };
                Self::new(*source, *target, new_label)
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
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

impl RewriteN {
    pub fn new(dimension: usize, cones: Vec<Cone>) -> Self {
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

    #[must_use]
    fn orientation_transform_above(&self, k: Orientation, dim: usize) -> Self {
        let cones = self
            .cones()
            .iter()
            .map(|c| {
                Cone::new(
                    c.index,
                    c.source().to_vec(),
                    c.target().map(|r| r.orientation_transform_above(k, dim)),
                    c.regular_slices()
                        .iter()
                        .map(|r| r.orientation_transform_above(k, dim))
                        .collect(),
                    c.singular_slices()
                        .iter()
                        .map(|r| r.orientation_transform_above(k, dim))
                        .collect(),
                )
            })
            .collect();

        Self::new(self.dimension(), cones)
    }

    pub fn dimension(&self) -> usize {
        self.0.dimension
    }

    /// For each cone, find its target singular height
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

                        delayed_offset -= 1 - f_cone.len() as isize;

                        g_cones.push(Cone::new_unlabelled(
                            g_cone.index,
                            source,
                            g_cone.target().clone(),
                            singular_slices,
                        ));
                    }
                }
            }
        }

        Ok(Self::new_unsafe(self.dimension(), cones))
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

    pub(crate) fn max_generator(&self, boundary: Boundary) -> Option<Diagram0> {
        match boundary {
            Boundary::Source => *self.0.max_generator_source.get_or_init(|| {
                self.cones()
                    .iter()
                    .flat_map(Cone::source)
                    .filter_map(Cospan::max_generator)
                    .rev()
                    .max_by_key(|d| d.generator.dimension)
            }),
            Boundary::Target => *self.0.max_generator_target.get_or_init(|| {
                self.cones()
                    .iter()
                    .filter_map(|cone| cone.target().max_generator())
                    .rev()
                    .max_by_key(|d| d.generator.dimension)
            }),
        }
    }

    #[must_use]
    pub fn remove_framing(&self, generator: Generator) -> Self {
        let cones = self
            .cones()
            .iter()
            .map(|c| {
                Cone::new(
                    c.index,
                    c.source()
                        .iter()
                        .map(|cs| cs.map(|r| r.remove_framing(generator)))
                        .collect(),
                    c.target().map(|r| r.remove_framing(generator)),
                    c.regular_slices()
                        .iter()
                        .map(|r| r.remove_framing(generator))
                        .collect(),
                    c.singular_slices()
                        .iter()
                        .map(|r| r.remove_framing(generator))
                        .collect(),
                )
            })
            .collect();

        Self::new_unsafe(self.dimension(), cones)
    }
}

#[derive(Clone, Eq, Serialize, Deserialize)]
struct RewriteInternal {
    dimension: usize,
    cones: Vec<Cone>,
    #[serde(skip)]
    max_generator_source: OnceCell<Option<Diagram0>>,
    #[serde(skip)]
    max_generator_target: OnceCell<Option<Diagram0>>,
}

impl PartialEq for RewriteInternal {
    fn eq(&self, other: &Self) -> bool {
        self.dimension == other.dimension && self.cones == other.cones
    }
}

impl Hash for RewriteInternal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.dimension.hash(state);
        self.cones.hash(state);
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

impl fmt::Debug for RewriteN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RewriteN")
            .field("dimension", &self.0.dimension)
            .field("cones", &self.0.cones)
            .finish()
    }
}

impl fmt::Debug for Rewrite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rewrite0(r) => r.fmt(f),
            Self::RewriteN(r) => r.fmt(f),
        }
    }
}

impl From<Rewrite0> for Rewrite {
    fn from(rewrite: Rewrite0) -> Self {
        Self::Rewrite0(rewrite)
    }
}

impl From<RewriteN> for Rewrite {
    fn from(rewrite: RewriteN) -> Self {
        Self::RewriteN(rewrite)
    }
}

impl TryFrom<Rewrite> for Rewrite0 {
    type Error = DimensionError;

    fn try_from(rewrite: Rewrite) -> Result<Self, Self::Error> {
        match rewrite {
            Rewrite::Rewrite0(rewrite) => Ok(rewrite),
            Rewrite::RewriteN(_) => Err(DimensionError),
        }
    }
}

impl<'a> TryFrom<&'a Rewrite> for &'a Rewrite0 {
    type Error = DimensionError;

    fn try_from(rewrite: &'a Rewrite) -> Result<Self, Self::Error> {
        match rewrite {
            Rewrite::Rewrite0(rewrite) => Ok(rewrite),
            Rewrite::RewriteN(_) => Err(DimensionError),
        }
    }
}

impl TryFrom<Rewrite> for RewriteN {
    type Error = DimensionError;

    fn try_from(rewrite: Rewrite) -> Result<Self, Self::Error> {
        match rewrite {
            Rewrite::Rewrite0(_) => Err(DimensionError),
            Rewrite::RewriteN(rewrite) => Ok(rewrite),
        }
    }
}

impl<'a> TryFrom<&'a Rewrite> for &'a RewriteN {
    type Error = DimensionError;

    fn try_from(rewrite: &'a Rewrite) -> Result<Self, Self::Error> {
        match rewrite {
            Rewrite::Rewrite0(_) => Err(DimensionError),
            Rewrite::RewriteN(rewrite) => Ok(rewrite),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct ConeInternal {
    source: Vec<Cospan>,
    target: Cospan,
    regular_slices: Vec<Rewrite>,
    singular_slices: Vec<Rewrite>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
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

impl Cone {
    #[inline]
    pub fn new(
        index: usize,
        source: Vec<Cospan>,
        target: Cospan,
        regular_slices: Vec<Rewrite>,
        singular_slices: Vec<Rewrite>,
    ) -> Self {
        assert_eq!(source.len(), singular_slices.len());
        assert_eq!(regular_slices.len(), singular_slices.len() + 1);
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

    /// Constructs a unit cone with a unique regular slice.
    #[inline]
    pub fn new_unit(index: usize, target: Cospan, regular_slice: Rewrite) -> Self {
        Self::new(index, vec![], target, vec![regular_slice], vec![])
    }

    /// Constructs a cone where the regular slices are computed from the rest of the data.
    /// Note: This should only be used for *unlabelled* rewrites.
    pub(crate) fn new_unlabelled(
        index: usize,
        source: Vec<Cospan>,
        target: Cospan,
        singular_slices: Vec<Rewrite>,
    ) -> Self {
        let regular_slices = std::iter::zip(&source, &singular_slices)
            .map(|(cs, slice)| cs.forward.compose(slice).unwrap())
            .chain(std::iter::once(target.backward.clone()))
            .collect();
        Self::new(index, source, target, regular_slices, singular_slices)
    }

    pub(crate) fn source(&self) -> &[Cospan] {
        &self.internal.source
    }

    pub(crate) fn target(&self) -> &Cospan {
        &self.internal.target
    }

    pub(crate) fn regular_slices(&self) -> &[Rewrite] {
        &self.internal.regular_slices
    }

    pub(crate) fn singular_slices(&self) -> &[Rewrite] {
        &self.internal.singular_slices
    }

    pub(crate) fn slice(&self, source_height: Height) -> &Rewrite {
        match source_height {
            Height::Regular(i) => &self.regular_slices()[i],
            Height::Singular(i) => &self.singular_slices()[i],
        }
    }

    pub(crate) fn collect_garbage() {
        CONE_FACTORY.with(|factory| factory.borrow_mut().collect_to_fit());
    }

    #[allow(dead_code)]
    pub(crate) fn is_unit(&self) -> bool {
        self.source().is_empty()
    }

    pub(crate) fn is_identity(&self) -> bool {
        self.len() == 1
            && self.source()[0] == *self.target()
            && self.regular_slices()[0] == self.target().forward
            && self.regular_slices()[1] == self.target().backward
            && self.singular_slices()[0].is_identity()
    }

    pub(crate) fn len(&self) -> usize {
        self.source().len()
    }

    pub(crate) fn pad(&self, embedding: &[usize]) -> Self {
        match embedding.split_first() {
            Some((offset, rest)) => Self::new(
                self.index + offset,
                self.source().iter().map(|c| c.pad(rest)).collect(),
                self.target().pad(rest),
                self.regular_slices().iter().map(|r| r.pad(rest)).collect(),
                self.singular_slices().iter().map(|r| r.pad(rest)).collect(),
            ),
            None => self.clone(),
        }
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

#[derive(Clone, Debug, Error)]
pub enum CompositionError {
    #[error("can't compose rewrites of dimensions {0} and {1}")]
    Dimension(usize, usize),

    #[error("failed to compose incompatible rewrites")]
    Incompatible,
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::examples::{scalar, two_monoid};

    fn correct_number_of_slices(rewrite: &RewriteN) {
        for cone in rewrite.cones() {
            assert_eq!(
                cone.regular_slices().len(),
                cone.singular_slices().len() + 1,
            );
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

        let first = Rewrite0::new(x, y, None);
        let second = Rewrite0::new(y, z, None);

        let actual = first.compose(&second).unwrap();
        let expected = Rewrite0::new(x, z, None);
        assert_eq!(actual, expected);
    }

    #[test]
    fn rewrite_compose() {
        let x = Generator::new(0, 0);
        let f = Generator::new(1, 1);
        let g = Generator::new(2, 1);
        let h = Generator::new(3, 1);

        let internal = |gen: Generator| -> Cospan {
            Cospan {
                forward: Rewrite0::new(x, gen, None).into(),
                backward: Rewrite0::new(x, gen, None).into(),
            }
        };
        let up = |gen: Generator| -> Rewrite { Rewrite0::new(x, gen, None).into() };

        let first = RewriteN::from_slices(
            1,
            &[],
            &[internal(f), internal(g)],
            vec![vec![up(f)], vec![up(g)]],
            vec![vec![], vec![]],
        );

        let second = RewriteN::from_slices(
            1,
            &[internal(f), internal(g)],
            &[internal(f), internal(h)],
            vec![vec![up(f), up(f)], vec![up(h), up(h)]],
            vec![
                vec![Rewrite0::identity().into()],
                vec![Rewrite0::new(g, h, None).into()],
            ],
        );

        let expected = RewriteN::from_slices(
            1,
            &[],
            &[internal(f), internal(h)],
            vec![vec![up(f)], vec![Rewrite0::new(x, h, None).into()]],
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

        let internal = |gen: Generator| -> Cospan {
            Cospan {
                forward: Rewrite0::new(x, gen, None).into(),
                backward: Rewrite0::new(x, gen, None).into(),
            }
        };
        let up = |gen: Generator| -> Rewrite { Rewrite0::new(x, gen, None).into() };

        let first = RewriteN::from_slices(1, &[], &[internal(f)], vec![vec![up(f)]], vec![vec![]]);

        let second = RewriteN::from_slices(
            1,
            &[internal(f)],
            &[internal(g), internal(h)],
            vec![vec![up(g), up(g)], vec![up(h)]],
            vec![vec![Rewrite0::new(f, g, None).into()], vec![]],
        );

        let expected = RewriteN::from_slices(
            1,
            &[],
            &[internal(g), internal(h)],
            vec![vec![Rewrite0::new(x, g, None).into()], vec![up(h)]],
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
                forward: Rewrite0::new(x, gen, None).into(),
                backward: Rewrite0::new(x, gen, None).into(),
            }
        };
        let up = |gen: Generator| -> Rewrite { Rewrite0::new(x, gen, None).into() };
        let f_to_g: Rewrite = Rewrite0::new(f, g, None).into();
        let g_to_f: Rewrite = Rewrite0::new(g, f, None).into();

        let first = RewriteN::from_slices(
            1,
            &[internal(g), internal(g), internal(f), internal(f)],
            &[internal(g), internal(g), internal(f)],
            vec![
                vec![up(g), up(g), up(g)],
                vec![up(g), up(g)],
                vec![up(f), up(f)],
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
                        vec![up(g); 3],
                        vec![Rewrite::identity(0); 2]
                    ),
                    Cone::new(
                        2,
                        vec![internal(f)],
                        internal(g),
                        vec![up(g); 2],
                        vec![f_to_g.clone()]
                    ),
                    Cone::new(
                        3,
                        vec![internal(f)],
                        internal(f),
                        vec![up(f); 2],
                        vec![Rewrite::identity(0)]
                    )
                ]
            )
        );
        let second = RewriteN::from_slices(
            1,
            &[internal(g), internal(g), internal(f)],
            &[internal(f), internal(f), internal(g)],
            vec![vec![up(f), up(f)], vec![up(f), up(f)], vec![up(g), up(g)]],
            vec![vec![g_to_f.clone()], vec![g_to_f], vec![f_to_g]],
        );

        let actual = RewriteN::compose(&first, &second);

        assert!(actual.is_ok());
    }
}
