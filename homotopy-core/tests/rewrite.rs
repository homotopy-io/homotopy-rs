use homotopy_core::{
    common::{BoundaryPath, Label},
    Boundary::{Source, Target},
    Cospan, Diagram0, DiagramN, Generator,
    Height::{Regular, Singular},
    Rewrite0, RewriteN,
};
use proptest::prelude::*;

prop_compose! {
    fn f_or_g()
        (id in 1..3_usize)
    -> Generator {
        Generator::new(id, 1)
    }
}

const MAX_SIZE: usize = 5;

// choose cone size of 0 with probability 2/7
// choose cone size of 1 with probability 3/7
// choose cone size of 2 with probability 1/7
// choose cone size of 3 with probability 1/7
fn choose(i: usize) -> usize {
    match i {
        i if i < 2 => 0,
        i if i < 5 => 1,
        i if i < 6 => 2,
        i if i < 7 => 3,
        _ => unreachable!(),
    }
}

fn arb_cone_sizes_fixed_width(target: usize) -> impl Strategy<Value = Vec<(usize, Generator)>> {
    match target {
        s if s == 0 => prop::collection::vec((0..2_usize, f_or_g()), 0..2).boxed(),
        s if s == 1 => (0..5_usize, f_or_g()).prop_flat_map(move |(i, g)| arb_cone_sizes_fixed_width(target - choose(i)).prop_map(move |mut vec| {vec.push((i, g)); vec })).boxed(),
        s if s == 2 => (0..6_usize, f_or_g()).prop_flat_map(move |(i, g)| arb_cone_sizes_fixed_width(target - choose(i)).prop_map(move |mut vec| {vec.push((i, g)); vec })).boxed(),
        _ /* pick any size */ => (0..7_usize, f_or_g()).prop_flat_map(move |(i, g)| arb_cone_sizes_fixed_width(target - choose(i)).prop_map(move |mut vec| {vec.push((i, g)); vec })).boxed(),
    }
}

prop_compose! {
    pub(crate) fn arb_rewrite_1d(sources: Vec<Generator>)
        (mut cone_sizes in arb_cone_sizes_fixed_width(sources.len()))
    -> (RewriteN, Vec<Generator>) {
        let x = Generator::new(0, 0);
        let internal = |g: Generator| -> Cospan {
            Cospan {
                forward: Rewrite0::new(x, g, Label::new(BoundaryPath(Source, 0), std::iter::once(vec![]).collect()).into()).into(),
                backward: Rewrite0::new(x, g, Label::new(BoundaryPath(Target, 0), std::iter::once(vec![]).collect()).into()).into(),
            }
        };

        let mut regular_slices = Vec::new();
        let mut singular_slices = Vec::new();
        let mut sources_remaining = sources.as_slice();
        let mut targets = Vec::new();
        while !sources_remaining.is_empty() && !cone_sizes.is_empty() {
            // add a new cone
            let (size_index, target) = cone_sizes.pop().unwrap();
            let size = std::cmp::min(sources_remaining.len(), choose(size_index));
            targets.push(target);

            singular_slices.push(
                sources_remaining[..size]
                    .iter()
                    .enumerate()
                    .map(|(i, &source)| {
                        Rewrite0::new(
                            source,
                            target,
                            Some(Label::new(
                                BoundaryPath(Source, 0),
                                std::iter::once(vec![Singular(i)]).collect(),
                            )),
                        )
                        .into()
                    })
                    .collect(),
            );

            regular_slices.push(
                (0..=size)
                    .map(|r| {
                        Rewrite0::new(
                            x,
                            target,
                            Some(Label::new(
                                BoundaryPath(Source, 0),
                                std::iter::once(vec![Regular(r)]).collect(),
                            )),
                        )
                        .into()
                    })
                    .collect(),
            );

            sources_remaining = &sources_remaining[size..];
        }

        let source_cospans: Vec<_> = sources.iter().copied().map(internal).collect();
        let target_cospans: Vec<_> = targets.iter().copied().map(internal).collect();
        let rewrite = RewriteN::from_slices(
            1,
            &source_cospans,
            &target_cospans,
            regular_slices,
            singular_slices,
        );
        (rewrite, targets)
    }
}

pub(crate) fn arb_rewrites_1d_composable() -> impl Strategy<Value = (usize, RewriteN, RewriteN)> {
    prop::collection::vec(f_or_g(), 1..MAX_SIZE).prop_flat_map(|sources| {
        let source_size = sources.len();
        arb_rewrite_1d(sources).prop_flat_map(move |(first, middle)| {
            arb_rewrite_1d(middle).prop_map(move |(second, _)| (source_size, first.clone(), second))
        })
    })
}

prop_compose! {
    pub(crate) fn arb_rewrite_1d_with_source_and_target()
        (sources in prop::collection::vec(f_or_g(), 1..MAX_SIZE))
        ((rewrite, targets) in arb_rewrite_1d(sources.clone()), sources in Just(sources))
    -> (RewriteN, DiagramN, DiagramN) {
        let x = Generator::new(0, 0);
        let internal = |g: Generator| -> Cospan {
            Cospan {
                forward: Rewrite0::new(x, g, Label::new(BoundaryPath(Source, 0), std::iter::once(vec![]).collect()).into()).into(),
                backward: Rewrite0::new(x, g, Label::new(BoundaryPath(Target, 0), std::iter::once(vec![]).collect()).into()).into(),
            }
        };
        (
            rewrite,
            DiagramN::new(
                Diagram0::from(x).into(),
                sources.into_iter().map(internal).collect(),
            ),
            DiagramN::new(
                Diagram0::from(x).into(),
                targets.into_iter().map(internal).collect(),
            ),
        )
    }
}

proptest! {
    #[test]
    fn compose_monotone((source_size, first, second) in arb_rewrites_1d_composable()) {
        let composed = RewriteN::compose(&first, &second);
        prop_assert!(composed.is_ok());
        let actual: Vec<usize> = (0..source_size)
            .map(|i| composed.as_ref().unwrap().singular_image(i))
            .collect();

        let expected: Vec<usize> = (0..source_size)
            .map(|i| {
                    second
                    .singular_image(first.singular_image(i))
            })
            .collect();

        prop_assert_eq!(actual, expected);
    }
}
