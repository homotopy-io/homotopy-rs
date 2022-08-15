use std::ops::Range;

use itertools::{Itertools, MultiProduct};

use crate::{
    monotone::{MonotoneIterator, Split},
    rewrite::Cone,
    Diagram, Height, Rewrite, RewriteN,
};

/// Given `Rewrite`s A -f> C <g- B, find some `Rewrite` A -h> B which factorises f = g ∘ h.
pub fn factorize(f: Rewrite, g: Rewrite, target: Diagram) -> Factorization {
    if g.is_identity() {
        return Factorization::Unique(f.into());
    }

    if f == g {
        return Factorization::Unique(Rewrite::identity(f.dimension()).into());
    }

    match (f, g, target) {
        (Rewrite::Rewrite0(f), Rewrite::Rewrite0(g), Diagram::Diagram0(t)) => {
            assert!(f.target() == None || f.target() == Some(t));
            assert!(g.target() == None || g.target() == Some(t));

            Factorization::Unique(None)
        }
        (Rewrite::RewriteN(f), Rewrite::RewriteN(g), Diagram::DiagramN(target)) => {
            assert_eq!(f.dimension(), g.dimension());
            assert_eq!(f.dimension(), target.dimension());

            let cones = {
                // the defining property of cones is that each singular height in the
                // target of a rewrite corresponds to exactly one (possibly sparse) cone,
                // in order
                //
                // for each singular height in the common target of f and g, there is a
                // corresponding f_cone and g_cone
                // a monotone function is then guessed from the base of f_cone to the base
                // of g_cone
                // this determines a number of h_cones to 'fill' this monotone function (0
                // when g_cone.len() = 0), represented as a Vec<Cone>
                // ultimately, obtain Vec<Vec<Cone>> with length = #singular heights in the
                // common target of f and g, whose concatenation give the cones of h
                let mut offset = 0;
                target
                    .singular_slices()
                    .into_iter()
                    .enumerate()
                    .map(|(i, singular)| {
                        let f_cone = f.cone_over_target(i).cloned();
                        if let Some(c) = &f_cone {
                            offset = c.index;
                        } else {
                            offset += 1;
                        }
                        match g.cone_over_target(i).cloned() {
                            None => ConeFactorization::Unique(
                                f_cone.map(|c| vec![c]).unwrap_or_default().into(),
                            ),
                            g_cone if f_cone == g_cone => ConeFactorization::Unique(vec![].into()),
                            Some(g_cone) => {
                                let f_cone_len = f_cone.as_ref().map_or(1, Cone::len);
                                let constraints: Vec<Range<usize>> =
                                    vec![0..g_cone.singular_slices().len(); f_cone_len];
                                let monotone = MonotoneIterator::new(false, &constraints);
                                ConeFactorization::Iterator(ConeFactorizationInternal {
                                    f_cone,
                                    g_cone,
                                    singular,
                                    monotone,
                                    base_offset: offset,
                                    cur: None,
                                })
                            }
                        }
                    })
                    .multi_cartesian_product()
            };

            Factorization::Iterator(f.dimension(), cones)
        }
        _ => panic!("Mismatched dimensions"),
    }
}

#[derive(Clone)]
pub enum Factorization {
    Unique(Option<Rewrite>),
    Iterator(usize, MultiProduct<ConeFactorization>),
}

impl Iterator for Factorization {
    type Item = Rewrite;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Unique(h) => h.take(),
            Self::Iterator(dim, cones) => cones
                .next()
                .map(|cs| RewriteN::new(*dim, cs.concat()).into()),
        }
    }
}

impl std::iter::FusedIterator for Factorization {}

#[derive(Clone)]
pub enum ConeFactorization {
    Unique(Option<Vec<Cone>>),
    Iterator(ConeFactorizationInternal),
}

impl Iterator for ConeFactorization {
    type Item = Vec<Cone>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ConeFactorization::Unique(cones) => cones.take(),
            ConeFactorization::Iterator(cones) => cones.next(),
        }
    }
}

impl std::iter::FusedIterator for ConeFactorization {}

#[derive(Clone)]
pub struct ConeFactorizationInternal {
    f_cone: Option<Cone>,
    g_cone: Cone,
    singular: Diagram,
    monotone: MonotoneIterator,
    base_offset: usize,
    cur: Option<MultiProduct<SlicesToCones>>,
}

#[derive(Clone)]
struct SlicesToCones {
    slices_iterator: MultiProduct<Factorization>,
    base_offset: usize,
    source: Range<usize>,
    target: usize,
    f_cone: Option<Cone>,
    g_cone: Cone,
}

impl Iterator for SlicesToCones {
    type Item = Cone;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let slices = self.slices_iterator.next()?;
            let mut regular_slices = Vec::with_capacity(slices.len() / 2 + 1);
            let mut singular_slices = Vec::with_capacity(slices.len() / 2);
            for (i, slice) in slices.into_iter().enumerate() {
                if i % 2 == 0 {
                    regular_slices.push(slice);
                } else {
                    singular_slices.push(slice);
                }
            }
            let cone = Cone::new(
                self.base_offset + self.source.start,
                self.f_cone.as_ref().map_or_else(
                    || vec![self.g_cone.target().clone()],
                    |c| c.source()[self.source.clone()].to_vec(),
                ),
                self.g_cone.source()[self.target].clone(),
                regular_slices,
                singular_slices,
            );
            if cone.check().is_ok() {
                return Some(cone);
            }
        }
    }
}

impl Iterator for ConeFactorizationInternal {
    type Item = Vec<Cone>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.cur {
                None => {
                    let underlying = self.monotone.next()?;
                    let slices_iterator =
                        underlying
                            .cones(self.g_cone.len())
                            .map(|Split { source, target }| {
                                let g_slice = &self.g_cone.singular_slices()[target];
                                let dimension = g_slice.dimension();
                                let product = (usize::from(Height::Regular(source.start))
                                    ..=usize::from(Height::Regular(source.end)))
                                    .map(|h| {
                                        let f_slice = self
                                            .f_cone
                                            .as_ref()
                                            .map_or(Rewrite::identity(dimension), |c| {
                                                c.slice(Height::from(h)).clone()
                                            });
                                        factorize(f_slice, g_slice.clone(), self.singular.clone())
                                    })
                                    .multi_cartesian_product();
                                (Split { source, target }, product)
                            });
                    self.cur = Some({
                        slices_iterator
                            .map(|(Split { source, target }, product)| SlicesToCones {
                                base_offset: self.base_offset,
                                f_cone: self.f_cone.clone(),
                                g_cone: self.g_cone.clone(),
                                source,
                                target,
                                slices_iterator: product,
                            })
                            .multi_cartesian_product()
                    });
                }
                Some(cone_factorizations) => match cone_factorizations.next() {
                    Some(slices) => return Some(slices),
                    None => {
                        self.cur = None;
                    }
                },
            }
        }
    }
}

impl std::iter::FusedIterator for ConeFactorizationInternal {}
