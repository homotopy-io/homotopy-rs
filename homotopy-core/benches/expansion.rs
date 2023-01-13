use criterion::{criterion_group, Criterion};
use homotopy_core::{
    examples,
    typecheck::{typecheck, Mode},
    Bias, Boundary, Direction, Height,
};

fn expand_matchsticks(crit: &mut Criterion) {
    use Height::Singular;
    let mut group = crit.benchmark_group("expand matchsticks");

    let (sig, diagram) = examples::matchsticks();
    let contracted = diagram
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            Some(Bias::Lower),
            &sig,
        )
        .unwrap()
        .target();

    group.bench_function("expand", |b| {
        b.iter(|| {
            contracted
                .clone()
                .identity()
                .expand(
                    Boundary::Target.into(),
                    &mut [],
                    [Singular(0), Singular(1)],
                    Direction::Forward,
                    &sig,
                )
                .unwrap()
        });
    });

    let expanded = contracted
        .identity()
        .expand(
            Boundary::Target.into(),
            &mut [],
            [Singular(0), Singular(1)],
            Direction::Forward,
            &sig,
        )
        .unwrap()
        .into();
    group.bench_function("typecheck", |b| {
        b.iter(|| typecheck(&expanded, &sig, Mode::Deep).unwrap());
    });

    group.finish();
}

criterion_group!(expansion, expand_matchsticks);
