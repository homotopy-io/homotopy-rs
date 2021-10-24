use std::{
    collections::HashSet,
    convert::{From, Into, TryFrom},
    fmt,
    hash::Hash,
};

use hashconsing::{consign, HConsed, HashConsign};
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use thiserror::Error;

use crate::{
    attach::{attach, BoundaryPath},
    common::{
        Boundary, DimensionError, Direction, Generator, Height, Mode, RegularHeight, SliceIndex,
    },
    rewrite::{Cospan, MalformedRewrite, Rewrite, RewriteN},
    util::first_max_generator,
};

#[derive(PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub enum Diagram {
    Diagram0(Generator),
    DiagramN(DiagramN),
}

impl Diagram {
    pub fn to_generator(&self) -> Option<Generator> {
        use Diagram::{Diagram0, DiagramN};
        match self {
            Diagram0(g) => Some(*g),
            DiagramN(_) => None,
        }
    }

    pub fn max_generator(&self) -> Generator {
        use Diagram::{Diagram0, DiagramN};
        match self {
            Diagram0(g) => *g,
            DiagramN(d) => d.max_generator(),
        }
    }

    /// Returns all the generators mentioned by this diagram.
    pub fn generators(&self) -> HashSet<Generator> {
        use Diagram::{Diagram0, DiagramN};
        fn add_generators(
            diagram: &Diagram,
            generators: &mut HashSet<Generator>,
            visited: &mut HashSet<Diagram>,
        ) {
            match diagram {
                Diagram0(g) => {
                    generators.insert(*g);
                    visited.insert(diagram.clone());
                }
                DiagramN(d) => {
                    for slice in d.slices() {
                        if !visited.contains(&slice) {
                            add_generators(&slice, generators, visited);
                            visited.insert(slice);
                        }
                    }
                }
            }
        }
        let mut gs: HashSet<Generator> = Default::default();
        let mut visited: HashSet<Self> = Default::default();
        add_generators(self, &mut gs, &mut visited);
        gs
    }

    pub fn dimension(&self) -> usize {
        use Diagram::{Diagram0, DiagramN};
        match self {
            Diagram0(_) => 0,
            DiagramN(d) => d.dimension(),
        }
    }

    pub fn identity(&self) -> DiagramN {
        DiagramN::new_unsafe(self.clone(), vec![])
    }

    pub fn check_well_formed(&self, mode: Mode) -> Result<(), Vec<MalformedDiagram>> {
        match self {
            Self::Diagram0(_) => Ok(()),
            Self::DiagramN(d) => d.check_well_formed(mode),
        }
    }

    pub fn embeds(&self, diagram: &Self, embedding: &[usize]) -> bool {
        use Diagram::{Diagram0, DiagramN};
        match (self, diagram) {
            (Diagram0(g0), Diagram0(g1)) => g0 == g1,
            (Diagram0(_), DiagramN(_)) => false,
            (DiagramN(d), _) => d.embeds(diagram, embedding),
        }
    }

    pub fn embeddings(&self, diagram: &Self) -> Embeddings {
        use Diagram::{Diagram0, DiagramN};
        match (self, diagram) {
            (Diagram0(g0), Diagram0(g1)) if g0 == g1 => {
                Embeddings(Box::new(std::iter::once(vec![])))
            }
            (Diagram0(_), _) => Embeddings(Box::new(std::iter::empty())),
            (DiagramN(d), _) => d.embeddings(diagram),
        }
    }

    pub(crate) fn rewrite_forward(self, rewrite: &Rewrite) -> Result<Self, RewritingError> {
        use Diagram::{Diagram0, DiagramN};
        match self {
            Diagram0(g) => match &rewrite {
                Rewrite::Rewrite0(r) => match r.0 {
                    None => Ok(self),
                    Some((source, target)) => {
                        if g == source {
                            Ok(Diagram0(target))
                        } else {
                            Err(RewritingError::Incompatible)
                        }
                    }
                },
                Rewrite::RewriteN(r) => Err(RewritingError::Dimension(0, r.dimension())),
            },
            DiagramN(d) => match &rewrite {
                Rewrite::Rewrite0(_) => Err(RewritingError::Dimension(d.dimension(), 0)),
                Rewrite::RewriteN(r) => d.rewrite_forward(r).map(DiagramN),
            },
        }
    }

