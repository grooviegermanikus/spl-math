use criterion::{criterion_group, criterion_main};

mod pn;

criterion_group!(
    benches_basic_math,
    pn::basic_math::bench_add,
    pn::basic_math::bench_sub,
    pn::basic_math::bench_unsigned_sub,
    pn::basic_math::bench_ceiling,
    pn::basic_math::bench_mul,
    pn::basic_math::bench_div,
);

criterion_main!(benches_basic_math);
