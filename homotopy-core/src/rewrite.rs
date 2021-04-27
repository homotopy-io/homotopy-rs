use crate::{
    common::{DimensionError, Generator, SingularHeight},
    util::CachedCell,
    Boundary,
};
use crate::{diagram::Diagram, util::first_max_generator};

use hashconsing::{HConsed, HConsign, HashConsign};
use std::convert::{From, Into, TryFrom};
use std::fmt;
use std::hash::Hash;
use std::ops::Range;
use std::{cell::RefCell, cmp::Ordering};

use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Cospan {
    pub forward: Rewrite,
    pub backward: Rewrite,
}

impl Cospan {
    pub(crate) fn pad(&self, embedding: &[usize]) -> Self {
        let forward = self.forward.pad(embedding);
        let backward = self.backward.pad(embedding);
        Self { forward, backward }
    }

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

#[derive(PartialEq, Eq, Clone, Hash)]
pub enum Rewrite {
    Rewrite0(Rewrite0),
    RewriteN(RewriteN),
}

impl fmt::Debug for Rewrite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RewriteN(r) => r.fmt(f),
            Self::Rewrite0(r) => r.fmt(f),
        }
    }
}

impl From<RewriteN> for Rewrite {
    fn from(r: RewriteN) -> Self {
        Self::RewriteN(r)
    }
}

impl From<Rewrite0> for Rewrite {
    fn from(r: Rewrite0) -> Self {
        Self::Rewrite0(r)
    }
}

impl TryFrom<Rewrite> for RewriteN {
    type Error = DimensionError;

    fn try_from(value: Rewrite) -> Result<Self, Self::Error> {
        match value {
            Rewrite::Rewrite0(_) => Err(DimensionError),
            Rewrite::RewriteN(r) => Ok(r),
        }
    }
}