    pub(crate) fn rewrite_backward(self, rewrite: &Rewrite) -> Result<Self, RewritingError> {
        use Diagram::{Diagram0, DiagramN};
        match self {
            Diagram0(g) => match &rewrite {
                Rewrite::Rewrite0(r) => match r.0 {
                    None => Ok(self),
                    Some((source, target)) => {
                        if g == target {
                            Ok(Diagram0(source))
                        } else {
                            Err(RewritingError::Incompatible)
                        }
                    }
                },
                Rewrite::RewriteN(r) => Err(RewritingError::Dimension(0, r.dimension())),
            },
            DiagramN(d) => match &rewrite {
                Rewrite::Rewrite0(_) => Err(RewritingError::Dimension(d.dimension(), 0)),
                Rewrite::RewriteN(r) => d.rewrite_backward(r).map(DiagramN),
            },
        }
    }
}

pub fn globularity(s: &Diagram, t: &Diagram) -> bool {
    match (s.dimension(), t.dimension()) {
        (0, 0) => true,
        (i, j) => {
            i == j && {
                let s: &DiagramN = <&DiagramN>::try_from(s).unwrap();
                let t: &DiagramN = <&DiagramN>::try_from(t).unwrap();
                s.source() == t.source() && s.target() == t.target()
            }
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct DiagramN(HConsed<DiagramInternal>);

consign! {
   let DIAGRAM_FACTORY = consign(37) for DiagramInternal;
}

impl DiagramN {
    pub fn from_generator<S, T>(
        generator: Generator,
        source: S,
        target: T,
    ) -> Result<Self, NewDiagramError>
    where
        S: Into<Diagram>,
        T: Into<Diagram>,
    {
        let source: Diagram = source.into();
        let target: Diagram = target.into();

        if source.dimension() != target.dimension() || generator.dimension != source.dimension() + 1
        {
            return Err(NewDiagramError::Dimension);
        }

        if let (Diagram::DiagramN(source), Diagram::DiagramN(target)) = (&source, &target) {
            if source.source() != target.source() || source.target() != target.target() {
                return Err(NewDiagramError::NonGlobular);
            }
        }

        let cospan = Cospan {
            forward: Rewrite::cone_over_generator(generator, source.clone()),
            backward: Rewrite::cone_over_generator(generator, target),
        };

        Ok(Self::new_unsafe(source, vec![cospan]))
    }

    #[allow(clippy::expect_used, clippy::let_and_return)]
    pub(crate) fn new(source: Diagram, cospans: Vec<Cospan>) -> Self {
        let diagram = Self::new_unsafe(source, cospans);
        #[cfg(feature = "safety-checks")]
        {
            background_only! {
                diagram
                    .check_well_formed(Mode::Shallow)
                    .expect("Diagram is malformed")
            };
        }
        diagram
    }

    /// Unsafe version of `new` which does not check if the diagram is well-formed.
    #[inline]
    pub(crate) fn new_unsafe(source: Diagram, cospans: Vec<Cospan>) -> Self {
        Self(DIAGRAM_FACTORY.mk(DiagramInternal { source, cospans }))
    }

    pub(crate) fn collect_garbage() {
        DIAGRAM_FACTORY.collect_to_fit();
    }

    /// The dimension of the diagram, which is at least one.
    pub fn dimension(&self) -> usize {
        self.0.source.dimension() + 1
    }

    pub fn check_well_formed(&self, mode: Mode) -> Result<(), Vec<MalformedDiagram>> {
        let mut errors: Vec<MalformedDiagram> = Default::default();
        let mut slice = self.source();

        // Check that the source slice is well-formed.
        if mode == Mode::Deep {
            if let Err(e) = slice.check_well_formed(mode) {
                errors.push(MalformedDiagram::Slice(Height::Regular(0), e));
            }
        }

        for (i, cospan) in self.cospans().iter().enumerate() {
            // Check that the forward rewrite is well-formed.
            if mode == Mode::Deep {
                if let Err(e) = cospan.forward.check_well_formed(mode) {
                    errors.push(MalformedDiagram::Rewrite(i, Direction::Forward, e));
                }
            }

            // Check that the forward rewrite is compatible with the regular slice.
            match slice.rewrite_forward(&cospan.forward) {
                Ok(next) if mode == Mode::Deep => {
                    if let Err(e) = next.check_well_formed(mode) {
                        errors.push(MalformedDiagram::Slice(Height::Singular(i), e));
                    }
                    slice = next;
                }
                Ok(next) => slice = next,
                Err(re) => {
                    errors.push(MalformedDiagram::Incompatible(i, Direction::Forward, re));
                    break;
                }
            }

            // Check that the backward rewrite is well-formed.
            if mode == Mode::Deep {
                if let Err(e) = cospan.backward.check_well_formed(mode) {
                    errors.push(MalformedDiagram::Rewrite(i, Direction::Backward, e));
                }
            }

            // Check that the backward rewrite is compatible with the singular slice.
            match slice.rewrite_backward(&cospan.backward) {
                Ok(next) if mode == Mode::Deep => {
                    if let Err(e) = next.check_well_formed(mode) {
                        errors.push(MalformedDiagram::Slice(Height::Regular(i + 1), e));
                    }
                    slice = next;
                }
                Ok(next) => slice = next,
                Err(re) => {
                    errors.push(MalformedDiagram::Incompatible(i, Direction::Backward, re));
                    break;
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// The source boundary of the diagram.
    pub fn source(&self) -> Diagram {
        self.0.source.clone()
    }

    /// The target boundary of the diagram.
    ///
    /// This function rewrites the source slice of the diagram with all of the diagram's cospans.
    pub fn target(&self) -> Diagram {
        let mut slice = self.0.source.clone();

        for cospan in &self.0.cospans {
            slice = slice.rewrite_forward(&cospan.forward).unwrap();
            slice = slice.rewrite_backward(&cospan.backward).unwrap();
        }

        slice
    }

    /// An iterator over all of the diagram's slices.
    pub fn slices(&self) -> Slices {
        Slices::new(self)
    }

    pub(crate) fn singular_slices(&self) -> Vec<Diagram> {
        let mut regular = self.0.source.clone();
        let mut slices = Vec::new();

        for cospan in &self.0.cospans {
            let singular = regular.rewrite_forward(&cospan.forward).unwrap();
            slices.push(singular.clone());
            regular = singular.rewrite_backward(&cospan.backward).unwrap();
        }

        slices
    }

    /// Access a particular slice.
    ///
    /// This function rewrites the source until the slice of the desired height is reached.  When
    /// all of the diagram's slices are needed, use [slices](crate::diagram::DiagramN::slices) to avoid
    /// quadratic complexity.
    pub fn slice<I>(&self, index: I) -> Option<Diagram>
    where
        I: Into<SliceIndex>,
    {
        match index.into() {
            SliceIndex::Boundary(Boundary::Source) => Some(self.source()),
            SliceIndex::Boundary(Boundary::Target) => Some(self.target()),
            SliceIndex::Interior(height) => self.slices().nth(height.to_int()),
        }
    }

    pub(crate) fn rewrite_forward(self, rewrite: &RewriteN) -> Result<Self, RewritingError> {
        if self.dimension() != rewrite.dimension() {
            return Err(RewritingError::Dimension(
                self.dimension(),
                rewrite.dimension(),
            ));
        }

        let mut cospans = self.cospans().to_vec();
        let mut offset: isize = 0;

        for cone in rewrite.cones() {
            let start = (cone.index as isize + offset) as usize;
            let stop = (cone.index as isize + cone.len() as isize + offset) as usize;
            if cospans[start..stop] != cone.internal.source {
                return Err(RewritingError::Incompatible);
            }
            cospans.splice(start..stop, std::iter::once(cone.internal.target.clone()));
            offset -= cone.len() as isize - 1;
        }

        Ok(Self::new_unsafe(self.source(), cospans))
    }

    pub(crate) fn rewrite_backward(self, rewrite: &RewriteN) -> Result<Self, RewritingError> {
        if self.dimension() != rewrite.dimension() {
            return Err(RewritingError::Dimension(
                self.dimension(),
                rewrite.dimension(),
            ));
        }

        let mut cospans = self.cospans().to_vec();

        for cone in rewrite.cones() {
            let start = cone.index;
            let stop = cone.index + 1;
            if cospans[start] != cone.internal.target {
                return Err(RewritingError::Incompatible);
            }
            cospans.splice(start..stop, cone.internal.source.iter().cloned());
        }

        Ok(Self::new_unsafe(self.source(), cospans))
    }

    pub fn cospans(&self) -> &[Cospan] {
        &self.0.cospans
    }

    /// Check if [diagram] embeds into this diagram via the specified [embedding].
    pub fn embeds(&self, diagram: &Diagram, embedding: &[usize]) -> bool {
        use Diagram::{Diagram0, DiagramN};

        let (regular, rest) = match embedding.split_first() {
            Some((regular, rest)) => (*regular, rest),
            None => (0, embedding),
        };

        let height = SliceIndex::Interior(Height::Regular(regular));

        let slice = match self.slice(height) {
            Some(slice) => slice,
            None => {
                return false;
            }
        };

        match diagram {
            Diagram0(_) => slice.embeds(diagram, rest),
            DiagramN(d) => {
                use std::cmp::Ordering::{Equal, Greater, Less};
                match d.dimension().cmp(&self.dimension()) {
                    Greater => false,
                    Less => slice.embeds(diagram, rest),
                    Equal => {
                        slice.embeds(&d.source(), rest)
                            && self.0.cospans.get(regular..d.size() + regular)
                                == Some(
                                    &d.0.cospans.iter().map(|c| c.pad(rest)).collect::<Vec<_>>(),
                                )
                    }
                }
            }
        }
    }

    pub fn embeddings(&self, diagram: &Diagram) -> Embeddings {
        use std::cmp::Ordering;

        match self.dimension().cmp(&diagram.dimension()) {
            Ordering::Less => Embeddings(Box::new(std::iter::empty())),
            Ordering::Equal => {
                let diagram = Self::try_from(diagram.clone()).unwrap();
                let embeddings = self.embeddings_slice(diagram.source());
                let haystack = self.clone();
                Embeddings(Box::new(embeddings.filter(move |embedding| {
                    let (start, rest) = embedding.split_first().unwrap();
                    haystack.cospans().get(*start..diagram.size() + *start)
                        == Some(
                            &diagram
                                .cospans()
                                .iter()
                                .map(|c| c.pad(rest))
                                .collect::<Vec<_>>(),
                        )
                })))
            }
            Ordering::Greater => Embeddings(Box::new(self.embeddings_slice(diagram.clone()))),
        }
    }

    fn embeddings_slice(&self, diagram: Diagram) -> impl Iterator<Item = Vec<usize>> {
        self.clone()
            .slices()
            .step_by(2)
            .enumerate()
            .flat_map(move |(index, slice)| {
                slice.embeddings(&diagram).into_iter().map(move |mut emb| {
                    emb.insert(0, index);
                    emb
                })
            })
    }

    /// The size of the diagram is the number of singular slices or equivalently the number of
    /// cospans in the diagram.
    pub fn size(&self) -> usize {
        self.0.cospans.len()
    }

    /// Determine the first maximum-dimensional generator.
    pub fn max_generator(&self) -> Generator {
        let source = std::iter::once(self.source().max_generator());
        let cospans = self.cospans().iter().filter_map(Cospan::max_generator);
        let generators = source.chain(cospans);
        first_max_generator(generators).unwrap()
    }

    pub fn identity(&self) -> Self {
        Diagram::from(self.clone()).identity()
    }

    // TODO: This needs better documentation

    /// Attach a [diagram] to this diagram at the specified [boundary] and the given [embedding].
    pub fn attach(
        &self,
        diagram: &Self,
        boundary: Boundary,
        embedding: &[usize],
    ) -> Result<Self, AttachmentError> {
        let depth = self
            .dimension()
            .checked_sub(diagram.dimension())
            .ok_or_else(|| AttachmentError::Dimension(diagram.dimension(), self.dimension()))?;

        attach(self, &BoundaryPath(boundary, depth), |slice| {
            if slice.embeds(&diagram.slice(boundary.flip()).unwrap(), embedding) {
                Ok(diagram.cospans().iter().map(|c| c.pad(embedding)).collect())
            } else {
                Err(AttachmentError::Incompatible)
            }
        })
    }
}

impl fmt::Debug for DiagramN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

impl fmt::Debug for Diagram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Diagram0(generator) => f.debug_tuple("Diagram0").field(generator).finish(),
            Self::DiagramN(diagram) => diagram.fmt(f),
        }
    }
}

impl From<DiagramN> for Diagram {
    fn from(diagram: DiagramN) -> Self {
        Self::DiagramN(diagram)
    }
}

impl From<Generator> for Diagram {
    fn from(generator: Generator) -> Self {
        Self::Diagram0(generator)
    }
}

impl TryFrom<Diagram> for DiagramN {
    type Error = DimensionError;

    fn try_from(from: Diagram) -> Result<Self, Self::Error> {
        match from {
            Diagram::DiagramN(from) => Ok(from),
            Diagram::Diagram0(_) => Err(DimensionError),
        }
    }
}

impl<'a> TryFrom<&'a Diagram> for &'a DiagramN {
    type Error = DimensionError;

    fn try_from(from: &'a Diagram) -> Result<Self, Self::Error> {
        match from {
            Diagram::DiagramN(from) => Ok(from),
            Diagram::Diagram0(_) => Err(DimensionError),
        }
    }
}

impl TryFrom<Diagram> for Generator {
    type Error = DimensionError;

    fn try_from(from: Diagram) -> Result<Self, Self::Error> {
        match from {
            Diagram::DiagramN(_) => Err(DimensionError),
            Diagram::Diagram0(g) => Ok(g),
        }
    }
}

#[derive(Eq, Clone)]
struct DiagramInternal {
    source: Diagram,
    cospans: Vec<Cospan>,
}

impl PartialEq for DiagramInternal {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && self.cospans == other.cospans
    }
}

impl Hash for DiagramInternal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.cospans.hash(state);
    }
}

