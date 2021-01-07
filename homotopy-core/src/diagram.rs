use crate::attach::*;
use crate::common::*;
use crate::contraction::*;
use crate::expansion::*;
use crate::rewrite::*;
use std::convert::TryFrom;
use std::convert::*;
use std::fmt;
use std::rc::Rc;
use thiserror::Error;

#[derive(PartialEq, Eq, Clone)]
pub enum Diagram {
    Diagram0(Generator),
    DiagramN(DiagramN),
}

impl Diagram {
    pub fn to_generator(&self) -> Option<Generator> {
        use Diagram::*;
        match self {
            Diagram0(g) => Some(*g),
            DiagramN(_) => None,
        }
    }

    pub fn max_generator(&self) -> Generator {
        use Diagram::*;
        match self {
            Diagram0(g) => *g,
            DiagramN(d) => d.max_generator(),
        }
    }

    pub fn dimension(&self) -> usize {
        use Diagram::*;
        match self {
            Diagram0(_) => 0,
            DiagramN(d) => d.dimension(),
        }
    }

    pub fn identity(&self) -> DiagramN {
        DiagramN::new_unsafe(self.clone(), vec![])
    }

    pub fn embeds(&self, diagram: &Diagram, embedding: &[usize]) -> bool {
        use Diagram::*;
        match (self, diagram) {
            (Diagram0(g0), Diagram0(g1)) => g0 == g1,
            (Diagram0(_), DiagramN(_)) => false,
            (DiagramN(d), _) => d.embeds(diagram, embedding),
        }
    }

    pub fn embeddings(&self, diagram: &Diagram) -> Embeddings {
        use Diagram::*;
        match (self, diagram) {
            (Diagram0(g0), Diagram0(g1)) if g0 == g1 => {
                Embeddings(Box::new(std::iter::once(vec![])))
            }
            (Diagram0(_), _) => Embeddings(Box::new(std::iter::empty())),
            (DiagramN(d), _) => d.embeddings(diagram),
        }
    }

    pub(crate) fn rewrite_forward(self, rewrite: &Rewrite) -> Diagram {
        use Diagram::*;
        use Rewrite::*;
        match self {
            Diagram0(_) => match &rewrite {
                Rewrite0(r) => match r.target() {
                    Some(target) => Diagram0(target),
                    None => self,
                },
                RewriteN(_) => panic!(),
            },
            DiagramN(d) => match &rewrite {
                Rewrite0(_) => panic!(),
                RewriteN(r) => DiagramN(d.rewrite_forward(r)),
            },
        }
    }

