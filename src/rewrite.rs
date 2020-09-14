use crate::common::*;
use crate::diagram::*;
use std::rc::Rc;
use std::ops::Range;

#[derive(PartialEq, Eq, Clone)]
pub struct Cospan {
    pub forward: Rewrite,
    pub backward: Rewrite,
}

impl Cospan {
    fn pad(&self, embedding: &[usize]) -> Self {
        let forward = self.forward.pad(embedding);
        let backward = self.backward.pad(embedding);
        Cospan { forward, backward }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum Rewrite {
    RewriteI,
    Rewrite0(Generator, Generator),
    RewriteN(RewriteN),
}

impl Rewrite {
    pub fn identity(dimension: usize) -> Self {
        if dimension == 0 {
            Rewrite::RewriteI
        } else {
            Rewrite::RewriteN(RewriteN {
                dimension,
                cones: Vec::new(),
            })
        }
    }

    pub fn dimension(&self) -> usize {
        use Rewrite::*;
        match self {
            RewriteI => 0,
            Rewrite0(_, _) => 0,
            RewriteN(r) => r.dimension(),
        }
    }

    fn pad(&self, embedding: &[usize]) -> Self {
        use Rewrite::*;
        match self {
            RewriteI => RewriteI,
            Rewrite0(s, t) => Rewrite0(*s, *t),
            RewriteN(r) => RewriteN(r.pad(embedding)),
        }
    }

    pub fn to_n(&self) -> Option<&RewriteN> {
        use Rewrite::*;
        match self {
            RewriteI => None,
            Rewrite0(_, _) => None,
            RewriteN(r) => Some(r),
        }
    }

    pub fn compose(f: Rewrite, g: Rewrite) -> Rewrite {
        match (f, g) {
            (Rewrite::RewriteI, Rewrite::Rewrite0(g_source, g_target)) => {
                Rewrite::Rewrite0(g_source, g_target)
            }
            (Rewrite::Rewrite0(f_source, f_target), Rewrite::RewriteI) => {
                Rewrite::Rewrite0(f_source, f_target)
            }
            (Rewrite::RewriteI, Rewrite::RewriteI) => Rewrite::RewriteI,

            (Rewrite::Rewrite0(f_source, f_target), Rewrite::Rewrite0(g_source, g_target)) => {
                if f_target == g_source {
                    Rewrite::Rewrite0(f_source, g_target)
                } else {
                    panic!()
                }
            }

            (Rewrite::RewriteN(f), Rewrite::RewriteN(g)) => {
                Rewrite::RewriteN(RewriteN::compose(f, g))
            }

            (_, _) => panic!(),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct RewriteN {
    dimension: usize,
    cones: Vec<Rc<Cone>>,
}

impl RewriteN {
    fn pad(&self, embedding: &[usize]) -> Self {
        let cones = self
            .cones
            .iter()
            .map(|cone| Rc::new(cone.pad(embedding)))
            .collect();
        let dimension = self.dimension;
        RewriteN { cones, dimension }
    }

    pub fn from_slices(
        source: &DiagramN,
        target: &DiagramN,
        slices: Vec<Vec<Rewrite>>,
    ) -> RewriteN {
        let mut cones = Vec::new();
        let mut index = 0;
        let source_cospans = source.cospans();
        let target_cospans = target.cospans();

        for (target, cone_slices) in slices.into_iter().enumerate() {
            // TODO: Detect identities
            let size = cone_slices.len();
            cones.push(Rc::new(Cone {
                source: source_cospans[index..index + size].to_vec(),
                target: target_cospans[target].clone(),
                slices: cone_slices,
                index: index,
            }));
            index += size;
        }

        RewriteN {
            cones,
            dimension: source.dimension(),
        }
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn slice(&self, height: usize) -> Rewrite {
        self.cones
            .iter()
            .find(|cone| cone.index <= height && height < cone.index + cone.len())
            .map(|cone| cone.slices[height - cone.index].clone())
            .unwrap_or(Rewrite::identity(self.dimension() - 1))
    }

    pub fn compose(f: RewriteN, g: RewriteN) -> RewriteN {
        if f.dimension() != g.dimension() {
            panic!()
        }

        let mut offset = 0;
        let mut f_cones: Vec<Rc<Cone>> = f.cones.iter().rev().cloned().collect();
        let mut g_cones: Vec<Rc<Cone>> = g.cones.iter().rev().cloned().collect();
        let mut cones: Vec<Rc<Cone>> = Vec::new();

        loop {
            match (f_cones.pop(), g_cones.pop()) {
                (None, None) => break,
                (Some(f_cone), None) => cones.push(f_cone.clone()),
                (None, Some(g_cone)) => {
                    let mut cone: Cone = g_cone.as_ref().clone();
                    cone.index = (cone.index as isize + offset) as usize;
                    cones.push(Rc::new(cone));
                }
                (Some(f_cone), Some(g_cone)) => {
                    let index = f_cone.index as isize - g_cone.index as isize + offset;

                    if index >= g_cone.len() as isize {
                        let mut cone = g_cone.as_ref().clone();
                        cone.index = (cone.index as isize + offset) as usize;
                        cones.push(Rc::new(cone));
                        f_cones.push(f_cone);
                    } else if index < 0 {
                        cones.push(f_cone.clone());
                        g_cones.push(g_cone);
                        offset += 1 - f_cone.len() as isize;
                    } else {
                        let index = index as usize;

                        if f_cone.target != g_cone.source[index] {
                            panic!();
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
                                .map(|f_slice| Rewrite::compose(f_slice.clone(), g_slice.clone())),
                        );
                        slices.extend(g_cone.slices[index + 1..].iter().cloned());

                        g_cones.push(Rc::new(Cone {
                            index: (g_cone.index as isize + offset) as usize,
                            source: source,
                            target: g_cone.target.clone(),
                            slices: slices,
                        }));
                    }
                }
            }
        }

        RewriteN {
            dimension: f.dimension(),
            cones,
        }
    }

    pub fn singular_image(&self, index: usize) -> usize {
        let mut offset: isize = 0;

        for cone in &self.cones {
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

        for cone in &self.cones {
            let adjusted = (index as isize - offset) as usize;
            if adjusted < cone.index {
                return adjusted..adjusted + 1;
            } else if adjusted == cone.index {
                return cone.index..cone.index + cone.len();
            } else {
                offset += 1 - cone.len() as isize;
            }
        }

        let adjusted = (index as isize - offset) as usize;
        adjusted..adjusted + 1
    }

    pub fn regular_image(&self, index: usize) -> usize {
        let mut offset = 0;

        for cone in &self.cones {
            if index < (cone.index as isize + offset) as usize {
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
            self.singular_image(index - 1)
        };

        let right = self.singular_image(index) + 1;
        left..right
    }
}

#[derive(PartialEq, Eq, Clone)]
struct Cone {
    index: usize,
    source: Vec<Cospan>,
    target: Cospan,
    slices: Vec<Rewrite>,
}

impl Cone {
    fn len(&self) -> usize {
        self.source.len()
    }

    fn pad(&self, embedding: &[usize]) -> Self {
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

