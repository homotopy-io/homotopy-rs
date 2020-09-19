use crate::common::*;
use crate::diagram::*;

use std::cmp::Ordering;
use std::ops::Range;
use std::rc::Rc;
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
        Cospan { forward, backward }
    }

    pub fn is_identity(&self) -> bool {
        self.forward.is_identity() && self.backward.is_identity()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Rewrite {
    Rewrite0(Rewrite0),
    RewriteN(RewriteN),
}

// impl fmt::Debug for Rewrite {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Rewrite::Rewrite0(None) => f.debug_struct("Rewrite0").finish(),
//             Rewrite::Rewrite0(Some((s, t))) => f
//                 .debug_struct("Rewrite0")
//                 .field("source", s)
//                 .field("target", t)
//                 .finish(),
//             Rewrite::RewriteN(r) => r.fmt(f),
//         }
//     }
// }

impl From<RewriteN> for Rewrite {
    fn from(r: RewriteN) -> Self {
        Rewrite::RewriteN(r)
    }
}

impl From<Rewrite0> for Rewrite {
    fn from(r: Rewrite0) -> Self {
        Rewrite::Rewrite0(r)
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
        use Rewrite::*;
        match self {
            Rewrite0(_) => 0,
            RewriteN(r) => r.dimension(),
        }
    }

    pub fn is_identity(&self) -> bool {
        use Rewrite::*;
        match self {
            Rewrite0(r) => r.is_identity(),
            RewriteN(r) => r.is_identity(),
        }
    }

    pub(crate) fn pad(&self, embedding: &[usize]) -> Self {
        use Rewrite::*;
        match self {
            Rewrite0(r) => Rewrite0(*r),
            RewriteN(r) => RewriteN(r.pad(embedding)),
        }
    }

    pub fn to_n(&self) -> Option<&RewriteN> {
        use Rewrite::*;
        match self {
            Rewrite0(_) => None,
            RewriteN(r) => Some(r),
        }
    }

    pub fn compose(f: Rewrite, g: Rewrite) -> Result<Rewrite, CompositionError> {
        match (f, g) {
            (Rewrite::Rewrite0(f), Rewrite::Rewrite0(g)) => Ok(Rewrite0::compose(f, g)?.into()),
            (Rewrite::RewriteN(f), Rewrite::RewriteN(g)) => Ok(RewriteN::compose(f, g)?.into()),
            (f, g) => Err(CompositionError::Dimension(f.dimension(), g.dimension())),
        }
    }

    pub fn cone_over_generator(generator: Generator, base: Diagram) -> Rewrite {
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
                        .map(|slice| Rewrite::cone_over_generator(generator, slice))
                        .collect(),
                    target: Cospan {
                        forward: Rewrite::cone_over_generator(generator, base.source()),
                        backward: Rewrite::cone_over_generator(generator, base.target()),
                    },
                }],
            )
            .into(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Rewrite0(pub(crate) Option<(Generator, Generator)>);

impl Rewrite0 {
    pub fn new(source: Generator, target: Generator) -> Self {
        if source == target {
            Rewrite0(None)
        } else {
            Rewrite0(Some((source, target)))
        }
    }

    pub fn identity() -> Self {
        Rewrite0(None)
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
                if f_t != g_s {
                    Err(CompositionError::Incompatible)
                } else {
                    Ok(Rewrite0(Some((f_s, g_t))))
                }
            }
            (Some(_), None) => Ok(f),
            (None, Some(_)) => Ok(g),
            (None, None) => Ok(Rewrite0(None)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct RewriteN(Rc<RewriteInternal>);

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct RewriteInternal {
    dimension: usize,
    cones: Vec<Cone>,
}

impl RewriteN {
    pub(crate) fn new(dimension: usize, cones: Vec<Cone>) -> Self {
        if dimension == 0 {
            panic!("Can not create RewriteN of dimension zero.");
        }

        let cones = cones
            .into_iter()
            .filter(|cone| !cone.is_identity())
            .collect();

        RewriteN(Rc::new(RewriteInternal { dimension, cones }))
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
        RewriteN::new(self.dimension(), cones)
    }

    pub fn identity(dimension: usize) -> Self {
        RewriteN::new(dimension, Vec::new())
    }

    pub fn is_identity(&self) -> bool {
        self.0.cones.is_empty()
    }

    pub(crate) fn make_degeneracy(
        dimension: usize,
        target_cospans: &[Cospan],
        trivial_heights: &[SingularHeight],
    ) -> Self {
        let cones = trivial_heights
            .iter()
            .enumerate()
            .map(|(i, height)| Cone {
                index: height - i,
                source: vec![],
                target: target_cospans[*height].clone(),
                slices: vec![],
            })
            .collect();

        RewriteN::new(dimension, cones)
    }

    pub fn from_slices(
        dimension: usize,
        source_cospans: &[Cospan],
        target_cospans: &[Cospan],
        slices: Vec<Vec<Rewrite>>,
    ) -> RewriteN {
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

        RewriteN::new(dimension, cones)
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
            .map(|cone| cone.slices[height - cone.index].clone())
            .unwrap_or_else(|| Rewrite::identity(self.dimension() - 1))
    }

    pub fn compose(f: RewriteN, g: RewriteN) -> Result<RewriteN, CompositionError> {
        if f.dimension() != g.dimension() {
            return Err(CompositionError::Dimension(f.dimension(), g.dimension()));
        }

        let mut offset = 0;
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
                    cones.push(cone);
                }
                (Some(f_cone), Some(g_cone)) => {
                    let index = f_cone.index as isize - g_cone.index as isize + offset;

                    if index >= g_cone.len() as isize {
                        let mut cone = g_cone.clone();
                        cone.index = (cone.index as isize + offset) as usize;
                        cones.push(cone);
                        f_cones.push(f_cone);
                    } else if index < 0 {
                        cones.push(f_cone.clone());
                        g_cones.push(g_cone);
                        offset += 1 - f_cone.len() as isize;
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

                        g_cones.push(Cone {
                            index: (g_cone.index as isize + offset) as usize,
                            source,
                            target: g_cone.target.clone(),
                            slices,
                        });
                    }
                }
            }
        }

        Ok(RewriteN::new(f.dimension(), cones))
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
        let left = if index == 0 {
            0
        } else {
            self.singular_image(index - 1) + 1
        };

        let right = self.singular_image(index) + 1;
        left..right
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
                Cone {
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