    pub(crate) fn rewrite_backward(self, rewrite: &Rewrite) -> Diagram {
        use Diagram::*;
        use Rewrite::*;
        match self {
            Diagram0(_) => match &rewrite {
                Rewrite0(r) => match r.source() {
                    Some(source) => Diagram0(source),
                    None => self,
                },
                RewriteN(_) => panic!(),
            },
            DiagramN(d) => match &rewrite {
                Rewrite0(_) => panic!(),
                RewriteN(r) => DiagramN(d.rewrite_backward(r)),
            },
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct DiagramN(Rc<DiagramInternal>);

impl DiagramN {
    pub fn new<S, T>(generator: Generator, source: S, target: T) -> Result<Self, NewDiagramError>
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

        Ok(DiagramN::new_unsafe(source, vec![cospan]))
    }

    pub(crate) fn new_unsafe(source: Diagram, cospans: Vec<Cospan>) -> Self {
        DiagramN(Rc::new(DiagramInternal { source, cospans }))
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
            slice = slice.rewrite_forward(&cospan.forward);
            slice = slice.rewrite_backward(&cospan.backward);
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
            let singular = regular.rewrite_forward(&cospan.forward);
            slices.push(singular.clone());
            regular = singular.rewrite_backward(&cospan.backward);
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

    pub(crate) fn rewrite_forward(mut self, rewrite: &RewriteN) -> DiagramN {
        let diagram: &mut DiagramInternal = Rc::make_mut(&mut self.0);
        let mut offset: isize = 0;

        for cone in rewrite.cones() {
            let start = (cone.index as isize + offset) as usize;
            let stop = (cone.index as isize + cone.len() as isize + offset) as usize;
            diagram
                .cospans
                .splice(start..stop, vec![cone.target.clone()]);
            offset -= cone.len() as isize - 1;
        }

        DiagramN(self.0)
    }

    pub(crate) fn rewrite_backward(mut self, rewrite: &RewriteN) -> DiagramN {
        let diagram: &mut DiagramInternal = Rc::make_mut(&mut self.0);

        for cone in rewrite.cones() {
            let start = cone.index;
            let stop = cone.index + 1;
            diagram.cospans.splice(start..stop, cone.source.clone());
        }

        DiagramN(self.0)
    }

    pub fn cospans(&self) -> &[Cospan] {
        &self.0.cospans
    }

    /// Check if [diagram] embeds into this diagram via the specified [embedding].
    pub fn embeds(&self, diagram: &Diagram, embedding: &[usize]) -> bool {
        use Diagram::*;

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
                use std::cmp::Ordering::*;
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
                let diagram = DiagramN::try_from(diagram.clone()).unwrap();
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
        // TODO: This can be done more efficiently by looking at the parts
        self.slices()
            .map(|slice| slice.max_generator())
            .max_by_key(|generator| generator.dimension)
            .unwrap()
    }

    pub fn identity(&self) -> DiagramN {
        Diagram::from(self.clone()).identity()
    }

    pub fn contract(
        &self,
        path: &[SliceIndex],
        height: SingularHeight,
        bias: Option<Bias>,
    ) -> Option<DiagramN> {
        let (boundary_path, interior_path) = BoundaryPath::split(path);

        match boundary_path {
            Some(boundary_path) => contract(self, boundary_path, &interior_path, height, bias),
            None => {
                let result = contract(
                    &self.identity(),
                    Boundary::Target.into(),
                    &interior_path,
                    height,
                    bias,
                )?;
                Some(result.target().try_into().unwrap())
            }
        }
    }

    pub fn expand(
        &self,
        path: &[SliceIndex],
        direction: Direction,
    ) -> Result<DiagramN, ExpansionError> {
        let (boundary_path, interior_path) = BoundaryPath::split(path);

        match boundary_path {
            Some(boundary_path) => expand(self, boundary_path, &interior_path, direction),
            None => {
                let result = expand(
                    &self.identity(),
                    Boundary::Target.into(),
                    &interior_path,
                    direction,
                )?;
                Ok(result.target().try_into().unwrap())
            }
        }
    }

    // TODO: This needs better documentation

    /// Attach a [diagram] to this diagram at the specified [boundary] and the given [embedding].
    pub fn attach(
        &self,
        diagram: DiagramN,
        boundary: Boundary,
        embedding: &[usize],
    ) -> Result<DiagramN, AttachmentError> {
        let depth = self
            .dimension()
            .checked_sub(diagram.dimension())
            .ok_or_else(|| AttachmentError::Dimension(diagram.dimension(), self.dimension()))?;

        attach(self.clone(), BoundaryPath(boundary, depth), |slice| {
            if !slice.embeds(&diagram.slice(boundary.flip()).unwrap(), embedding) {
                Err(AttachmentError::Incompatible)
            } else {
                Ok(diagram
                    .cospans()
                    .iter()
                    .map(|c| c.pad(&embedding))
                    .collect())
            }
        })
    }
}

impl fmt::Debug for DiagramN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for Diagram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Diagram::Diagram0(generator) => f.debug_tuple("Diagram0").field(generator).finish(),
            Diagram::DiagramN(diagram) => diagram.fmt(f),
        }
    }
}

impl From<DiagramN> for Diagram {
    fn from(diagram: DiagramN) -> Self {
        Diagram::DiagramN(diagram)
    }
}

impl From<Generator> for Diagram {
    fn from(generator: Generator) -> Self {
        Diagram::Diagram0(generator)
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

#[derive(PartialEq, Eq, Clone)]
struct DiagramInternal {
    source: Diagram,
    cospans: Vec<Cospan>,
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
        Slices {
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
                current.clone().rewrite_forward(&cospan.forward)
            }
            Direction::Backward => {
                let cospan = self.cospans.pop().unwrap();
                self.direction = Direction::Forward;
                current.clone().rewrite_backward(&cospan.backward)
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

#[cfg(test)]
mod test {
    use super::*;
    use std::error::Error;

    fn assert_point_ids<D>(diagram: D, points: &[(&[usize], usize)])
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
        use Boundary::*;

        let x = Diagram::from(Generator::new(0, 0));
        let f = DiagramN::new(Generator::new(1, 1), x.clone(), x.clone())?;
        let ff = f.attach(f.clone(), Target, &[])?;
        let m = DiagramN::new(Generator::new(2, 2), ff.clone(), f.clone())?;
        let left = m.attach(m.clone(), Source, &[0])?;

        assert_point_ids(
            left,
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
        let f = DiagramN::new(Generator::new(1, 2), x.identity(), x.identity()).unwrap();

        assert_eq!(f.source(), x.identity().into());
        assert_eq!(f.target(), x.identity().into());

        let cospan = &f.cospans()[0];
        let forward: &RewriteN = (&cospan.forward).try_into().unwrap();

        assert_eq!(forward.singular_image(0), 1);
        assert_eq!(forward.regular_preimage(0), 0..2);
    }
}
