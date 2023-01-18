use criterion::{criterion_group, BenchmarkId, Criterion};
use homotopy_core::examples;

fn label_identifications_high_dimensions(crit: &mut Criterion) {
    let mut group = crit.benchmark_group("label identifications high dimensions");
    for dimension in 2..9 {
        group.bench_with_input(
            BenchmarkId::from_parameter(dimension),
            &dimension,
            |b, &dimension| {
                b.iter(|| {
                    examples::iterated_endomorphism(dimension);
                });
            },
        );
    }
}

criterion_group!(collapse, label_identifications_high_dimensions,);
