use std::{
    cmp::Ordering,
    convert::{Into, TryInto},
};

use thiserror::Error;

use crate::{
    antipushout::{antipushout, factorize_inc},
    attach::{attach, BoundaryPath},
    common::{Boundary, DimensionError, Direction, Height, Orientation, RegularHeight, SingularHeight},
    diagram::{Diagram, DiagramN},
    factorization::factorize,
    normalization::normalize_singular,
    rewrite::{Cone, Cospan, Rewrite, RewriteN},
    signature::Signature,
    typecheck::{typecheck_cospan, TypeError},
};

#[derive(Debug, Error)]
pub enum ExpansionError {
    #[error("expansion location must be at least 2-dimensional")]
    LocationTooShort,

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

    #[error("expansion is ill-typed: {0}")]
    IllTyped(#[from] TypeError),

    #[error("invalid boundary path provided to expansion")]
    Dimension(#[from] DimensionError),
}

impl DiagramN {
    pub fn expand<S>(
        &self,
        boundary_path: BoundaryPath,
        interior_path: &[Height],
        direction: Direction,
        signature: &S,
    ) -> Result<Self, ExpansionError>
    where
        S: Signature,
    {
        attach(self, boundary_path, |slice| {
            let expand: Rewrite = expand_in_path(&slice, interior_path, direction)?;
            let identity = Rewrite::identity(slice.dimension());
            let cospan = match boundary_path.boundary() {
                Boundary::Source => Cospan {
                    forward: expand,
                    backward: identity,
                },
                Boundary::Target => Cospan {
                    forward: identity,
                    backward: expand,
                },
            };

            // TODO
            // typecheck_cospan(slice, cospan.clone(), boundary_path.boundary(), signature)?;

            Ok(vec![cospan])
        })
    }
}

impl Cospan {
    /// Promotes a `Cospan` to one dimension higher by bubbling.
    #[must_use]
    pub fn bubble(&self) -> Self {
        use Orientation::Zero;
        let forward = RewriteN::new(
            self.forward.dimension() + 1,
            vec![Cone::new_untrimmed(
                0,
                Default::default(),
                Cospan {
                    forward: self.forward.clone().orientation_transform(Zero),
                    backward: self.forward.clone().orientation_transform(Zero),
                },
                vec![self.forward.clone().orientation_transform(Zero)],
                Default::default(),
            )],
        )
        .into();
        let backward = RewriteN::new(
            self.forward.dimension() + 1,
            vec![Cone::new_untrimmed(
                0,
                vec![self.clone(), self.inverse()],
                Cospan {
                    forward: self.forward.clone().orientation_transform(Zero),
                    backward: self.forward.clone().orientation_transform(Zero),
                },
                vec![self.backward.clone().orientation_transform(Zero)],
                vec![Rewrite::identity(self.forward.dimension()); 2],
            )],
        )
        .into();
        Self { forward, backward }
    }
}

pub fn expand_in_path(
    diagram: &Diagram,
    location: &[Height],
    direction: Direction,
) -> Result<Rewrite, ExpansionError> {
    use Height::{Regular, Singular};

    match location.split_first() {
        _ if diagram.dimension() < location.len() => Err(ExpansionError::OutOfBounds),
        None | Some((Singular(_), &[])) => Err(ExpansionError::LocationTooShort),
        Some((Regular(h0), &([] | [_]))) => expand_base_regular(diagram, *h0, direction),
        Some((Singular(h0), &[Singular(h1)])) => expand_base_singular(diagram, *h0, h1, direction),
        Some((Regular(_), _)) => Err(ExpansionError::RegularSlice),
        Some((Singular(height), rest)) => {
            let diagram = match diagram {
                Diagram::Diagram0(_) => Err(ExpansionError::OutOfBounds),
                Diagram::DiagramN(diagram) => Ok(diagram),
            }?;

            let slice = diagram
                .slice(Height::Singular(*height))
                .ok_or(ExpansionError::OutOfBounds)?;

            let recursive = expand_in_path(&slice, rest, direction)?;
            Ok(expand_propagate(diagram, *height, recursive)?.into())
        }
    }
}

/// Remove a redundant singular level where its incoming rewrites are identical.
/// This move is algebraically valid (if it typechecks).
fn expand_base_regular(
    diagram: &Diagram,
    h0: RegularHeight,
    direction: Direction,
) -> Result<Rewrite, ExpansionError> {
    let diagram = match diagram {
        Diagram::Diagram0(_) => Err(ExpansionError::OutOfBounds),
        Diagram::DiagramN(diagram) => Ok(diagram),
    }?;

    if (h0 == 0 && direction == Direction::Backward)
        || (h0 == diagram.size() && direction == Direction::Forward)
        || h0 > diagram.size()
    {
        return Err(ExpansionError::OutOfBounds);
    }

    let i = h0
        - match direction {
            Direction::Forward => 0,
            Direction::Backward => 1,
        }; // cospans[i] needs to be deleted by the smoothing rewrite

    let cs = &diagram.cospans()[i];
    if cs.is_smoothable() {
        Ok(RewriteN::new(
            diagram.dimension(),
            vec![Cone::new(
                i,
                vec![],
                cs.clone(),
                vec![cs.forward.clone()],
                vec![],
            )],
        )
        .into())
    } else {
        Err(ExpansionError::Unsmoothable)
    }
}

impl Cospan {
    fn is_smoothable(&self) -> bool {
        self.forward == self.backward
            && self
                .forward
                .max_generator()
                .map_or(false, |(g, _)| g.dimension <= self.forward.dimension())
    }
}

fn expand_base_singular(
    diagram: &Diagram,
    h0: SingularHeight,
    h1: SingularHeight,
    direction: Direction,
) -> Result<Rewrite, ExpansionError> {
    let diagram = match diagram {
        Diagram::Diagram0(_) => Err(ExpansionError::OutOfBounds),
        Diagram::DiagramN(diagram) => Ok(diagram),
    }?;

    if h0 >= diagram.size() {
        return Err(ExpansionError::OutOfBounds);
    }

    let cospan = &diagram.cospans()[h0];
    let forward: &RewriteN = (&cospan.forward).try_into().unwrap();
    let backward: &RewriteN = (&cospan.backward).try_into().unwrap();

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
                    forward.slice(h1),
                    expansion.regular_slice.into(),
                    backward.slice(h1),
                ],
                vec![
                    expansion.singular_slices[0].clone().into(),
                    expansion.singular_slices[1].clone().into(),
                ],
            );

