use criterion::{criterion_group, criterion_main};

mod muldiv;
mod pn;

criterion_group!(
    benches_basic_math,
    pn::basic_math::bench_to_imprecise,
    pn::basic_math::bench_add,
    pn::basic_math::bench_sub,
    pn::basic_math::bench_unsigned_sub,
    pn::basic_math::bench_ceiling,
    pn::basic_math::bench_mul,
    pn::basic_math::bench_div,
    pn::basic_math::bench_div_bigdecimal_lib,
    pn::basic_math::bench_div_fixed_lib,
);

criterion_group!(benches_pow, pn::pow::bench_pow,);

criterion_group!(benches_sqrt, pn::sqrt::bench_sqrt,);

criterion_group!(benches_muldiv, muldiv::basic::bench_muldiv_nooverflow,
    muldiv::basic::bench_muldiv_overflowing,);

criterion_main!(benches_basic_math, benches_pow, benches_sqrt,benches_muldiv,
);
