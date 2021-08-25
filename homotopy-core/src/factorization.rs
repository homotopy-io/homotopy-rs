use std::ops::Range;

use crate::common::Mode;
use crate::monotone::MonotoneIterator;
use crate::Rewrite;
use crate::{Diagram, Rewrite0};
use crate::{Height, RewriteN};
use thiserror::Error;

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
            MonotoneIterator::new(false, &constraints)
                .find_map(|h_mono| {
                    // Recurse on each monotone component
                    let mut cone_slices: Vec<Vec<Rewrite>> = vec![vec![]; t.size()];

                    for (si, ti) in h_mono.into_iter().enumerate() {
                        let sub_s = s.slice(Height::Singular(si))?;
                        let sub_t = t.slice(Height::Singular(ti))?;
                        let slice = factorize(fr.slice(si), gr.slice(ti), sub_s, sub_t).ok()?;
                        cone_slices[ti].push(slice);
                    }

                    let hr = RewriteN::from_slices(
                        fr.dimension(),
                        s.cospans(),
                        t.cospans(),
                        cone_slices,
                    );

                    // TODO(calintat): Think about removing this.
                    if hr.check_well_formed(Mode::Shallow).is_ok() {
                        Some(hr.into())
                    } else {
                        None
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
