use homotopy_core::*;
use quickcheck::*;
use quickcheck_macros::quickcheck;

fn gen_generator_1d(g: &mut Gen) -> Generator {
    let id = *g.choose(&[1, 2]).unwrap();
    Generator::new(id, 1)
}

fn create_cospan(generator: Generator) -> Cospan {
    let x = Generator::new(0, 0);
    Cospan {
        forward: Rewrite0::new(x, generator).into(),
        backward: Rewrite0::new(x, generator).into(),
    }
}

fn gen_rewrite(g: &mut Gen, source: &[Generator]) -> (RewriteN, Vec<Generator>) {
    let mut slices: Vec<Vec<Rewrite>> = Vec::new();
    let mut source_remaining = source;
    let mut target = Vec::new();

    loop {
        if source_remaining.is_empty() && bool::arbitrary(g) {
            break;
        }

        let sizes: Vec<usize> = [0, 0, 1, 1, 1, 2, 3]
            .iter()
            .copied()
            .filter(|i| *i <= source_remaining.len())
            .collect();

        let size = *g.choose(&sizes).unwrap();
        let target_generator = gen_generator_1d(g);

        target.push(target_generator);

        slices.push(
            source_remaining[..size]
                .iter()
                .map(|source_generator| Rewrite0::new(*source_generator, target_generator).into())
                .collect(),
        );

        source_remaining = &source_remaining[size..];
    }

    let source_cospans: Vec<_> = source.iter().copied().map(create_cospan).collect();
    let target_cospans: Vec<_> = target.iter().copied().map(create_cospan).collect();
    let rewrite = RewriteN::from_slices(1, &source_cospans, &target_cospans, slices);
    (rewrite, target)
}

#[derive(Debug, Clone)]
struct ComposableRewrites {
    source_size: usize,
    first: RewriteN,
    second: RewriteN,
}

impl Arbitrary for ComposableRewrites {
    fn arbitrary(g: &mut Gen) -> Self {
        let source_size = usize::arbitrary(g) % g.size();
        let source: Vec<_> = (0..source_size).map(|_| gen_generator_1d(g)).collect();
        let (first, middle) = gen_rewrite(g, &source);
        let (second, _) = gen_rewrite(g, &middle);
        ComposableRewrites {
            source_size,
            first,
            second,
        }
    }
}

#[quickcheck]
fn compose_monotone(rewrites: ComposableRewrites) {
    let composed = RewriteN::compose(&rewrites.first, &rewrites.second).unwrap();
    let actual: Vec<usize> = (0..rewrites.source_size)
        .map(|i| composed.singular_image(i))
        .collect();

    let expected: Vec<usize> = (0..rewrites.source_size)
        .map(|i| {
            rewrites
                .second
                .singular_image(rewrites.first.singular_image(i))
        })
        .collect();

    assert_eq!(actual, expected);
}
