use std::ops::Sub;
use criterion::Criterion;
use itertools::Itertools;
use spl_math::PreciseNumber;
use spl_math::uint::U256;

pub(crate) fn bench_add(c: &mut Criterion) {
    const SAMPLES: u64 = 1_000_000;

    let testdata = (0..SAMPLES)
        .map(|i| i).collect_vec();

    let mut testdata_iter = testdata.into_iter().cycle();

    c.bench_function("bench_add", |b| {
        b.iter(|| {
            let i = testdata_iter.next()?;
            let a = PreciseNumber { value: U256::from(i) };
            let b = PreciseNumber { value: U256::from(1_000_000_000_000u64.sub(i as u64)) };
            let result = a.checked_add(&b).unwrap();

            Some(result)
        });
    });
}

pub(crate) fn bench_sub(c: &mut Criterion) {
    const SAMPLES: u64 = 1_000_000;

    let testdata = (0..SAMPLES)
        .map(|i| i).collect_vec();

    let mut testdata_iter = testdata.into_iter().cycle();

    c.bench_function("bench_sub", |b| {
        b.iter(|| {
            let i = testdata_iter.next()?;
            let a = PreciseNumber { value: U256::from(1_000_000_000_000u64.sub(i as u64)) };
            let b = PreciseNumber { value: U256::from(i as u64) };
            let result = a.checked_sub(&b).unwrap();

            Some(result)
        });
    });
}


pub(crate) fn bench_unsigned_sub(c: &mut Criterion) {
    const SAMPLES: u64 = 1_000_000;

    let testdata = (0..SAMPLES)
        .map(|i| i).collect_vec();

    let mut testdata_iter = testdata.into_iter().cycle();

    c.bench_function("bench_sub", |b| {
        b.iter(|| {
            let i = testdata_iter.next()?;
            let a = PreciseNumber { value: U256::from(1_000_000_000_000u64.sub(i as u64)) };
            let b = PreciseNumber { value: U256::from(i as u64) };
            let (abs, _sign) = a.unsigned_sub(&b);

            Some(abs)
        });
    });
}

pub(crate) fn bench_ceiling(c: &mut Criterion) {
    const SAMPLES: u64 = 1_000_000;

    let testdata = (1..=SAMPLES)
        .map(|i| PreciseNumber::new(1)
            .checked_add(&PreciseNumber {
                value: U256::from(i as u64)
            }).unwrap()).collect_vec();

    let mut testdata_iter = testdata.into_iter().cycle();

    c.bench_function("bench_ceiling", |b| {
        b.iter(|| {
            let a = testdata_iter.next()?;
            let result = a.ceiling().unwrap();

            Some(result)
        });
    });
}


pub(crate) fn bench_mul(c: &mut Criterion) {
    const SAMPLES: u64 = 1_000_000;

    let testdata = (1..=SAMPLES)
        .map(|i| (
            PreciseNumber::new(100)
                .checked_add(&PreciseNumber {
                    value: U256::from(i as u64)
                }).unwrap(),
            PreciseNumber::new(200)
                .checked_add(&PreciseNumber {
                    value: U256::from(i * 3)
                }).unwrap(),
        )).collect_vec();

    let mut testdata_iter = testdata.into_iter().cycle();

    c.bench_function("bench_mul", |b| {
        b.iter(|| {
            let (a, b) = testdata_iter.next()?;
            let result = a.checked_mul(&b).unwrap();

            Some(result)
        });
    });
}



pub(crate) fn bench_div(c: &mut Criterion) {
    const SAMPLES: u64 = 1_000_000;

    let testdata = (1..=SAMPLES)
        .map(|i| (
            PreciseNumber::new(if i % 2 == 0 { 100 } else { u128::MAX / 1_000_000 })
                .checked_add(&PreciseNumber {
                    value: U256::from(i as u64)
                }).unwrap(),
            PreciseNumber::new(200)
                .checked_add(&PreciseNumber {
                    value: U256::from(i * 3)
                }).unwrap(),
        )).collect_vec();

    let mut testdata_iter = testdata.into_iter().cycle();

    c.bench_function("bench_div", |b| {
        b.iter(|| {
            let (a, b) = testdata_iter.next()?;
            let result = a.checked_div(&b).unwrap();

            Some(result)
        });
    });
}


