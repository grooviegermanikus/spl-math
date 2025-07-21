use criterion::{criterion_group, criterion_main, Criterion};
use spl_math::PreciseNumber;

mod pn;

criterion_group!(
    benches_basic_math,
    pn::basic_math::bench_add,
);

criterion_main!(benches_basic_math);
