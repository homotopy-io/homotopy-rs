use std::ops::Range;

use itertools::{Itertools, MultiProduct};

use crate::{
    common::Mode,
    monotone::{MonotoneIterator, Split},
    rewrite::Cone,
    Cospan, Diagram, Height, Rewrite, RewriteN,
};

/// Given `Rewrite`s A -f> C <g- B, find some `Rewrite` A -h> B which factorises f = g ∘ h.
pub fn factorize(f: Rewrite, g: Rewrite, target: Diagram) -> Factorization {
    if g.is_identity() {
        return Factorization::Unique(f.into());
    }

    if f.equals_modulo_labels(&g) {
        return Factorization::Unique(Rewrite::identity(f.dimension()).into());
    }

    match (f, g, target) {
        (Rewrite::Rewrite0(f), Rewrite::Rewrite0(g), Diagram::Diagram0(t)) => {
            assert!(f.target().is_none() || f.target() == Some(t));
            assert!(g.target().is_none() || g.target() == Some(t));

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
                            Some(g_cone)
                                if f_cone.as_ref().map_or(false, |c| {
                                    c.source() == g_cone.source()
                                        && c.target() == g_cone.target()
                                        && std::iter::zip(
                                            c.regular_slices(),
                                            g_cone.regular_slices(),
                                        )
                                        .all(|(f, g)| f.equals_modulo_labels(g))
                                        && std::iter::zip(
                                            c.singular_slices(),
                                            g_cone.singular_slices(),
                                        )
                                        .all(|(f, g)| f.equals_modulo_labels(g))
                                }) =>
                            {
                                ConeFactorization::Unique(vec![].into())
                            }
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
                                    offset,
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
    offset: usize,
    cur: Option<MultiProduct<ConeIterator>>,
}

impl Iterator for ConeFactorizationInternal {
    type Item = Vec<Cone>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.cur {
                None => {
                    self.cur = self
                        .monotone
                        .next()?
                        .cones(self.g_cone.len())
                        .map(|Split { source, target }| {
                            let f_slice = |h: Height| {
                                self.f_cone.as_ref().map_or_else(
                                    || Rewrite::identity(self.singular.dimension()),
                                    |f_cone| f_cone.slice(h).clone(),
                                )
                            };
                            let g_slice = &self.g_cone.singular_slices()[target];

                            let slices_product = (usize::from(Height::Regular(source.start))
                                ..=usize::from(Height::Regular(source.end)))
                                .map(|i| {
                                    factorize(
                                        f_slice(Height::from(i)),
                                        g_slice.clone(),
                                        self.singular.clone(),
                                    )
                                })
                                .multi_cartesian_product();

                            ConeIterator {
                                slices_product,
                                index: self.offset + source.start,
                                source: if source.is_empty() {
                                    vec![]
                                } else {
                                    self.f_cone.as_ref().map_or_else(
                                        || vec![self.g_cone.target().clone()],
                                        |f_cone| f_cone.source()[source].to_vec(),
                                    )
                                },
                                target: self.g_cone.source()[target].clone(),
                            }
                        })
                        .multi_cartesian_product()
                        .into();
                }
                Some(cone_factorizations) => match cone_factorizations.next() {
                    None => self.cur = None,
                    Some(slices) => return Some(slices),
                },
            }
        }
    }
}

impl std::iter::FusedIterator for ConeFactorizationInternal {}

#[derive(Clone)]
struct ConeIterator {
    index: usize,
    source: Vec<Cospan>,
    target: Cospan,
    slices_product: MultiProduct<Factorization>,
}

impl Iterator for ConeIterator {
    type Item = Cone;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let slices = self.slices_product.next()?;
            let cone = Cone::new(
                self.index,
                self.source.clone(),
                self.target.clone(),
                slices.iter().step_by(2).cloned().collect_vec(),
                slices.into_iter().skip(1).step_by(2).collect_vec(),
            );
            if cone.check(Mode::Shallow).is_ok() {
                return Some(cone);
            }
        }
    }
}

impl std::iter::FusedIterator for ConeIterator {}
