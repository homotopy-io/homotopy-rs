use homotopy_common::iter::ZeroOneMany;
use itertools::{Either, Itertools, MultiProduct};

use crate::{
    common::Mode,
    monotone::{MonotoneIterator, Split},
    rewrite::Cone,
    Cospan, Height, Rewrite, Rewrite0, RewriteN,
};

pub type Factorization = ZeroOneMany<FactorizationInternal>;
pub type ConeFactorization = ZeroOneMany<ConeFactorizationInternal>;

/// Given `Rewrite`s A -f> C <g- B, find some `Rewrite` A -h> B which factorises f = g ∘ h.
pub fn factorize(f: &Rewrite, g: &Rewrite) -> Factorization {
    match (f, g) {
        (Rewrite::Rewrite0(f), Rewrite::Rewrite0(g)) => {
            assert!(f
                .target()
                .zip(g.target())
                .map_or(true, |(f_t, g_t)| f_t == g_t));

            match g.source() {
                None => Factorization::One(f.clone().into()),
                Some(g_s) => {
                    match f
                        .source()
                        .filter(|f_s| f_s.generator.dimension <= g_s.generator.dimension)
                    {
                        None => Factorization::Empty,
                        Some(f_s) => Factorization::One(Rewrite0::new(f_s, g_s, None).into()),
                    }
                }
            }
        }
        (Rewrite::RewriteN(f), Rewrite::RewriteN(g)) => {
            assert_eq!(f.dimension(), g.dimension());

            if g.is_identity() {
                return Factorization::One(f.clone().into());
            }

            if f.equivalent(g) {
                return Factorization::One(Rewrite::identity(f.dimension()));
            }

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
                array_cones([f, g])
                    .filter_map(|(_, [f_cone, g_cone])| {
                        let f_cone = f_cone.map_left(Clone::clone);
                        match g_cone.left().cloned() {
                            None => Some(ConeFactorization::One(vec![f_cone.unwrap_left()])),
                            Some(g_cone) => {
                                // If the two cones are equivalent, skip them since the factorization is trivial.
                                if f_cone
                                    .as_ref()
                                    .either(|f_cone| f_cone.equivalent(&g_cone), |_| false)
                                {
                                    return None;
                                }

                                let f_cone_len = f_cone.as_ref().either(Cone::len, |_| 1);
                                let monotone =
                                    MonotoneIterator::new(false, vec![0..g_cone.len(); f_cone_len]);
                                Some(ConeFactorization::Many(ConeFactorizationInternal {
                                    f_cone,
                                    g_cone,
                                    monotone,
                                    cur: None,
                                }))
                            }
                        }
                    })
                    .multi_cartesian_product()
            };

            Factorization::Many(FactorizationInternal {
                dimension: f.dimension(),
                cones,
            })
        }
        _ => panic!("Mismatched dimensions"),
    }
}

#[derive(Clone)]
pub struct FactorizationInternal {
    dimension: usize,
    cones: MultiProduct<ConeFactorization>,
}

impl Iterator for FactorizationInternal {
    type Item = Rewrite;

    fn next(&mut self) -> Option<Self::Item> {
        self.cones
            .next()
            .map(|cs| RewriteN::new(self.dimension, cs.concat()).into())
    }
}

impl std::iter::FusedIterator for FactorizationInternal {}

#[derive(Clone)]
pub struct ConeFactorizationInternal {
    f_cone: Either<Cone, usize>,
    g_cone: Cone,
    monotone: MonotoneIterator,
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
                            let g_slice = &self.g_cone.singular_slices()[target];
                            let id = Rewrite::identity(g_slice.dimension());
                            let f_slice = |h: Height| {
                                self.f_cone
                                    .as_ref()
                                    .either(|f_cone| f_cone.slice(h), |_| &id)
                            };

                            let slices_product = (usize::from(Height::Regular(source.start))
                                ..=usize::from(Height::Regular(source.end)))
                                .map(|i| factorize(f_slice(Height::from(i)), g_slice))
                                .multi_cartesian_product();