impl<'a> TryFrom<&'a Rewrite> for &'a RewriteN {
    type Error = DimensionError;

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
    pub fn identity(dimension: usize) -> Self {
        if dimension == 0 {
            Rewrite0::identity().into()
        } else {
            RewriteN::identity(dimension).into()
        }
    }

    pub fn dimension(&self) -> usize {
        use Rewrite::{Rewrite0, RewriteN};
        match self {
            Rewrite0(_) => 0,
            RewriteN(r) => r.dimension(),
        }
    }

    pub fn is_identity(&self) -> bool {
        use Rewrite::{Rewrite0, RewriteN};
        match self {
            Rewrite0(r) => r.is_identity(),
            RewriteN(r) => r.is_identity(),
        }
    }

    pub(crate) fn pad(&self, embedding: &[usize]) -> Self {
        use Rewrite::{Rewrite0, RewriteN};
        match self {
            Rewrite0(r) => Rewrite0(*r),
            RewriteN(r) => RewriteN(r.pad(embedding)),
        }
    }

    pub fn compose(f: Self, g: Self) -> Result<Self, CompositionError> {
        match (f, g) {
            (Self::Rewrite0(f), Self::Rewrite0(g)) => Ok(Rewrite0::compose(f, g)?.into()),
            (Self::RewriteN(f), Self::RewriteN(g)) => Ok(RewriteN::compose(&f, &g)?.into()),
            (f, g) => Err(CompositionError::Dimension(f.dimension(), g.dimension())),
        }
    }

    pub fn cone_over_generator(generator: Generator, base: Diagram) -> Self {
        match base {
            Diagram::Diagram0(base) => Rewrite0::new(base, generator).into(),
            Diagram::DiagramN(base) => RewriteN::new(
                base.dimension(),
                vec![Cone {
                    index: 0,
                    source: base.cospans().to_vec(),
                    slices: base
                        .singular_slices()
                        .into_iter()
                        .map(|slice| Self::cone_over_generator(generator, slice))
                        .collect(),
                    target: Cospan {
                        forward: Self::cone_over_generator(generator, base.source()),
                        backward: Self::cone_over_generator(generator, base.target()),
                    },
                }],
            )
            .into(),
        }
    }

    pub(crate) fn max_generator(&self, boundary: Boundary) -> Option<Generator> {
        match self {
            Rewrite::Rewrite0(r) => r.max_generator(boundary),
            Rewrite::RewriteN(r) => r.max_generator(boundary),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct Rewrite0(pub(crate) Option<(Generator, Generator)>);

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

    pub fn source(&self) -> Option<Generator> {
        self.0.map(|(source, _)| source)
    }

    pub fn target(&self) -> Option<Generator> {
        self.0.map(|(_, target)| target)
    }

    pub fn compose(f: Self, g: Self) -> Result<Self, CompositionError> {
        match (f.0, g.0) {
            (Some((f_s, f_t)), Some((g_s, g_t))) => {
                if f_t == g_s {
                    Ok(Self(Some((f_s, g_t))))
                } else {
                    Err(CompositionError::Incompatible)
                }
            }
            (Some(_), None) => Ok(f),
            (None, Some(_)) => Ok(g),
            (None, None) => Ok(Self(None)),
        }
    }

    pub(crate) fn max_generator(&self, boundary: Boundary) -> Option<Generator> {
        match boundary {
            Boundary::Source => self.source(),
            Boundary::Target => self.target(),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct RewriteN(HConsed<RewriteInternal>);

// consign! { let REWRITE_FACTORY = consign(37) for RewriteInternal; }

thread_local! {
    static REWRITE_FACTORY: RefCell<HConsign<RewriteInternal>> = RefCell::new(HConsign::with_capacity(37));
}

impl fmt::Debug for RewriteN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

#[derive(Clone)]
struct RewriteInternal {
    dimension: usize,
    cones: Vec<Cone>,
    max_generator_source: CachedCell<Option<Generator>>,
    max_generator_target: CachedCell<Option<Generator>>,
}

impl PartialEq for RewriteInternal {
    fn eq(&self, other: &Self) -> bool {
        self.dimension == other.dimension && self.cones == other.cones
    }
}

impl Eq for RewriteInternal {}

impl Hash for RewriteInternal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.dimension.hash(state);
        self.cones.hash(state);
    }
}

impl RewriteN {
    pub(crate) fn new(dimension: usize, mut cones: Vec<Cone>) -> Self {
        if dimension == 0 {
            panic!("Can not create RewriteN of dimension zero.");
        }

        // Remove all identity cones. This is not only important to reduce memory consumption, but
        // it allows us the check if the rewrite is an identity by shallowly checking if it has any
        // cones.
        cones.retain(|cone| !cone.is_identity());

        Self(REWRITE_FACTORY.with(|factory| {
            factory.borrow_mut().mk(RewriteInternal {
                dimension,
                cones,
                max_generator_source: CachedCell::new(),
                max_generator_target: CachedCell::new(),
            })
        }))
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

    pub fn identity(dimension: usize) -> Self {
        Self::new(dimension, Vec::new())
    }

    pub fn is_identity(&self) -> bool {
        self.0.cones.is_empty()
    }

    pub(crate) fn make_degeneracy(dimension: usize, trivial_heights: &[SingularHeight]) -> Self {
        let cones = trivial_heights
            .iter()
            .enumerate()
            .map(|(i, height)| Cone {
                index: height - i,
                source: vec![],
                target: Cospan {
                    forward: Rewrite::identity(dimension - 1),
                    backward: Rewrite::identity(dimension - 1),
                },
                slices: vec![],
            })
            .collect();

        Self::new(dimension, cones)
    }

    pub fn from_slices(
        dimension: usize,
        source_cospans: &[Cospan],
        target_cospans: &[Cospan],
        slices: Vec<Vec<Rewrite>>,
    ) -> Self {
        let mut cones = Vec::new();
        let mut index = 0;

        for (target, cone_slices) in slices.into_iter().enumerate() {
            let size = cone_slices.len();
            cones.push(Cone {
                source: source_cospans[index..index + size].to_vec(),
                target: target_cospans[target].clone(),
                slices: cone_slices,
                index,
            });
            index += size;
        }

        Self::new(dimension, cones)
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

    pub(crate) fn cone_over_target(&self, height: usize) -> Option<&Cone> {
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

    pub fn slice(&self, height: usize) -> Rewrite {
        self.cones()
            .iter()
            .find(|cone| cone.index <= height && height < cone.index + cone.len())
            .map_or(Rewrite::identity(self.dimension() - 1), |cone| {
                cone.slices[height - cone.index].clone()
            })
    }

    pub fn compose(f: &Self, g: &Self) -> Result<Self, CompositionError> {
        if f.dimension() != g.dimension() {
            return Err(CompositionError::Dimension(f.dimension(), g.dimension()));
        }

        let mut offset = 0;
        let mut delayed_offset = 0;

        let mut f_cones: Vec<Cone> = f.cones().iter().rev().cloned().collect();
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

                        if f_cone.target != g_cone.source[index] {
                            return Err(CompositionError::Incompatible);
                        }

                        let mut source = vec![];
                        source.extend(g_cone.source[..index].iter().cloned());
                        source.extend(f_cone.source.iter().cloned());
                        source.extend(g_cone.source[index + 1..].iter().cloned());

                        let g_slice = &g_cone.slices[index];
                        let mut slices = vec![];
                        slices.extend(g_cone.slices[..index].iter().cloned());
                        slices.extend(
                            f_cone
                                .slices
                                .iter()
                                .map(|f_slice| Rewrite::compose(f_slice.clone(), g_slice.clone()))
                                .collect::<Result<Vec<_>, _>>()?,
                        );
                        slices.extend(g_cone.slices[index + 1..].iter().cloned());

                        delayed_offset -= 1 - f_cone.len() as isize;

                        g_cones.push(Cone {
                            index: g_cone.index,
                            source,
                            target: g_cone.target.clone(),
                            slices,
                        });
                    }
                }
            }
        }

        Ok(Self::new(f.dimension(), cones))
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
                        .flat_map(|cone| &cone.source)
                        .flat_map(Cospan::max_generator),
                    None,
                )
            }),
            Boundary::Target => self.0.max_generator_target.compute(|| {
                first_max_generator(
                    self.cones()
                        .iter()
                        .flat_map(|cone| cone.target.max_generator()),
                    None,
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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub(crate) struct Cone {
    pub(crate) index: usize,
    pub(crate) source: Vec<Cospan>,
    pub(crate) target: Cospan,
    pub(crate) slices: Vec<Rewrite>,
}

impl Cone {
    pub(crate) fn is_identity(&self) -> bool {
        self.slices.len() == 1
            && self.source.len() == 1
            && self.source[0] == self.target
            && self.slices[0].is_identity()
    }

    pub(crate) fn len(&self) -> usize {
        self.source.len()
    }

    pub(crate) fn pad(&self, embedding: &[usize]) -> Self {
        match embedding.split_first() {
            Some((offset, rest)) => {
                let index = self.index + offset;
                let source = self.source.iter().map(|c| c.pad(rest)).collect();
                let target = self.target.pad(rest);
                let slices = self.slices.iter().map(|r| r.pad(rest)).collect();
                Self {
                    index,
                    source,
                    target,
                    slices,
                }
            }
            None => self.clone(),
        }
    }
}

#[derive(Debug, Error)]
pub enum CompositionError {
    #[error("can't compose rewrites of dimensions {0} and {1}")]
    Dimension(usize, usize),

    #[error("failed to compose incompatible rewrites")]
    Incompatible,
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
