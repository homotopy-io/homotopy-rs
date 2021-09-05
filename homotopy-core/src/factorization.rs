use std::ops::Range;

use crate::monotone::{Monotone, MonotoneIterator};
use crate::Height;
use crate::{Diagram, DiagramN};
use crate::{Rewrite, Rewrite0, RewriteN};

use itertools::Itertools;
use itertools::MultiProduct;

/// Given `Rewrite`s A -f> C <g- B, find some `Rewrite` A -h> B which factorises f = g ∘ h.
pub fn factorize(f: Rewrite, g: Rewrite, source: Diagram, target: Diagram) -> Factorization {
    match (f, g, source, target) {
        (
            Rewrite::Rewrite0(f),
            Rewrite::Rewrite0(g),
            Diagram::Diagram0(s),
            Diagram::Diagram0(t),
        ) => {
            assert!(f.source() == None || f.source() == Some(s));
            assert!(g.source() == None || g.source() == Some(t));

            if s.dimension > t.dimension {
                Factorization::Factorization0(None)
            } else {
                Factorization::Factorization0(Some(Rewrite::from(Rewrite0::new(s, t))))
            }
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

            let constraints: Vec<Range<usize>> = (0..source.size())
                .map(|i| g.singular_preimage(f.singular_image(i)))
                .collect();

            Factorization::FactorizationN(FactorizationInternal {
                f,
                g,
                source,
                target,
                monotone: MonotoneIterator::new(false, &constraints),
                cur: None,
            })
        }
        _ => panic!("Mismatched dimensions"),
    }
}

#[derive(Clone)]
pub enum Factorization {
    Factorization0(Option<Rewrite>),
    FactorizationN(FactorizationInternal),
}

impl Iterator for Factorization {
    type Item = Rewrite;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Factorization0(h) => h.take(),
            Self::FactorizationN(internal) => internal.next(),
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
                self.next()
            }
            Some((h_mono, product)) => match product.next() {
                None => {
                    self.cur = None;
                    self.next()
                }
                Some(slices) => {
                    let mut cone_slices: Vec<Vec<Rewrite>> = vec![vec![]; self.target.size()];
                    for (i, &j) in h_mono.iter().enumerate() {
                        cone_slices[j].push(slices[i].clone());
                    }
                    let h = RewriteN::from_slices_safe(
                        self.f.dimension(),
                        self.source.cospans(),
                        self.target.cospans(),
                        cone_slices,
                    );
                    h.map_or_else(|_| self.next(), |h| Some(Rewrite::from(h)))
                }
            },
        }
    }
}
