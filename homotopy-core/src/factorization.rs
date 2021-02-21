use std::collections::HashSet;

use crate::{Cospan, Rewrite};
use crate::{Diagram, Rewrite0};
use crate::{Height, RewriteN};
use thiserror::Error;

pub struct MonotoneSequences {
    cur: Option<Vec<usize>>,

    // invariant: ∀ x ∈ cur(end, len). x = max
    end: usize,

    pub len: usize,
    pub max: usize,
}

impl MonotoneSequences {
    pub fn new(len: usize, max: usize) -> MonotoneSequences {
        MonotoneSequences {
            cur: None,
            end: len - 1,
            len,
            max,
        }
    }
}

impl Iterator for MonotoneSequences {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.cur {
            None => {
                // first monotone sequence is all 0s
                self.cur = Some([0].repeat(self.len));
            }
            Some(seq) => {
                if seq != &[self.max].repeat(self.len) {
                    seq[self.end] += 1; // increment last non-max digit
                    if seq[self.end] == self.max {
                        self.end = self.end.saturating_sub(1) // maintain invariant
                    } else {
                        for i in (self.end + 1)..self.len {
                            // set all values to the right to it
                            seq[i] = seq[self.end]
                        }
                        self.end = self.len - 1 // maintain invariant
                    }
                } else {
                    self.cur = None
                }
            }
        }
        self.cur.clone()
    }
}

/// Given `Rewrite`s A -f> C <g- B, find some `Rewrite` A -h> B which factorises f = g ∘ h
// modulo trivial cases, this works by guessing a monotone function to underly h, and then recurse
// down dimensions (as in the 0-dimensional case, the only of the rewrite is the monotone function)
fn factorize(
    f: Rewrite,
    g: Rewrite,
    source: Diagram,
    target: Diagram,
) -> Result<Rewrite, FactorizationError> {
    if g.is_identity() {
        Ok(f)
    } else if f == g {
        Ok(Rewrite::identity(f.dimension()))
    } else {
        match (f, g, source, target) {
            (
                Rewrite::Rewrite0(Rewrite0(Some((fs, ft)))),
                Rewrite::Rewrite0(Rewrite0(Some((gs, gt)))),
                Diagram::Diagram0(s),
                Diagram::Diagram0(t),
            ) if fs == s && ft == gt && gs == t => Ok(Rewrite::from(Rewrite0(Some((fs, gs))))),
            (
                Rewrite::RewriteN(fr),
                Rewrite::RewriteN(gr),
                Diagram::DiagramN(s),
                Diagram::DiagramN(t),
            ) if fr.dimension() == gr.dimension() => {
                // get the singular levels in the source of r which aren't tips of identity spans
                let sources = |r: &RewriteN| {
                    let mut sources = HashSet::new();
                    let mut offset = 0;
                    for cone in r.cones() {
                        sources.extend((cone.index..(cone.index + cone.len())).map(|i| i + offset));
                        offset += 1 - cone.len();
                    }
                    sources
                };
                let f_height = *fr.targets().iter().max().unwrap();
                let g_height = *gr.targets().iter().max().unwrap();
                if g_height < f_height {
                    return Err(FactorizationError::Codomain);
                }
                let f_mono: Vec<usize> = (0..f_height - 1).map(|i| fr.singular_image(i)).collect();
                let g_mono: Vec<usize> = (0..g_height - 1).map(|i| gr.singular_image(i)).collect();
                // iterator to guess a monotone function underlying h
                let mut mss = MonotoneSequences::new(
                    // number of singular levels of B
                    *sources(&gr).iter().max().unwrap(),
                    // number of singular levels of C
                    g_height,
                );
                mss.find_map(|ms| {
                    if
                    // if the monotone sequence hits a singular level which is
                    // the tip of an identity span, then we can skip it
                    !ms.iter().copied().collect::<HashSet<_>>().is_subset(&sources(&gr))

                    // check that this monotone composes with that of g to get that of f
                    || (0 .. f_height - 1).map(|i| g_mono[ms[i]]).collect::<Vec<_>>() != f_mono
                    {
                        None
                    } else {
                        // recurse on each monotone component
                        let mut cone_slices: Vec<Vec<Rewrite>> = Vec::new();
                        let mut sources: Vec<Cospan> = Vec::new();
                        let mut targets: Vec<Cospan> = Vec::new();
                        let mut cur: Option<Vec<Rewrite>> = None;
                        for (si, ti) in ms.iter().enumerate() {
                            let sub_s = s.slice(Height::Singular(si))?;
                            let sub_t = t.slice(Height::Singular(*ti))?;
                            let slice =
                                factorize(fr.slice(si), gr.slice(*ti), sub_s, sub_t).ok()?;
                            if !slice.is_identity() {
                                sources.push(s.cospans()[si].clone());
                                match &mut cur {
                                    Some(slices) => slices.push(slice),
                                    None => {
                                        cur = Some(vec![slice]);
                                        targets.push(t.cospans()[*ti].clone())
                                    }
                                }
                            } else {
                                if let Some(slices) = cur {
                                    cone_slices.push(slices)
                                }
                                cur = None
                            }
                        }
                        if let Some(slices) = cur {
                            cone_slices.push(slices)
                        }
                        Some(
                            RewriteN::from_slices(fr.dimension(), &sources, &targets, cone_slices)
                                .into(),
                        )
                    }
                })
                .ok_or(FactorizationError::Failed)
            }

            // ideally, we would check for matching codomains in the n-rewrite
            // case also, but this requires threading A through the function
            (Rewrite::Rewrite0(_), Rewrite::Rewrite0(_), _, _) => Err(FactorizationError::Codomain),

            (x, y, _, _) => Err(FactorizationError::Dimension(x.dimension(), y.dimension())),
        }
    }
}

#[derive(Debug, Error)]
pub enum FactorizationError {
    #[error("can't factorize rewrites of different dimensions {0} and {1}")]
    Dimension(usize, usize),

    #[error("rewrites have different codomains")]
    Codomain,

    #[error("singular level at height {0} is not in both images")]
    Image(usize),

    #[error("failed to factorize")]
    Failed,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn monotone_sequences() {
        let ms_2_1 = MonotoneSequences::new(2, 1);
        assert_eq!(ms_2_1.collect::<Vec<_>>(), [[0, 0], [0, 1], [1, 1]]);
        let ms_3_3 = MonotoneSequences::new(3, 3);
        assert_eq!(
            ms_3_3.collect::<Vec<_>>(),
            [
                [0, 0, 0],
                [0, 0, 1],
                [0, 0, 2],
                [0, 0, 3],
                [0, 1, 1],
                [0, 1, 2],
                [0, 1, 3],
                [0, 2, 2],
                [0, 2, 3],
                [0, 3, 3],
                [1, 1, 1],
                [1, 1, 2],
                [1, 1, 3],
                [1, 2, 2],
                [1, 2, 3],
                [1, 3, 3],
                [2, 2, 2],
                [2, 2, 3],
                [2, 3, 3],
                [3, 3, 3]
            ]
        );
    }

    // TODO: test factorize
}
