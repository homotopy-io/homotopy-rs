use std::{cmp, ops::Range};

use crate::Rewrite;
use crate::{Diagram, Rewrite0};
use crate::{Height, RewriteN};
use thiserror::Error;

/// Given a constraints vector of length $`n`$, this iterator generates all monotone
/// (non-decreasing) sequences of $`n`$ digits, where each digit satisfies its corresponding
/// constraint. Each constraint is a `Range<usize>`, a half-open range inclusive below and
/// exclusive above, specifying the values which the digit with the same index in the output
/// monotone sequence may take. The order of outputs is lexicographic.
// this iterator is cyclic, *not* fused
#[derive(Debug, Clone)]
pub struct MonotoneSequences {
    cur: Option<Vec<usize>>,

    // invariant: ∀ x ∈ cur[end, len). x maxxed within its range
    end: usize,
    constraints: Vec<Range<usize>>, // digit-wise range constraints
}

#[allow(clippy::len_without_is_empty)]
impl MonotoneSequences {
    pub fn new(constraints: Vec<Range<usize>>) -> Self {
        Self {
            cur: None,
            end: constraints.len(),
            constraints,
        }
    }

    /// The number of digits in the monotone sequence.
    pub fn len(&self) -> usize {
        self.constraints.len()
    }
}

impl Iterator for MonotoneSequences {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.len();
        match &mut self.cur {
            None => {
                // first monotone sequence is range.start for each digit
                let mut min = Default::default();
                let mut first = Vec::with_capacity(self.constraints.len());
                for i in 0..self.constraints.len() {
                    min = cmp::max(min, self.constraints[i].start);
                    if min < self.constraints[i].end {
                        first.push(min);
                    } else {
                        return None;
                    }
                }

                // maintain invariant
                self.end = first.len();
                while self.end > 0 && first[self.end - 1] == self.constraints[self.end - 1].end - 1
                {
                    self.end -= 1;
                }

                self.cur = first.into();
            }
            Some(seq) => {
                if self.end == 0 {
                    self.cur = None;
                } else {
                    // increment last non-max digit
                    let l = self.end - 1;
                    seq[l] = cmp::min(seq[l] + 1, self.constraints[l].end - 1);

                    if seq[l] == self.constraints[l].end - 1 {
                        // maxxed seq[l] within its range
                        self.end -= 1; // maintain invariant
                    } else {
                        for i in (self.end)..len {
                            // preserve monotonicity for all values to the right
                            seq[i] = cmp::max(seq[l], self.constraints[i].start);
                            debug_assert!(seq[i] >= seq[l]);
                        }

                        // maintain invariant
                        self.end = len;
                        while seq[self.end - 1] == self.constraints[self.end - 1].end - 1 {
                            self.end -= 1;
                        }
                    }
                }
            }
        }
        self.cur.clone()
    }
}

/// Given `Rewrite`s A -f> C <g- B, find some `Rewrite` A -h> B which factorises f = g ∘ h
// modulo trivial cases, this works by guessing a monotone function to underly h, and then recurse
// down dimensions (as in the 0-dimensional case, the only of the rewrite is the monotone function)
pub fn factorize(
    f: Rewrite,
    g: Rewrite,
    source: Diagram,
    target: Diagram,
) -> Result<Rewrite, FactorizationError> {
    // Simple special cases
    if g.is_identity() {
        return Ok(f);
    }

    if f == g {
        return Ok(Rewrite::identity(f.dimension()));
    }

    // General cases
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
            if t.size() == 0 && s.size() > 0 {
                return Err(FactorizationError::Function);
            }

            let f_height = fr.singular_image(s.size());
            let g_height = gr.singular_image(t.size());

            if g_height < f_height {
                return Err(FactorizationError::Codomain);
            }

            // obtain an iterator over all possible monotone sequences which may underly h
            // ( as f(a) = g(h(a)) ⟹ h(a) ∈ g⁻¹(f(a)) )
            let constraints: Vec<Range<usize>> = (0..s.size())
                .map(|i| gr.singular_preimage(fr.singular_image(i)))
                .collect();

            // find a particular monotone sequence which works
            MonotoneSequences::new(constraints)
                .find_map(|h_mono| {
                    // Recurse on each monotone component
                    let mut cone_slices: Vec<Vec<Rewrite>> = vec![vec![]; t.size()];

                    for (si, ti) in h_mono.iter().enumerate() {
                        let sub_s = s.slice(Height::Singular(si))?;
                        let sub_t = t.slice(Height::Singular(*ti))?;
                        let slice = factorize(fr.slice(si), gr.slice(*ti), sub_s, sub_t).ok()?;
                        cone_slices[*ti].push(slice);
                    }

                    Some(
                        RewriteN::from_slices(
                            fr.dimension(),
                            s.cospans(),
                            t.cospans(),
                            cone_slices,
                        )
                        .into(),
                    )
                })
                .ok_or(FactorizationError::Failed)
        }

        // ideally, we would check for matching codomains in the n-rewrite
        // case also, but this requires threading A through the function
        (Rewrite::Rewrite0(_), Rewrite::Rewrite0(_), _, _) => Err(FactorizationError::Codomain),

        (x, y, _, _) => Err(FactorizationError::Dimension(x.dimension(), y.dimension())),
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

    #[error("codomain is empty, but domain is nonempty, so no function exists")]
    Function,

    #[error("failed to factorize")]
    Failed,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn monotone_sequences() {
        let ms_0_1_2 = MonotoneSequences::new(vec![0..2, 0..2]);
        assert_eq!(ms_0_1_2.collect::<Vec<_>>(), [[0, 0], [0, 1], [1, 1]]);
        let ms_0_3_3 = MonotoneSequences::new(vec![0..4, 0..4, 0..4]);
        assert_eq!(
            ms_0_3_3.collect::<Vec<_>>(),
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
        let ms_1_3_3 = MonotoneSequences::new(vec![1..4, 0..4, 1..4]);
        assert_eq!(
            ms_1_3_3.collect::<Vec<_>>(),
            [
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
        // unsatisfiable constraints
        let invalid_ms = MonotoneSequences::new(vec![1..2, 0..1]);
        assert!(invalid_ms.collect::<Vec<_>>().is_empty());
        // iterator should be cyclic
        let mut ms_0_0_1 = MonotoneSequences::new(vec![0..1]);
        dbg!(ms_0_0_1.clone().collect::<Vec<_>>());
        assert_eq!(ms_0_0_1.next(), Some(vec![0]));
        assert_eq!(ms_0_0_1.next(), None);
        assert_eq!(ms_0_0_1.next(), Some(vec![0]));
    }

    // TODO: test factorize
}