impl fmt::Debug for DiagramInternal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiagramN")
            .field("source", &self.source)
            .field("cospans", &self.cospans)
            .finish()
    }
}

/// Iterator over a diagram's slices. Constructed via [DiagramN::slices].
pub struct Slices {
    current: Option<Diagram>,
    direction: Direction,
    cospans: Vec<Cospan>,
}

impl Slices {
    fn new(diagram: &DiagramN) -> Self {
        Self {
            current: Some(diagram.source()),
            direction: Direction::Forward,
            cospans: diagram.cospans().iter().rev().cloned().collect(),
        }
    }
}

impl Iterator for Slices {
    type Item = Diagram;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cospans.is_empty() {
            return std::mem::replace(&mut self.current, None);
        }

        let current = self.current.as_ref()?;

        let next = match self.direction {
            Direction::Forward => {
                let cospan = self.cospans.last().unwrap();
                self.direction = Direction::Backward;
                current.clone().rewrite_forward(&cospan.forward).unwrap()
            }
            Direction::Backward => {
                let cospan = self.cospans.pop().unwrap();
                self.direction = Direction::Forward;
                current.clone().rewrite_backward(&cospan.backward).unwrap()
            }
        };

        std::mem::replace(&mut self.current, Some(next))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a> ExactSizeIterator for Slices {
    fn len(&self) -> usize {
        if self.current.is_none() {
            0
        } else {
            match self.direction {
                Direction::Forward => self.cospans.len() * 2 + 1,
                Direction::Backward => self.cospans.len() * 2,
            }
        }
    }
}

impl<'a> std::iter::FusedIterator for Slices {}

pub struct Embeddings(Box<dyn Iterator<Item = Vec<RegularHeight>>>);

impl Iterator for Embeddings {
    type Item = Vec<RegularHeight>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl std::iter::FusedIterator for Embeddings {}

#[derive(Debug, Error)]
pub enum NewDiagramError {
    #[error("non-compatible dimensions when creating diagram")]
    Dimension,

