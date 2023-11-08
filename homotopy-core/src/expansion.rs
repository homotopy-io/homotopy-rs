use std::{
    cmp::Ordering,
    convert::{Into, TryInto},
};

use thiserror::Error;

use crate::{
    attach::attach,
    common::{
        Boundary, BoundaryPath, DimensionError, Direction, Height, RegularHeight, SingularHeight,
    },
    diagram::DiagramN,
    factorization::factorize,
    rewrite::{Cone, Cospan, Rewrite, RewriteN},
    signature::Signature,
    typecheck::{typecheck_cospan, TypeError},
};

#[derive(Debug, Error)]
pub enum ExpansionError {
    #[error("can't perform expansion at this location")]
    WrongLocation,

    #[error("can't perform expansion in a regular slice")]
    RegularSlice,

    #[error("location is outside of the diagram")]
    OutOfBounds,

    #[error("can't expand a single component")]
    SingleComponent,

    #[error("no component to expand")]
    NoComponent,

    #[error("singular height is not smoothable")]
    Unsmoothable,

    #[error("expansion failed to propagate")]
    FailedToPropagate,

    #[error("expansion is ill-typed")]
    IllTyped(#[from] TypeError),

    #[error(transparent)]
    Dimension(#[from] DimensionError),
}

impl DiagramN {
    pub fn expand(
        &self,
        boundary_path: BoundaryPath,
        interior_path: &mut [Height],
        point: [Height; 2],
        direction: Direction,
        signature: &impl Signature,
    ) -> Result<Self, ExpansionError> {
        attach(self, boundary_path, |slice| {
            let slice = slice.try_into()?;
            let expand = expand_in_path(&slice, interior_path, point, direction)?;
            let identity = Rewrite::identity(slice.dimension());
            let cospan = Cospan {
                forward: identity,
                backward: expand.into(),
            };

            typecheck_cospan(slice.into(), cospan.clone(), signature)?;

            let cospan = match boundary_path.boundary() {
                Boundary::Source => cospan.flip(),
                Boundary::Target => cospan,
            };

            Ok(vec![cospan])
        })
    }
}

pub fn expand_in_path(
    diagram: &DiagramN,
    path: &mut [Height],
    point: [Height; 2],
    direction: Direction,
) -> Result<RewriteN, ExpansionError> {
    use Height::{Regular, Singular};

    match path.split_first_mut() {
        None => match point {
            [Regular(h0), Regular(h1)] => expand_base_regular(diagram, h0, h1, direction),
            [Singular(h0), Singular(h1)] => expand_base_singular(diagram, h0, h1, direction),
            _ => Err(ExpansionError::WrongLocation),
        },
        Some((step, rest)) => {
            let slice: DiagramN = diagram
                .slice(*step)
                .ok_or(ExpansionError::OutOfBounds)?
                .try_into()?;
            let recursive = expand_in_path(&slice, rest, point, direction)?;
            expand_propagate(diagram, step, recursive.into(), true)
        }
    }
}

fn expand_base_regular(
    diagram: &DiagramN,
    h0: RegularHeight,
    h1: RegularHeight,
    direction: Direction,
) -> Result<RewriteN, ExpansionError> {
    if h0 > diagram.size()
        || (h0 == 0 && direction == Direction::Backward)
        || (h0 == diagram.size() && direction == Direction::Forward)
    {
        return Err(ExpansionError::OutOfBounds);
    }

    let i = match direction {
        Direction::Forward => h0,
        Direction::Backward => h0 - 1,
    };

    let cospan = &diagram.cospans()[i];
    let forward: &RewriteN = (&cospan.forward).try_into()?;
    let backward: &RewriteN = (&cospan.backward).try_into()?;

    let j = {
        let preimage = match direction {
            Direction::Forward => forward.regular_preimage(h1),
            Direction::Backward => backward.regular_preimage(h1),
        };
        if matches!(preimage.len(), 0 | 2) {
            preimage.start
        } else {
            return Err(ExpansionError::Unsmoothable);
        }
    };

    let mut s_cones = vec![];
    let mut f_cones = forward.cones().to_vec();
    let mut b_cones = backward.cones().to_vec();

    let f_cone_index = cone_over_target(&f_cones, j).ok_or(ExpansionError::Unsmoothable)?;
    let b_cone_index = cone_over_target(&b_cones, j).ok_or(ExpansionError::Unsmoothable)?;

    if f_cones[f_cone_index].internal == b_cones[b_cone_index].internal
        && f_cones[f_cone_index].is_homotopy()
    {
        let f_cone = f_cones.remove(f_cone_index);
        b_cones.remove(b_cone_index);
        s_cones.push(Cone {
            index: j,
            internal: f_cone.internal,
        });
    } else {
        return Err(ExpansionError::Unsmoothable);
    }

    let smooth = RewriteN::new(forward.dimension(), s_cones).into();
    let smooth_cospan = Cospan {
        forward: RewriteN::new(forward.dimension(), f_cones).into(),
        backward: RewriteN::new(backward.dimension(), b_cones).into(),
    };

    let cone = if smooth_cospan.is_identity() {
        // Decrease diagram height by 1.
        Cone::new_unit(i, cospan.clone(), smooth)
    } else {
        // Keep diagram height the same.
        Cone::new(
            i,
            vec![smooth_cospan],
            cospan.clone(),
            vec![cospan.forward.clone(), cospan.backward.clone()],
            vec![smooth],
        )
    };

    Ok(RewriteN::new(diagram.dimension(), vec![cone]))
}

// This is the same as `Rewrite::cone_over_target` but returns an index instead of a cone.
fn cone_over_target(cones: &[Cone], height: SingularHeight) -> Option<usize> {
    let mut offset: isize = 0;

    for (i, cone) in cones.iter().enumerate() {
        let target = (cone.index as isize + offset) as usize;

        if target == height {
            return Some(i);
        }

        offset += 1 - cone.len() as isize;
    }

    None
}

fn expand_base_singular(
    diagram: &DiagramN,
    h0: SingularHeight,
    h1: SingularHeight,
    direction: Direction,
) -> Result<RewriteN, ExpansionError> {
    if h0 >= diagram.size() {
        return Err(ExpansionError::OutOfBounds);
    }

    let cospan = &diagram.cospans()[h0];
    let forward: &RewriteN = (&cospan.forward).try_into()?;
    let backward: &RewriteN = (&cospan.backward).try_into()?;

    match direction {
        Direction::Forward => {
            let expansion = expand_cospan(h1, forward, backward)?;

            let cone = Cone::new(
                h0,
                vec![
                    Cospan {
                        forward: expansion.rewrites[0].clone().into(),
                        backward: expansion.rewrites[1].clone().into(),
                    },
                    Cospan {
                        forward: expansion.rewrites[2].clone().into(),
                        backward: expansion.rewrites[3].clone().into(),
                    },
                ],
                cospan.clone(),
                vec![
                    cospan.forward.clone(),
                    expansion.regular_slice.into(),
                    cospan.backward.clone(),
                ],
                vec![
                    expansion.singular_slices[0].clone().into(),
                    expansion.singular_slices[1].clone().into(),
                ],
            );

            Ok(RewriteN::new(diagram.dimension(), vec![cone]))
        }
        Direction::Backward => {
            let expansion = expand_cospan(h1, backward, forward)?;

            let cone = Cone::new(
                h0,
                vec![
                    Cospan {
                        forward: expansion.rewrites[3].clone().into(),
                        backward: expansion.rewrites[2].clone().into(),
                    },
                    Cospan {
                        forward: expansion.rewrites[1].clone().into(),
                        backward: expansion.rewrites[0].clone().into(),
                    },
                ],
                cospan.clone(),
                vec![
                    cospan.forward.clone(),
                    expansion.regular_slice.into(),
                    cospan.backward.clone(),
                ],
                vec![
                    expansion.singular_slices[1].clone().into(),
                    expansion.singular_slices[0].clone().into(),
                ],
            );

            Ok(RewriteN::new(diagram.dimension(), vec![cone]))
        }
    }
}

/// rewrites forms 2 cospans
/// singular_slices are the singular slices of the expanding rewrite from the expansion to the
/// original diagram
/// regular_slice is the middle regular slice of the expanding rewrite from the expansion to the
/// original diagram (the other two are given by input data)
struct ExpandedCospan {
    rewrites: [RewriteN; 4],
    regular_slice: RewriteN,
    singular_slices: [RewriteN; 2],
}

fn expand_cospan(
    height: SingularHeight,
    forward: &RewriteN,
    backward: &RewriteN,
) -> Result<ExpandedCospan, ExpansionError> {
    debug_assert_eq!(forward.dimension(), backward.dimension());
    let forward_targets: Vec<_> = forward.targets().collect();
    let backward_targets: Vec<_> = backward.targets().collect();

    let forward_index = forward_targets.iter().position(|t| *t == height);
    let backward_index = backward_targets.iter().position(|t| *t == height);

    if forward_index.is_none() && backward_index.is_none() {
        return Err(ExpansionError::NoComponent);
    }

    if (forward_targets.len() + backward_targets.len() == 1)
        || (forward_targets.len() == 1
            && backward_targets.len() == 1
            && forward_index.is_some()
            && backward_index.is_some())
    {
        return Err(ExpansionError::SingleComponent);
    }

    let forward_delta: isize = forward
        .cones()
        .iter()
        .enumerate()
        .take_while(|(i, _)| forward_targets[*i] < height)
        .map(|(_, c)| c.len() as isize - 1)
        .sum();

    let backward_delta: isize = backward
        .cones()
        .iter()
        .enumerate()
        .take_while(|(i, _)| backward_targets[*i] < height)
        .map(|(_, c)| c.len() as isize - 1)
        .sum();

    let forward_offset = forward_index.map_or(0, |i| forward.cones()[i].len() as isize - 1);
    let backward_offset = backward_index.map_or(0, |i| backward.cones()[i].len() as isize - 1);

    let new_forward0 = match forward_index {
        None => forward.clone(),
        Some(index) => {
            let mut cones = forward.cones().to_vec();
            cones.remove(index);
            RewriteN::new(forward.dimension(), cones)
        }
    };

    let new_backward0 = RewriteN::new(
        backward.dimension(),
        backward
            .cones()
            .iter()
            .enumerate()
            .filter_map(|(i, c)| match backward_targets[i].cmp(&height) {
                Ordering::Greater => {
                    let mut c = c.clone();
                    c.index = (c.index as isize + forward_offset - backward_offset) as usize;
                    Some(c)
                }
                Ordering::Less => Some(c.clone()),
                Ordering::Equal => None,
            })
            .collect(),
    );

    let new_forward1 = match forward_index {
        None => RewriteN::identity(forward.dimension()),
        Some(index) => {
            let mut cone = forward.cones()[index].clone();
            cone.index = (cone.index as isize - forward_delta + backward_delta) as usize;
            RewriteN::new(forward.dimension(), vec![cone])
        }
    };

    let new_backward1 = match backward_index {
        None => RewriteN::identity(backward.dimension()),
        Some(index) => RewriteN::new(backward.dimension(), vec![backward.cones()[index].clone()]),
    };

    let new_singular_slice0 = match forward_index {
        None => RewriteN::identity(forward.dimension()),
        Some(index) => {
            let mut cone = forward.cones()[index].clone();
            cone.index = (cone.index as isize - forward_delta) as usize;
            RewriteN::new(forward.dimension(), vec![cone])
        }
    };

    let new_singular_slice1 = match backward_index {
        None => backward.clone(),
        Some(index) => RewriteN::new(
            backward.dimension(),
            backward
                .cones()
                .iter()
                .enumerate()
                .filter_map(|(i, c)| match i.cmp(&index) {
                    Ordering::Greater => {
                        let mut c = c.clone();
                        c.index = (c.index as isize - backward_offset) as usize;
                        Some(c)
                    }
                    Ordering::Equal => None,
                    Ordering::Less => Some(c.clone()),
                })
                .collect(),
        ),
    };

    let new_regular_slice = {
        let mut cones = new_backward0.cones().to_vec();
        if let Some(cone) = new_forward1.cones().get(0) {
            debug_assert!(forward_index.is_some());
            let index = backward_index.unwrap_or_else(|| {
                backward_targets
                    .iter()
                    .rev()
                    .position(|t| *t > height)
                    .unwrap_or(backward_targets.len())
            });
            cones.insert(index, cone.clone());
        }
        RewriteN::new(forward.dimension(), cones)
    };

    Ok(ExpandedCospan {
        rewrites: [new_forward0, new_backward0, new_forward1, new_backward1],
        regular_slice: new_regular_slice,
        singular_slices: [new_singular_slice0, new_singular_slice1],
    })
}

/// Propagate a expansion on a slice to the whole diagram.
pub(crate) fn expand_propagate(
    diagram: &DiagramN,
    height: &mut Height,
    expansion: Rewrite,
    normalize: bool,
) -> Result<RewriteN, ExpansionError> {
    let i = match *height {
        Height::Regular(_) => return Err(ExpansionError::RegularSlice),
        Height::Singular(i) => i,
    };

    let target_cospan = diagram
        .cospans()
        .get(i)
        .ok_or(ExpansionError::OutOfBounds)?;

    let forward = factorize(&target_cospan.forward, &expansion).next();
    let backward = factorize(&target_cospan.backward, &expansion).next();

    #[allow(clippy::single_match_else)]
    let cone = match (forward, backward) {
        (Some(forward), Some(backward)) => {
            let source_cospan = Cospan { forward, backward };
            if normalize && source_cospan.is_redundant() {
                *height = Height::Regular(i);
                Some(Cone::new_unit(
                    i,
                    target_cospan.clone(),
                    target_cospan.forward.clone(),
                ))
            } else {
                Some(Cone::new(
                    i,
                    vec![source_cospan],
                    target_cospan.clone(),
                    vec![
                        target_cospan.forward.clone(),
                        target_cospan.backward.clone(),
                    ],
                    vec![expansion],
                ))
            }
        }
        // (Some(forward), None) => {
        //     let slice = diagram.slice(Height::Singular(i)).unwrap();
        //     let (backward, inclusion) = factorize2(&target_cospan.backward).unwrap();
        //     let (_, inner_backward, inner_forward) = antipushout(
        //         &slice.clone().rewrite_backward(&expansion).unwrap(),
        //         &slice.clone().rewrite_backward(&inclusion).unwrap(),
        //         &slice,
        //         &expansion,
        //         &inclusion,
        //     )[0]
        //     .clone();

        //     Some(Cone::new_unlabelled(
        //         i,
        //         vec![
        //             Cospan {
        //                 forward,
        //                 backward: inner_backward,
        //             },
        //             Cospan {
        //                 forward: inner_forward,
        //                 backward,
        //             },
        //         ],
        //         target_cospan.clone(),
        //         vec![expansion, inclusion],
        //     ))
        // }
        // (None, Some(backward)) => {
        //     let slice = diagram.slice(Height::Singular(i)).unwrap();
        //     let (forward, inclusion) = factorize2(&target_cospan.forward).unwrap();
        //     let (_, inner_backward, inner_forward) = antipushout(
        //         &slice.clone().rewrite_backward(&inclusion).unwrap(),
        //         &slice.clone().rewrite_backward(&expansion).unwrap(),
        //         &slice,
        //         &inclusion,
        //         &expansion,
        //     )[0]
        //     .clone();

        //     Some(Cone::new_unlabelled(
        //         i,
        //         vec![
        //             Cospan {
        //                 forward,
        //                 backward: inner_backward,
        //             },
        //             Cospan {
        //                 forward: inner_forward,
        //                 backward,
        //             },
        //         ],
        //         target_cospan.clone(),
        //         vec![inclusion, expansion],
        //     ))
        // }
        _ => {
            let source_cospans = vec![
                Cospan {
                    forward: target_cospan.forward.clone(),
                    backward: expansion.clone(),
                },
                Cospan {
                    forward: expansion.clone(),
                    backward: target_cospan.backward.clone(),
                },
            ];
            if source_cospans[0].is_redundant() || source_cospans[1].is_redundant() {
                // Identity
                None
            } else if target_cospan.forward.is_homotopy() && target_cospan.backward.is_homotopy() {
                // Insert a bubble
                *height = Height::Regular(i + 1);
                Some(Cone::new(
                    i,
                    source_cospans,
                    target_cospan.clone(),
                    vec![
                        target_cospan.forward.clone(),
                        expansion,
                        target_cospan.backward.clone(),
                    ],
                    vec![Rewrite::identity(diagram.dimension() - 1); 2],
                ))
            } else {
                return Err(ExpansionError::FailedToPropagate);
            }
        }
    };

    Ok(RewriteN::new(
        diagram.dimension(),
        cone.into_iter().collect(),
    ))
}

impl Cone {
    fn is_homotopy(&self) -> bool {
        self.target().max_generator().map_or(true, |d| {
            d.generator.dimension <= self.target().forward.dimension() + 1
        })
    }
}