            Ok(RewriteN::new(diagram.dimension(), vec![cone]).into())
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
                    forward.slice(h1),
                    expansion.regular_slice.into(),
                    backward.slice(h1),
                ],
                vec![
                    expansion.singular_slices[1].clone().into(),
                    expansion.singular_slices[0].clone().into(),
                ],
            );

            Ok(RewriteN::new(diagram.dimension(), vec![cone]).into())
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
    let forward_targets = forward.targets();
    let backward_targets = backward.targets();

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
        if let Some(index) = backward_index {
            cones.insert(index, new_forward1.cones()[0].clone());
        }
        RewriteN::new(forward.dimension(), cones)
    };

    Ok(ExpandedCospan {
        rewrites: [new_forward0, new_backward0, new_forward1, new_backward1],
        regular_slice: new_regular_slice,
        singular_slices: [new_singular_slice0, new_singular_slice1],
    })
}

/// Propagate a expansion on a singular level to the whole diagram
pub(crate) fn expand_propagate(
    diagram: &DiagramN,
    height: SingularHeight,
    expansion: Rewrite,
) -> Result<RewriteN, ExpansionError> {
    let slice = diagram
        .slice(Height::Singular(height))
        .ok_or(ExpansionError::OutOfBounds)?;

    let target_cospan = &diagram.cospans()[height];

    let forward = factorize(
        target_cospan.forward.clone(),
        expansion.clone(),
        slice.clone(),
    )
    .next();

    let backward = factorize(
        target_cospan.backward.clone(),
        expansion.clone(),
        slice.clone(),
    )
    .next();

    let expansion_rewrite = match (forward, backward) {
        (Some(forward), Some(backward)) => {
            let cone = if forward == backward && forward.is_redundant() {
                Cone::new_untrimmed(
                    height,
                    vec![],
                    target_cospan.clone(),
                    vec![target_cospan.forward.clone()],
                    vec![],
                )
            } else {
                Cone::new_untrimmed(
                    height,
                    vec![Cospan { forward, backward }],
                    target_cospan.clone(),
                    vec![],
                    vec![expansion],
                )
            };
            RewriteN::new(diagram.dimension(), vec![cone])
        }
        // (Some(forward), None) => {
        //     let (backward, inclusion) = factorize_inc(
        //         &slice
        //             .clone()
        //             .rewrite_backward(&target_cospan.backward)
        //             .unwrap(),
        //         &slice,
        //         &target_cospan.backward,
        //     );
        //     let (_, inner_backward, inner_forward) = antipushout(
        //         &slice.clone().rewrite_backward(&expansion).unwrap(),
        //         &slice.clone().rewrite_backward(&inclusion).unwrap(),
        //         &slice,
        //         &expansion,
        //         &inclusion,
        //     )[0]
        //     .clone();

        //     RewriteN::new(
        //         diagram.dimension(),
        //         vec![Cone::new(
        //             height,
        //             vec![
        //                 Cospan {
        //                     forward,
        //                     backward: inner_backward,
        //                 },
        //                 Cospan {
        //                     forward: inner_forward,
        //                     backward,
        //                 },
        //             ],
        //             target_cospan.clone(),
        //             todo!("need antipushout"),
        //             vec![expansion, inclusion],
        //         )],
        //     )
        // }
        // (None, Some(backward)) => {
        //     let (forward, inclusion) = factorize_inc(
        //         &slice
        //             .clone()
        //             .rewrite_backward(&target_cospan.forward)
        //             .unwrap(),
        //         &slice,
        //         &target_cospan.forward,
        //     );
        //     let (_, inner_backward, inner_forward) = antipushout(
        //         &slice.clone().rewrite_backward(&inclusion).unwrap(),
        //         &slice.clone().rewrite_backward(&expansion).unwrap(),
        //         &slice,
        //         &inclusion,
        //         &expansion,
        //     )[0]
        //     .clone();

        //     RewriteN::new(
        //         diagram.dimension(),
        //         vec![Cone::new(
        //             height,
        //             vec![
        //                 Cospan {
        //                     forward,
        //                     backward: inner_backward,
        //                 },
        //                 Cospan {
        //                     forward: inner_forward,
        //                     backward,
        //                 },
        //             ],
        //             target_cospan.clone(),
        //             todo!("need antipushout"),
        //             vec![inclusion, expansion],
        //         )],
        //     )
        // }
        _ => {
            // Insert a bubble
            // This is only logically valid if the target cospan is smoothable.
            if !target_cospan.is_smoothable() {
                return Err(ExpansionError::Unsmoothable);
            }
            RewriteN::new(
                diagram.dimension(),
                vec![Cone::new(
                    height,
                    vec![
                        Cospan {
                            forward: target_cospan.forward.clone(),
                            backward: expansion.clone(),
                        },
                        Cospan {
                            forward: expansion.clone(),
                            backward: target_cospan.backward.clone(),
                        },
                    ],
                    target_cospan.clone(),
                    vec![
                        target_cospan.forward.clone(),
                        expansion,
                        target_cospan.backward.clone(),
                    ],
                    vec![
                        Rewrite::identity(diagram.dimension() - 1),
                        Rewrite::identity(diagram.dimension() - 1),
                    ],
                )],
            )
        }
    };

    Ok(expansion_rewrite)
}
