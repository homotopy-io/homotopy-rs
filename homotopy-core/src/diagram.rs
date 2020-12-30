use crate::attach::*;
use crate::common::*;
use crate::rewrite::*;
use std::convert::TryFrom;
use std::convert::*;
use std::rc::Rc;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
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

        match (&source, &target) {
            (Diagram::DiagramN(source), Diagram::DiagramN(target)) => {
                if source.source() != target.source() || source.target() != target.target() {
                    return Err(NewDiagramError::NonGlobular);
                }
            }
            _ => {}
        }

        let cospan = Cospan {
            forward: Rewrite::cone_over_generator(generator, source.clone()),
            backward: Rewrite::cone_over_generator(generator, target),
        };

        Ok(DiagramN::new_unsafe(source, vec![cospan]))
    }

    pub fn new_unsafe(source: Diagram, cospans: Vec<Cospan>) -> Self {
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

    pub fn singular_slices(&self) -> Vec<Diagram> {
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
        let mut offset: isize = 0;

        for cone in rewrite.cones() {
            let start = (cone.index as isize + offset) as usize;
            let stop = (cone.index as isize + 1 + offset) as usize;
            diagram.cospans.splice(start..stop, cone.source.clone());
            offset += cone.len() as isize - 1;
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

        // if depth == 0 {
        //     let cospans: Vec<_> = diagram
        //         .cospans()
        //         .iter()
        //         .map(|c| c.pad(&embedding))
        //         .collect();

        //     match boundary {
        //         Source => {
        //             let mut source = self.0.source.clone();

        //             if !source.embeds(&diagram.target(), embedding) {
        //                 return Err(AttachmentError::Incompatible);
        //             }

        //             for cospan in cospans.iter().rev() {
        //                 source = source.rewrite_forward(&cospan.backward);
        //                 source = source.rewrite_backward(&cospan.forward);
        //             }

        //             let mut result_cospans = Vec::new();
        //             result_cospans.extend(cospans.into_iter());
        //             result_cospans.extend(self.0.cospans.to_vec().into_iter());

        //             Ok(DiagramN::new_unsafe(source, result_cospans))
        //         }
        //         Target => {
        //             if !self.target().embeds(&diagram.source(), embedding) {
        //                 return Err(AttachmentError::Incompatible);
        //             }

        //             let mut result_cospans = Vec::new();
        //             result_cospans.extend(self.0.cospans.to_vec().into_iter());
        //             result_cospans.extend(cospans.into_iter());

        //             Ok(DiagramN::new_unsafe(self.0.source.clone(), result_cospans))
        //         }
        //     }
        // } else {
        //     let source = match &self.0.source {
        //         Diagram::Diagram0(_) => panic!(),
        //         Diagram::DiagramN(s) => s,
        //     };

        //     match boundary {
        //         Source => {
        //             let source = Diagram::DiagramN(source.attach(diagram, boundary, embedding)?);
        //             // TODO: Pad by 1 or by the size of `diagram`?
        //             let mut padding = vec![0; depth - 1];
        //             padding.push(1);
        //             let cospans = self.0.cospans.iter().map(|c| c.pad(&padding)).collect();
        //             Ok(DiagramN::new_unsafe(source, cospans))
        //         }
        //         Target => {
        //             let source = Diagram::DiagramN(source.attach(diagram, boundary, embedding)?);
        //             let cospans = self.0.cospans.clone();
        //             Ok(DiagramN::new_unsafe(source, cospans))
        //         }
        //     }
        // }
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
    type Error = ();

    fn try_from(from: Diagram) -> Result<Self, Self::Error> {
        match from {
            Diagram::DiagramN(from) => Ok(from),
            Diagram::Diagram0(_) => Err(()),
        }
    }
}

impl<'a> TryFrom<&'a Diagram> for &'a DiagramN {
    type Error = ();

    fn try_from(from: &'a Diagram) -> Result<Self, Self::Error> {
        match from {
            Diagram::DiagramN(from) => Ok(from),
            Diagram::Diagram0(_) => Err(()),
        }
    }
}

impl TryFrom<Diagram> for Generator {
    type Error = ();

    fn try_from(from: Diagram) -> Result<Self, Self::Error> {
        match from {
            Diagram::DiagramN(_) => Err(()),
            Diagram::Diagram0(g) => Ok(g),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct DiagramInternal {
    source: Diagram,
    cospans: Vec<Cospan>,
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

    fn example_assoc() -> DiagramN {
        let x = Generator {
            id: 0,
            dimension: 0,
        };
        let f = Generator {
            id: 1,
            dimension: 1,
        };
        let m = Generator {
            id: 2,
            dimension: 2,
        };
        let a = Generator {
            id: 3,
            dimension: 3,
        };

        let fd = DiagramN::new(f, x, x).unwrap();
        let ffd = fd.attach(fd.clone(), Boundary::Target, &[]).unwrap();
        let md = DiagramN::new(m, ffd, fd).unwrap();
        let ld = md.attach(md.clone(), Boundary::Source, &[0]).unwrap();
        let rd = md.attach(md.clone(), Boundary::Source, &[1]).unwrap();
        let ad = DiagramN::new(a, ld, rd).unwrap();

        ad
    }

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
    fn associativity_points() {
        let d = example_assoc().source();

        assert_point_ids(
            d,
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
    }

    #[test]
    fn scalar() {
        let x = Generator::new(0, 0);
        let f = Generator::new(1, 2);

        let xd = Diagram::from(x);
        let fd = DiagramN::new(f, xd.identity(), xd.identity()).unwrap();

        assert_eq!(fd.source(), xd.identity().into());
        assert_eq!(fd.target(), xd.identity().into());

        let cospan = &fd.cospans()[0];
        let forward = cospan.forward.to_n().unwrap();
        assert_eq!(forward.regular_preimage(0), 0..2);
    }
}
