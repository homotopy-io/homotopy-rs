use std::{
    cell::RefCell,
    convert::{From, Into, TryFrom},
    fmt,
    hash::Hash,
};

use hashconsing::{HConsed, HConsign, HashConsign};
use homotopy_common::hash::FastHashSet;
use once_cell::unsync::OnceCell;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    attach::attach,
    common::{
        Boundary, BoundaryPath, DimensionError, Direction, Generator, Height, Mode, RegularHeight,
        SliceIndex,
    },
    rewrite::{Cospan, Rewrite, RewriteN},
    signature::{GeneratorInfo, Signature},
    Orientation,
};

thread_local! {
    static DIAGRAM_FACTORY: RefCell<HConsign<DiagramInternal>> =
        RefCell::new(HConsign::with_capacity(37));
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Diagram {
    Diagram0(Diagram0),
    DiagramN(DiagramN),
}

impl Diagram {
    pub fn size(&self) -> Option<usize> {
        match self {
            Self::Diagram0(_) => None,
            Self::DiagramN(d) => Some(d.size()),
        }
    }

    pub fn max_generator(&self) -> Diagram0 {
        match self {
            Self::Diagram0(d) => *d,
            Self::DiagramN(d) => d.max_generator(),
        }
    }

    /// Returns all the generators mentioned by this diagram.
    pub fn generators(&self) -> FastHashSet<Generator> {
        use Diagram::{Diagram0, DiagramN};
        fn add_generators(
            diagram: &Diagram,
            generators: &mut FastHashSet<Generator>,
            visited: &mut FastHashSet<Diagram>,
        ) {
            match diagram {
                Diagram0(d) => {
                    generators.insert(d.generator);
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
        let mut gs: FastHashSet<Generator> = Default::default();
        let mut visited: FastHashSet<Self> = Default::default();
        add_generators(self, &mut gs, &mut visited);
        gs
    }

    pub fn dimension(&self) -> usize {
        match self {
            Self::Diagram0(_) => 0,
            Self::DiagramN(d) => d.dimension(),
        }
    }

    #[must_use]
    pub fn identity(self) -> DiagramN {
        DiagramN::new_unsafe(self, vec![])
    }

    #[must_use]
    pub fn weak_identity(self) -> DiagramN {
        let dimension = self.dimension();
        DiagramN::new_unsafe(
            self,
            vec![Cospan {
                forward: Rewrite::identity(dimension),
                backward: Rewrite::identity(dimension),
            }],
        )
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
                Rewrite::Rewrite0(r) => match &r.0 {
                    None => Ok(self),
                    Some((source, target, _label)) => {
                        if g == *source {
                            Ok(Diagram0(*target))
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
                Rewrite::Rewrite0(r) => match &r.0 {
                    None => Ok(self),
                    Some((source, target, _label)) => {
                        if g == *target {
                            Ok(Diagram0(*source))
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

    /// Removes the framing information belonging to the generators with given id.
    #[must_use]
    pub fn remove_framing(&self, generator: Generator) -> Self {
        match self {
            Self::Diagram0(g) => Self::Diagram0(*g),
            Self::DiagramN(d) => Self::DiagramN(DiagramN::new_unsafe(
                d.source().remove_framing(generator),
                d.cospans()
                    .iter()
                    .map(|cs| cs.map(|r| r.remove_framing(generator)))
                    .collect(),
            )),
        }
    }

    pub fn is_invertible<S>(&self, signature: &S) -> bool
    where
        S: Signature,
    {
        self.generators()
            .iter()
            .filter(|g| g.dimension >= self.dimension())
            .all(|g| signature.generator_info(*g).unwrap().is_invertible())
    }
}

pub(crate) fn globularity(s: &Diagram, t: &Diagram) -> bool {
    use Diagram::{Diagram0, DiagramN};
    match (s, t) {
        (Diagram0(_), Diagram0(_)) => true,
        (Diagram0(_), DiagramN(_)) | (DiagramN(_), Diagram0(_)) => false,
        (DiagramN(s), DiagramN(t)) => s.source() == t.source() && s.target() == t.target(),
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Diagram0 {
    pub generator: Generator,
    pub orientation: Orientation,
}

impl Diagram0 {
    pub fn new(generator: Generator, orientation: Orientation) -> Self {
        Self {
            generator,
            orientation,
        }
    }

    pub fn identity(&self) -> DiagramN {
        Diagram::from(*self).identity()
    }

    #[must_use]
    pub fn orientation_transform(self, k: Orientation) -> Self {
        Self::new(self.generator, self.orientation * k)
    }
}

impl From<Generator> for Diagram0 {
    fn from(generator: Generator) -> Self {
        Self::new(generator, Orientation::Positive)
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct DiagramN(HConsed<DiagramInternal>);

impl Serialize for DiagramN {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct("DiagramN", self.0.get())
    }
}

impl<'de> Deserialize<'de> for DiagramN {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer)
            .map(|d| DiagramN(DIAGRAM_FACTORY.with(|factory| factory.borrow_mut().mk(d))))
    }
}

impl DiagramN {
    pub fn from_generator(
        generator: Generator,
        source: impl Into<Diagram>,
        target: impl Into<Diagram>,
    ) -> Result<Self, NewDiagramError> {
        use crate::Boundary::{Source, Target};

        let source: Diagram = source.into();
        let target: Diagram = target.into();

        if source.dimension() != target.dimension() || generator.dimension != source.dimension() + 1
        {
            return Err(NewDiagramError::Dimension);
        }

        if !globularity(&source, &target) {
            return Err(NewDiagramError::NonGlobular);
        }

        let cospan = Cospan {
            forward: Rewrite::cone_over_generator(
                generator,
                source.clone(),
                BoundaryPath(Source, 0),
                0,
                &[],
                None,
            ),
            backward: Rewrite::cone_over_generator(
                generator,
                target,
                BoundaryPath(Target, 0),
                0,
                &[],
                None,
            ),
        };

        Ok(Self::new_unsafe(source, vec![cospan]))
    }

    pub fn new(source: Diagram, cospans: Vec<Cospan>) -> Self {
        let diagram = Self::new_unsafe(source, cospans);
        if cfg!(feature = "safety-checks") {
            diagram.check(Mode::Shallow).expect("Diagram is malformed");
        }
        diagram
    }

    /// Unsafe version of `new` which does not check if the diagram is well-formed.
    #[inline]
    pub(crate) fn new_unsafe(source: Diagram, cospans: Vec<Cospan>) -> Self {
        Self(DIAGRAM_FACTORY.with(|factory| {
            factory.borrow_mut().mk(DiagramInternal {
                source,
                cospans,
                max_generator: OnceCell::new(),
            })
        }))
    }

    pub(crate) fn collect_garbage() {
        DIAGRAM_FACTORY.with(|factory| factory.borrow_mut().collect_to_fit());
    }

    /// The dimension of the diagram, which is at least one.
    pub fn dimension(&self) -> usize {
        self.0.source.dimension() + 1
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

    pub(crate) fn regular_slices(&self) -> impl Iterator<Item = Diagram> {
        self.slices().step_by(2)
    }

    pub(crate) fn singular_slices(&self) -> impl Iterator<Item = Diagram> {
        self.slices().skip(1).step_by(2)
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
            SliceIndex::Interior(height) => self.slices().nth(height.into()),
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
            if &cospans[start..stop] != cone.source() {
                return Err(RewritingError::Incompatible);
            }
            cospans.splice(start..stop, std::iter::once(cone.target().clone()));
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
            if &cospans[start] != cone.target() {
                return Err(RewritingError::Incompatible);
            }
            cospans.splice(start..stop, cone.source().iter().cloned());
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
        self.regular_slices()
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
    pub fn max_generator(&self) -> Diagram0 {
        *self.0.max_generator.get_or_init(|| {
            std::iter::once(self.source().max_generator())
                .chain(self.cospans().iter().filter_map(Cospan::max_generator))
                .rev()
                .max_by_key(|d| d.generator.dimension)
                .unwrap()
        })
    }

    #[must_use]
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
            .ok_or(DimensionError)?;

        attach(self, BoundaryPath(boundary, depth), |slice| {
            if slice.embeds(&diagram.slice(boundary.flip()).unwrap(), embedding) {
                Ok(diagram.cospans().iter().map(|c| c.pad(embedding)).collect())
            } else {
                Err(AttachmentError::IncompatibleAttachment)
            }
        })
    }

    #[must_use]
    pub fn inverse(&self) -> Self {
        Self::new(
            self.target(),
            self.cospans().iter().map(Cospan::inverse).rev().collect(),
        )
    }

    #[must_use]
    pub fn behead(&self, max_height: RegularHeight) -> Self {
        Self::new(self.source(), self.cospans()[..max_height].to_vec())
    }

    #[must_use]
    pub fn befoot(&self, min_height: RegularHeight) -> Self {
        Self::new(
            self.slice(Height::Regular(min_height)).unwrap(),
            self.cospans()[min_height..].to_vec(),
        )
    }

    pub fn boundary(&self, boundary_path: BoundaryPath) -> Option<Diagram> {
        let mut diagram = self.clone();

        for _ in 0..boundary_path.depth() {
            diagram = diagram.source().try_into().ok()?;
        }

        diagram.slice(boundary_path.boundary())
    }
}

#[derive(Clone, Eq, Serialize, Deserialize)]
struct DiagramInternal {
    source: Diagram,
    cospans: Vec<Cospan>,
    #[serde(skip)]
    max_generator: OnceCell<Diagram0>,
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

impl fmt::Debug for Diagram0 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Diagram0")
            .field(&self.generator)
            .field(&self.orientation)
            .finish()
    }
}

impl fmt::Debug for DiagramN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiagramN")
            .field("source", &self.0.source)
            .field("cospans", &self.0.cospans)
            .finish()
    }
}

impl fmt::Debug for Diagram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Diagram0(d) => d.fmt(f),
            Self::DiagramN(d) => d.fmt(f),
        }
    }
}

impl From<Diagram0> for Diagram {
    fn from(diagram: Diagram0) -> Self {
        Self::Diagram0(diagram)
    }
}

impl From<DiagramN> for Diagram {
    fn from(diagram: DiagramN) -> Self {
        Self::DiagramN(diagram)
    }
}

impl TryFrom<Diagram> for Diagram0 {
    type Error = DimensionError;

    fn try_from(diagram: Diagram) -> Result<Self, Self::Error> {
        match diagram {
            Diagram::Diagram0(diagram) => Ok(diagram),
            Diagram::DiagramN(_) => Err(DimensionError),
        }
    }
}

impl<'a> TryFrom<&'a Diagram> for Diagram0 {
    type Error = DimensionError;

    fn try_from(diagram: &'a Diagram) -> Result<Self, Self::Error> {
        match diagram {
            Diagram::Diagram0(diagram) => Ok(*diagram),
            Diagram::DiagramN(_) => Err(DimensionError),
        }
    }
}

impl TryFrom<Diagram> for DiagramN {
    type Error = DimensionError;

    fn try_from(diagram: Diagram) -> Result<Self, Self::Error> {
        match diagram {
            Diagram::Diagram0(_) => Err(DimensionError),
            Diagram::DiagramN(diagram) => Ok(diagram),
        }
    }
}

impl<'a> TryFrom<&'a Diagram> for &'a DiagramN {
    type Error = DimensionError;

    fn try_from(diagram: &'a Diagram) -> Result<Self, Self::Error> {
        match diagram {
            Diagram::Diagram0(_) => Err(DimensionError),
            Diagram::DiagramN(diagram) => Ok(diagram),
        }
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

impl ExactSizeIterator for Slices {
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

impl std::iter::FusedIterator for Slices {}

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
    #[error("cannot attach diagram of a higher dimension")]
    Dimension(#[from] DimensionError),

    #[error("failed to attach incompatible diagrams")]
    IncompatibleAttachment,
}

#[derive(Clone, Debug, Error)]
pub enum RewritingError {
    #[error("can't rewrite diagram of dimension {0} along a rewrite of dimension {1}")]
    Dimension(usize, usize),

    #[error("failed to rewrite along incompatible rewrite")]
    Incompatible,
}

#[cfg(test)]
mod test {
    use std::{convert::TryInto, error::Error};

    use super::*;
    use crate::signature::SignatureBuilder;

    fn assert_point_ids<D>(diagram: &D, points: &[(&[usize], usize)])
    where
        D: Into<Diagram> + Clone,
    {
        for (point, id) in points {
            let mut slice = diagram.clone().into();

            for p in *point {
                slice = DiagramN::try_from(slice)
                    .unwrap()
                    .slice(Height::from(*p))
                    .unwrap();
            }

            let d: Diagram0 = slice.try_into().unwrap();
            assert_eq!(d.generator.id, *id);
        }
    }

    #[test]
    fn associativity_points() -> Result<(), Box<dyn Error>> {
        use Boundary::{Source, Target};

        let mut signature = SignatureBuilder::default();
        let x = signature.add_zero();
        let f = signature.add(x, x)?;
        let ff = f.attach(&f, Target, &[])?;
        let m = signature.add(ff, f)?;
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
        let mut signature = SignatureBuilder::default();
        let x = signature.add_zero();
        let f = signature.add(x.identity(), x.identity()).unwrap();

        assert_eq!(f.source(), x.identity().into());
        assert_eq!(f.target(), x.identity().into());

        let cospan = &f.cospans()[0];
        let forward: &RewriteN = (&cospan.forward).try_into().unwrap();

        assert_eq!(forward.singular_image(0), 1);
        assert_eq!(forward.regular_preimage(0), 0..2);
    }
}
