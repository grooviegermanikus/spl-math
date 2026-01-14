use criterion::{criterion_group, criterion_main, Criterion};
use itertools::Itertools;
use spl_math_evolved::precise_number::PreciseNumber;
use spl_math_evolved::uint::U256;


#[inline(never)]
fn calc_newton_sqrt_roots(testdata: u128) -> PreciseNumber {
    let a = PreciseNumber::new(10u128 + testdata)
        .unwrap()
        .sqrt_newton()
        .unwrap();
    let b = PreciseNumber::new(50_000_000_000_000u128 + testdata)
        .unwrap()
        .sqrt_newton()
        .unwrap();
    let c = PreciseNumber::new(50_000_000_000_000_000_000_000u128 + testdata)
        .unwrap()
        .sqrt_newton()
        .unwrap();
    let d = PreciseNumber::new(110_359_921_541_836_653_504_517_256_210_928_999_005u128 - testdata)
        .unwrap()
        .sqrt_newton()
        .unwrap();

    a.checked_add(&b)
        .unwrap()
        .checked_add(&c)
        .unwrap()
        .checked_add(&d)
        .unwrap()
}

#[inline(never)]
fn calc_cordic_sqrt_roots(testdata: u128) -> PreciseNumber {
    let a = PreciseNumber::new(10u128 + testdata)
        .unwrap()
        .sqrt_cordic()
        .unwrap();
    let b = PreciseNumber::new(50_000_000_000_000u128 + testdata)
        .unwrap()
        .sqrt_cordic()
        .unwrap();
    let c = PreciseNumber::new(50_000_000_000_000_000_000_000u128 + testdata)
        .unwrap()
        .sqrt_cordic()
        .unwrap();
    let d = PreciseNumber::new(110_359_921_541_836_653_504_517_256_210_928_999_005u128 - testdata)
        .unwrap()
        .sqrt_cordic()
        .unwrap();

    a.checked_add(&b)
        .unwrap()
        .checked_add(&c)
        .unwrap()
        .checked_add(&d)
        .unwrap()
}

#[inline(never)]
fn bench_sqrt_binary_system(c: &mut Criterion) {
    const SAMPLES: u128 = 1_000_000_000_000;

    let testdata = (0..SAMPLES).step_by(1_000_000).collect_vec();

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
            let root = spl_math_evolved::approximations::sqrt_binary_system_naiv(
                testdata_iter.next().unwrap(),
            );
            Some(root)
        });
    });
}

fn bench_fast_newton_sqrt(c: &mut Criterion) {
    const SAMPLES: u128 = 1_000_000;
    let testdata = (0..SAMPLES).step_by(13).collect_vec();
    let mut testdata_iter = testdata.into_iter().cycle();
    c.bench_function("bench_fast_newton_sqrt", |b| {
        b.iter(|| {
            let sum = calc_newton_sqrt_roots(testdata_iter.next().unwrap());
            assert!(sum.value > U256::zero());
        });
    });
}

fn bench_fast_cordic_sqrt(c: &mut Criterion) {
    const SAMPLES: u128 = 1_000_000;
    let testdata = (0..SAMPLES).step_by(13).collect_vec();
    let mut testdata_iter = testdata.into_iter().cycle();
    c.bench_function("bench_fast_coric_sqrt", |b| {
        b.iter(|| {
            let sum = calc_cordic_sqrt_roots(testdata_iter.next().unwrap());
            assert!(sum.value > U256::zero());
        });
    });
}

criterion_group!(benches_sqrt, bench_fast_newton_sqrt, bench_fast_cordic_sqrt, bench_sqrt_binary_system,);
criterion_main!(benches_sqrt);