    #[error("can't create diagram with non-globular boundaries")]
    NonGlobular,
}

#[derive(Debug, Error)]
pub enum AttachmentError {
    #[error("can't attach diagram of dimension {0} to a diagram of dimension {1}")]
    Dimension(usize, usize),

    #[error("failed to attach incompatible diagrams")]
    Incompatible,
}

#[derive(Debug, Error)]
pub enum RewritingError {
    #[error("can't rewrite diagram of dimension {0} along a rewrite of dimension {1}")]
    Dimension(usize, usize),

    #[error("failed to rewrite along incompatible rewrite")]
    Incompatible,
}

#[derive(Debug, Error)]
pub enum MalformedDiagram {
    #[error("slice {0:?} is malformed: {1:?}")]
    Slice(Height, Vec<MalformedDiagram>),

    #[error("rewrite {0} in direction {1:?} is malformed: {2:?}")]
    Rewrite(usize, Direction, Vec<MalformedRewrite>),

    #[error("rewrite {0} in direction {1:?} is incompatible with its source/target.")]
    Incompatible(usize, Direction, RewritingError),
}

#[cfg(test)]
mod test {
    use std::{convert::TryInto, error::Error};

    use super::*;

    fn assert_point_ids<D>(diagram: &D, points: &[(&[usize], usize)])
    where
        D: Into<Diagram> + Clone,
    {
        for (point, id) in points {
            let mut slice = diagram.clone().into();

            for p in *point {
                slice = DiagramN::try_from(slice)
                    .unwrap()
                    .slice(Height::from_int(*p))
                    .unwrap();
            }

            let generator = slice.to_generator().unwrap();
            assert_eq!(generator.id, *id);
        }
    }