                            let offset = self.f_cone.as_ref().either(|f_cone| f_cone.index, |i| *i);
                            ConeIterator {
                                slices_product,
                                index: offset + source.start,
                                source: if source.is_empty() {
                                    vec![]
                                } else {
                                    self.f_cone.as_ref().either(
                                        |f_cone| f_cone.source()[source].to_vec(),
                                        |_| vec![self.g_cone.target().clone()],
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

/// Given a `Rewrite` A -f> B, find some `Rewrite`s A -p> C -q> B such that f = q ∘ p.
pub fn factorize2(rewrite: &Rewrite) -> Option<(Rewrite, Rewrite)> {
    let (ps, q) = factorize_sink(std::slice::from_ref(rewrite))?;
    Some((ps.into_iter().next().unwrap(), q))
}

pub(crate) fn factorize_sink(rewrites: &[Rewrite]) -> Option<(Vec<Rewrite>, Rewrite)> {
    // Check all rewrites have the same dimension.
    let dimension = rewrites
        .iter()
        .map(Rewrite::dimension)
        .all_equal_value()
        .ok()?;

    // Base case: all rewrites are equal.
    if dimension == 0 {
        let target = match rewrites
            .iter()
            .filter_map(|r| {
                let r: &Rewrite0 = r.try_into().unwrap();
                r.target()
            })
            .all_equal_value()
        {
            Ok(t) => t,
            Err(None) => {
                // All identities
                return Some((
                    vec![Rewrite::identity(0); rewrites.len()],
                    Rewrite::identity(0),
                ));
            }
            Err(Some(_)) => {
                panic!("Mismatched targets")
            }
        };

        let sources = rewrites
            .iter()
            .map(|r| {
                let r: &Rewrite0 = r.try_into().unwrap();
                r.source().unwrap_or(target)
            })
            .collect::<Vec<_>>();

        let max_sources = sources.iter().max_set_by_key(|s| s.generator.dimension);
        let join = if max_sources.len() == 1 {
            *max_sources[0]
        } else {
            target
        };

        return Some((
            sources
                .into_iter()
                .map(|source| Rewrite0::new(source, join, None).into())
                .collect(),
            Rewrite0::new(join, target, None).into(),
        ));
    }

    let mut p_cones = vec![vec![]; rewrites.len()];
    let mut q_cones = vec![];

    for (i, cones_over_target) in cones(rewrites.iter().map(|r| r.try_into().unwrap())) {
        // Check all cones have the same target.
        let target_cospan = cones_over_target
            .iter()
            .filter_map(|cone| cone.left().map(Cone::target))
            .all_equal_value()
            .ok()?;

        // Recursively factorise the cone slices.
        let (mut ps, q) = factorize_sink(
            &cones_over_target
                .iter()
                .flat_map(|cone| match cone.left() {
                    None => ZeroOneMany::One(Rewrite::identity(dimension - 1)),
                    Some(c) => ZeroOneMany::Many(c.singular_slices().iter().cloned()),
                })
                .collect::<Vec<_>>(),
        )?;

        let middle_cospan = Cospan {
            forward: factorize(&target_cospan.forward, &q).next()?,
            backward: factorize(&target_cospan.backward, &q).next()?,
        };

        for (cone, p_cones) in cones_over_target.iter().zip(p_cones.iter_mut()) {
            let (index, source, len) = match *cone {
                Either::Left(c) => (c.index, c.source().to_vec(), c.len()),
                Either::Right(index) => (index, vec![target_cospan.clone()], 1),
            };

            p_cones.push(Cone::new_unlabelled(
                index,
                source,
                middle_cospan.clone(),
                ps.drain(..len).collect(),
            ));
        }

        q_cones.push(Cone::new_unlabelled(
            i,
            vec![middle_cospan],
            target_cospan.clone(),
            vec![q],
        ));
    }

    Some((
        p_cones
            .into_iter()
            .map(|p_cones| RewriteN::new(dimension, p_cones).into())
            .collect(),
        RewriteN::new(dimension, q_cones).into(),
    ))
}

// Given a family of rewrites with a common target, iterate over their cones in lockstep.
fn cones<'a>(
    rewrites: impl IntoIterator<Item = &'a RewriteN>,
) -> impl Iterator<Item = (usize, Vec<Either<&'a Cone, usize>>)> {
    // For each rewrite, we record a reference to its cones and an offset for calculating the targets.
    let mut state: Vec<_> = rewrites.into_iter().map(|r| (r.cones(), 0)).collect();

    std::iter::from_fn(move || {
        let calculate_target = |c: &Cone, offset: isize| (c.index as isize + offset) as usize;

        // Find the next target or return None if there are no more cones left.
        let target = state
            .iter()
            .filter_map(|(cs, offset)| cs.first().map(|c| calculate_target(c, *offset)))
            .min()?;

        // For each rewrite, pop the first cone (if any) if its target is minimal.
        Some((
            target,
            state
                .iter_mut()
                .map(|(cs, offset)| match cs.first() {
                    Some(c) if calculate_target(c, *offset) == target => {
                        *cs = &cs[1..];
                        *offset += 1 - c.len() as isize;
                        Either::Left(c)
                    }
                    _ => {
                        let index = (target as isize - *offset) as usize;
                        Either::Right(index)
                    }
                })
                .collect(),
        ))
    })
}

// Same as `cones` but returns arrays instead of vectors.
fn array_cones<const N: usize>(
    rewrites: [&RewriteN; N],
) -> impl Iterator<Item = (usize, [Either<&Cone, usize>; N])> {
    cones(rewrites).map(|(i, cones)| (i, cones.try_into().unwrap()))
}
