use criterion::{criterion_group, criterion_main, Criterion};
use itertools::Itertools;
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

#[inline(never)]
fn bench_sqrt_binary_system(c: &mut Criterion) {
    const SAMPLES: u128 = 1_000_000_000_000;

    let testdata = (0..SAMPLES).step_by(1_000_000)
        .collect_vec();

    let mut testdata_iter = testdata.clone().into_iter().cycle();

    c.bench_function("bench_sqrt_binary_system", |b| {
        b.iter(|| {
            let root = spl_math_evolved::approximations::sqrt(testdata_iter.next().unwrap());
            Some(root)
        });
    });

    let mut testdata_iter = testdata.into_iter().cycle();
    c.bench_function("bench_sqrt_binary_system_naiv", |b| {
        b.iter(|| {
            let root = spl_math_evolved::approximations::sqrt_binary_system_naiv(testdata_iter.next().unwrap());
            Some(root)
        });
    });
}


fn bench_newton(c: &mut Criterion) {
    c.bench_function("bench_newton", |b| {
        b.iter(|| calc_sqrt2);
    });
}

criterion_group!(benches_sqrt, bench_newton, bench_sqrt_binary_system,);
criterion_main!(benches_sqrt);
