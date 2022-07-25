use std::ops::Range;

use itertools::{Itertools, MultiProduct};

use crate::{
    monotone::{MonotoneIterator, Split},
    rewrite::Cone,
    Diagram, Height, Rewrite, RewriteN,
};

/// Given `Rewrite`s A -f> C <g- B, find some `Rewrite` A -h> B which factorises f = g ∘ h.
pub fn factorize(f: Rewrite, g: Rewrite, source: Diagram, target: Diagram) -> Factorization {
    if g.is_identity() {
        return Factorization::Unique(f.into());
    }

    if f == g {
        return Factorization::Unique(Rewrite::identity(f.dimension()).into());
    }

    match (f, g, source, target) {
        (
            Rewrite::Rewrite0(f),
            Rewrite::Rewrite0(g),
            Diagram::Diagram0(s),
            Diagram::Diagram0(t),
        ) => {
            assert!(f.source() == None || f.source() == Some(s));
            assert!(g.source() == None || g.source() == Some(t));

            Factorization::Unique((s == t).then(|| Rewrite::identity(0)))
        }
        (
            Rewrite::RewriteN(f),
            Rewrite::RewriteN(g),
            Diagram::DiagramN(source),
            Diagram::DiagramN(target),
        ) => {
            assert_eq!(f.dimension(), g.dimension());
            assert_eq!(f.dimension(), source.dimension());
            assert_eq!(g.dimension(), target.dimension());

            let common = source
                .clone()
                .rewrite_forward(&f)
                .expect("malformed input for factorization");
            assert_eq!(
                &common,
                &target
                    .clone()
                    .rewrite_forward(&g)
                    .expect("malformed input for factorization")
            );

            let sources = source.slices().collect::<Vec<_>>();
            let targets = target.slices().collect::<Vec<_>>();
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
                (0..common.size())
                    .map(|i| {
                        let f_cone = f.cone_over_target(i).cloned();
                        if let Some(c) = &f_cone {
                            offset = c.index;
                        } else {
                            offset += 1;
                        }
                        match g.cone_over_target(i).cloned() {
                            None => ConeFactorization::Unique(
                                f_cone.map(|c| vec![c]).unwrap_or_default(),
                            ),
                            g_cone if f_cone == g_cone => ConeFactorization::Unique(vec![]),
                            Some(g_cone) => {
                                let f_cone_len = f_cone.as_ref().map_or(1, Cone::len);
                                let constraints: Vec<Range<usize>> =
                                    vec![0..g_cone.singular_slices().len(); f_cone_len];
                                let monotone = MonotoneIterator::new(false, &constraints);
                                let source = sources[offset..offset + f_cone_len].to_vec();
                                let target =
                                    targets[g_cone.index..g_cone.index + g_cone.len()].to_vec();
                                ConeFactorization::Iterator(ConeFactorizationInternal {
                                    f_cone,
                                    g_cone,
                                    source,
                                    target,
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
    Unique(Vec<Cone>),
    Iterator(ConeFactorizationInternal),
}

impl Iterator for ConeFactorization {
    type Item = Vec<Cone>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ConeFactorization::Unique(cones) => std::iter::once(cones.clone()).next(),
            ConeFactorization::Iterator(cones) => cones.next(),
        }
    }
}

impl std::iter::FusedIterator for ConeFactorization {}

#[derive(Clone)]
pub struct ConeFactorizationInternal {
    f_cone: Option<Cone>,
    g_cone: Cone,
    source: Vec<Diagram>,
    target: Vec<Diagram>,
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
        let slices = self.slices_iterator.next();
        slices.map(|slices| {
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
            // TODO: check cone here for well-formedness
            cone
        })
    }
}

impl Iterator for ConeFactorizationInternal {
    type Item = Vec<Cone>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.cur {
                None => {
                    let underlying = self.monotone.next()?;
                    let slices_iterator = underlying.cones().map(|Split { source, target }| {
                        let (g_slice, g_source) = (
                            &self.g_cone.singular_slices()[target],
                            &self.target[usize::from(Height::Singular(target))],
                        );
                        let dimension = g_slice.dimension();
                        let product = (usize::from(Height::Regular(source.start))
                            ..=usize::from(Height::Regular(source.end + 1)))
                            .map(|h| {
                                let (f_slice, f_source) = (
                                    self.f_cone.as_ref().map_or(
                                        Rewrite::identity(dimension),
                                        |c| match Height::from(h) {
                                            Height::Singular(i) => c.singular_slices()[i].clone(),
                                            Height::Regular(i) => c.regular_slices()[i].clone(),
                                        },
                                    ),
                                    self.source[h].clone(),
                                );
                                factorize(f_slice, g_slice.clone(), f_source, g_source.clone())
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
