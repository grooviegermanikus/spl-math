use std::ops::Sub;
use std::str::FromStr;
use bigdecimal_rs::BigDecimal;
use criterion::Criterion;
use itertools::Itertools;
use spl_math::PreciseNumber;
use spl_math::uint::U256;


pub(crate) fn bench_to_imprecise(c: &mut Criterion) {
    const SAMPLES: u64 = 10_000;

    let testdata = (0..SAMPLES)
        .map(|i| {
            let one = PreciseNumber::new(2);
            one.checked_mul(&PreciseNumber { value: U256::from(i * 1_000_000) }).unwrap()

        }).collect_vec();

    let mut testdata_iter = testdata.into_iter().cycle();

    c.bench_function("bench_to_imprecise", |b| {
        b.iter(|| {
            let a = testdata_iter.next()?;
            let result = a.to_imprecise().unwrap();
            Some(result)
        });
    });
}



pub(crate) fn bench_add(c: &mut Criterion) {
    const SAMPLES: u64 = 10_000;

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
    const SAMPLES: u64 = 10_000;

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
    const SAMPLES: u64 = 10_000;

    let testdata = (0..SAMPLES)
        .map(|i| i).collect_vec();

    let mut testdata_iter = testdata.into_iter().cycle();

    c.bench_function("bench_unsigned_sub", |b| {
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
    const SAMPLES: u64 = 10_000;

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
    const SAMPLES: u64 = 10_000;

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
    const SAMPLES: u64 = 10_000;

    let testdata = (1..=SAMPLES)
        .map(create_divisor_dividend)
        .collect_vec();

    let mut testdata_iter = testdata.into_iter().cycle();

    c.bench_function("bench_div", |b| {
        b.iter(|| {
            let (a, b) = testdata_iter.next()?;
            let result = a.checked_div(&b).unwrap();

            Some(result)
        });
    });
}

// TODO consolidate
pub(crate) fn bench_div_bigdecimal(c: &mut Criterion) {
    const SAMPLES: u64 = 10_000;

    let testdata = (1..=SAMPLES)
        .map(create_divisor_dividend)
        .map(|(a, b)| {
            let fx_one = BigDecimal::from_str("1000000000000").unwrap(); // 1e12
            let a = BigDecimal::from_str(&format!("{}", a.value)).unwrap() / &fx_one;
            let b = BigDecimal::from_str(&format!("{}", b.value)).unwrap() / &fx_one;
            (a, b)
        }
        )
        .collect_vec();

    let mut testdata_iter = testdata.into_iter().cycle();

    c.bench_function("bench_div_bigdecimal", |b| {
        b.iter(|| {
            let (a, b) = testdata_iter.next()?;
            let result = a / b;

            Some(result)
        });
    });
}

fn create_divisor_dividend(i: u64) -> (PreciseNumber, PreciseNumber) {
    (
        PreciseNumber::new(if i % 2 == 0 { 100 } else { u128::MAX / 1_000_000 })
            .checked_add(&PreciseNumber {
                value: U256::from(i as u64)
            }).unwrap(),
        PreciseNumber::new(200)
            .checked_add(&PreciseNumber {
                value: U256::from(i * 3)
            }).unwrap(),
    )
}
