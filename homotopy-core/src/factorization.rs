use std::ops::Range;

use itertools::{Itertools, MultiProduct};

use crate::{
    common::Mode,
    monotone::{Monotone, MonotoneIterator},
    Diagram, DiagramN, Height, Rewrite, Rewrite0, RewriteN,
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
            assert!(f.source().is_none() || f.source() == Some(s));
            assert!(g.source().is_none() || g.source() == Some(t));

            Factorization::Unique(
                (s == t || s.dimension < t.dimension).then(|| Rewrite0::new(s, t, todo!()).into()),
            )
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

            if source.size() == 0 {
                // Try to construct the trivial rewrite.
                let h = RewriteN::from_slices_unsafe(
                    f.dimension(),
                    source.cospans(),
                    target.cospans(),
                    todo!(),
                    vec![vec![]; target.size()],
                );
                Factorization::Unique(h.check(Mode::Shallow).is_ok().then(|| Rewrite::from(h)))
            } else {
                let constraints: Vec<Range<usize>> = (0..source.size())
                    .map(|i| g.singular_preimage(f.singular_image(i)))
                    .collect();

                Factorization::Iterator(FactorizationInternal {
                    f,
                    g,
                    source,
                    target,
                    monotone: MonotoneIterator::new(false, &constraints),
                    cur: None,
                })
            }
        }
        _ => panic!("Mismatched dimensions"),
    }
}

#[derive(Clone)]
pub enum Factorization {
    Unique(Option<Rewrite>),
    Iterator(FactorizationInternal),
}

impl Iterator for Factorization {
    type Item = Rewrite;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Unique(h) => h.take(),
            Self::Iterator(internal) => internal.next(),
        }
    }
}

#[derive(Clone)]
pub struct FactorizationInternal {
    f: RewriteN,
    g: RewriteN,
    source: DiagramN,
    target: DiagramN,
    monotone: MonotoneIterator,
    cur: Option<(Monotone, MultiProduct<Factorization>)>,
}

impl Iterator for FactorizationInternal {
    type Item = Rewrite;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.cur {
                None => {
                    let h_mono = self.monotone.next()?;
                    let product = h_mono
                        .iter()
                        .enumerate()
                        .map(|(si, &ti)| {
                            factorize(
                                self.f.slice(si),
                                self.g.slice(ti),
                                self.source.slice(Height::Singular(si)).unwrap(),
                                self.target.slice(Height::Singular(ti)).unwrap(),
                            )
                        })
                        .multi_cartesian_product();
                    self.cur = Some((h_mono, product));
                }
                Some((h_mono, product)) => match product.next() {
                    None => {
                        self.cur = None;
                    }
                    Some(slices) => {
                        let h = RewriteN::from_monotone_unsafe(
                            self.f.dimension(),
                            self.source.cospans(),
                            self.target.cospans(),
                            h_mono,
                            todo!(),
                            &slices,
                        );
                        if h.check(Mode::Shallow).is_ok() {
                            return Some(Rewrite::from(h));
                        }
                    }
                },
            }
        }
    }
}