    #[test]
    fn associativity_points() -> Result<(), Box<dyn Error>> {
        use Boundary::{Source, Target};

        let x = Diagram::from(Generator::new(0, 0));
        let f = DiagramN::from_generator(Generator::new(1, 1), x.clone(), x)?;
        let ff = f.attach(&f, Target, &[])?;
        let m = DiagramN::from_generator(Generator::new(2, 2), ff, f)?;
        let left = m.attach(&m, Source, &[0])?;

        assert_point_ids(
            &left,
            &[
                (&[0, 0], 0),
                (&[0, 1], 1),
                (&[0, 2], 0),
                (&[0, 3], 1),
                (&[0, 4], 0),
                (&[0, 5], 1),
                (&[0, 6], 0),
                (&[1, 0], 0),
                (&[1, 1], 2),
                (&[1, 2], 0),
                (&[1, 3], 1),
                (&[1, 4], 0),
                (&[2, 0], 0),
                (&[2, 1], 1),
                (&[2, 2], 0),
                (&[2, 3], 1),
                (&[2, 4], 0),
                (&[3, 0], 0),
                (&[3, 1], 2),
                (&[3, 2], 0),
                (&[4, 0], 0),
                (&[4, 1], 1),
                (&[4, 2], 0),
            ],
        );

        Ok(())
    }

    #[test]
    fn scalar() {
        let x = Diagram::from(Generator::new(0, 0));
        let f = DiagramN::from_generator(Generator::new(1, 2), x.identity(), x.identity()).unwrap();

        assert_eq!(f.source(), x.identity().into());
        assert_eq!(f.target(), x.identity().into());

        let cospan = &f.cospans()[0];
        let forward: &RewriteN = (&cospan.forward).try_into().unwrap();

        assert_eq!(forward.singular_image(0), 1);
        assert_eq!(forward.regular_preimage(0), 0..2);
    }
}
