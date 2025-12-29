use criterion::{criterion_group, criterion_main, Criterion};
use spl_math_evolved::precise_number::PreciseNumber;

#[inline(never)]
fn calc_sqrt2() {
    PreciseNumber::new(10u128).unwrap().sqrt().unwrap();
    PreciseNumber::new(50_000_000_000_000u128)
        .unwrap()
        .sqrt()
        .unwrap();
    PreciseNumber::new(50_000_000_000_000_000_000_000u128)
        .unwrap()
        .sqrt()
        .unwrap();
    PreciseNumber::new(110_359_921_541_836_653_504_517_256_210_928_999_005u128)
        .unwrap()
        .sqrt()
        .unwrap();
}

fn bench_newton(c: &mut Criterion) {
    c.bench_function("bench_newton", |b| {
        b.iter(|| calc_sqrt2);
    });
}

criterion_group!(benches_sqrt, bench_newton,);
criterion_main!(benches_sqrt);
