use criterion::{criterion_group, BenchmarkId, Criterion};
use homotopy_core::typecheck::{typecheck, Mode};
use homotopy_core::*;

fn contract_scalar(crit: &mut Criterion) {
    let mut group = crit.benchmark_group("contract scalar");

    let (sig, diagram) = examples::two_scalars();
    group.bench_function("left", |b| {
        b.iter(|| {
            diagram
                .identity()
                .contract(&Boundary::Target.into(), &[], 0, Some(Bias::Lower), &sig)
        })
    });
    group.bench_function("right", |b| {
        b.iter(|| {
            diagram
                .identity()
                .contract(&Boundary::Target.into(), &[], 0, Some(Bias::Higher), &sig)
        })
    });

    group.finish();
}

fn contract_beads(crit: &mut Criterion) {
    let mut group = crit.benchmark_group("contract beads");

    let (sig, diagram) = examples::three_beads();
    group.bench_function("contract", |b| {
        b.iter(|| {
            diagram
                .identity()
                .contract(&Boundary::Target.into(), &[], 1, None, &sig)
        })
    });

    let contracted = diagram
        .identity()
        .contract(&Boundary::Target.into(), &[], 1, None, &sig)
        .unwrap()
        .into();
    group.bench_function("typecheck", |b| {
        b.iter(|| typecheck(&contracted, &sig, Mode::Deep).unwrap())
    });

    group.finish();
}

fn contract_stacks(crit: &mut Criterion) {
    let mut group = crit.benchmark_group("contract stacks");

    let (sig, diagram) = examples::stacks();
    group.bench_function("contract", |b| {
        b.iter(|| {
            diagram
                .identity()
                .contract(&Boundary::Target.into(), &[], 0, None, &sig)
        })
    });

    let contracted = diagram
        .identity()
        .contract(&Boundary::Target.into(), &[], 0, None, &sig)
        .unwrap()
        .into();
    group.bench_function("typecheck", |b| {
        b.iter(|| typecheck(&contracted, &sig, Mode::Deep).unwrap())
    });
    group.finish();
}

fn contract_high_dimensions(crit: &mut Criterion) {
    let mut group = crit.benchmark_group("contract high dimensions");
    for dimension in 2..10 {
        let (sig, dn) = examples::bead_series(dimension);
        group.bench_with_input(
            BenchmarkId::from_parameter(dimension),
            &dimension,
            |b, &dimension| {
                b.iter(|| {
                    let mut diagram = dn.clone();
                    for i in (1..dimension).rev() {
                        diagram = diagram.identity();
                        for _ in 0..i {
                            diagram = diagram
                                .contract(&Boundary::Target.into(), &[], 0, None, &sig)
                                .unwrap();
                        }
                    }
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    contraction,
    contract_scalar,
    contract_beads,
    contract_stacks,
    contract_high_dimensions
);
