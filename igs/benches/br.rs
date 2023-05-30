use criterion::{criterion_group, criterion_main, Criterion, BatchSize};
use igs::solver::*;
use igs::games::chomp::*;
use std::collections::HashMap;

pub fn criterion_benchmark(c: &mut Criterion) {
    let g = Chomp::new(9,9);
    c.bench_function("br chomp 9x9", |b| b.iter_batched(
        || Solver::new(
            &g,
            HashMap::new(),
            (),
            FewerBarsFirst /*SmallerComponentsFirst{}*/,
            ()
        ),
        |mut s| s.nimber_br(g.initial_position()),
        BatchSize::SmallInput
    ));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);