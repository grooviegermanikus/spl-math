use criterion::Criterion;
use itertools::Itertools;
use spl_math_evolved::precise_number::PreciseNumber;
use spl_math_evolved::uint::U256;

pub(crate) fn bench_muldiv_nooverflow(c: &mut Criterion) {
    let testdata = (0..100_000u32)
        .map(|i| (123 + i, 456 - i, (500 + i) / 10))
        .map(|(a, b, c)| {
            let a = PreciseNumber { value: U256::from(a) };
            let b = PreciseNumber { value: U256::from(b) };
            let c = PreciseNumber { value: U256::from(c) };
            (a, b, c)
        })
        .collect_vec();

    let mut testdata_iter = testdata.into_iter().cycle();

    c.bench_function("bench_muldiv_floor", |b| {
        b.iter(|| {
            let (a, b, c) = testdata_iter.next()?;
            let result = a.mul_div_floor(b, c);
            Some(result)
        });
    });

    c.bench_function("bench_muldiv_floor_naive", |b| {
        b.iter(|| {
            let (a, b, c) = testdata_iter.next()?;
            let result = a.mul_div_floor_naive(b, c);
            Some(result)
        });
    });

    c.bench_function("bench_muldiv_ceil", |b| {
        b.iter(|| {
            let (a, b, c) = testdata_iter.next()?;
            let result = a.mul_div_ceil(b, c);
            Some(result)
        });
    });

    c.bench_function("bench_muldiv_ceil_naive", |b| {
        b.iter(|| {
            let (a, b, c) = testdata_iter.next()?;
            let result = a.mul_div_ceil_naive(b, c);
            Some(result)
        });
    });
}


pub(crate) fn bench_muldiv_overflowing(c: &mut Criterion) {
    let ten = U256::from_dec_str("10").unwrap();
    let testdata = (0..100_000u32)
        .map(|i| ((13 * i) % 20, (29 * i) % 15, (500 + i) / 10))
        .map(|(a, b, c)| {
            // 1e77
            let a = ten.pow(U256::from(44 + a));
            let a = PreciseNumber { value: a };
            let b = ten.pow(U256::from(48 + b));
            let b = PreciseNumber { value: b };
            let c = PreciseNumber { value: U256::from(c) };
            (a, b, c)
        })
        .collect_vec();

    let mut testdata_iter = testdata.into_iter().cycle();

    c.bench_function("bench_muldiv_floor_large", |b| {
        b.iter(|| {
            let (a, b, c) = testdata_iter.next()?;
            let result = a.mul_div_floor(b, c);
            Some(result)
        });
    });

    c.bench_function("bench_muldiv_floor_naive_large", |b| {
        b.iter(|| {
            let (a, b, c) = testdata_iter.next()?;
            let result = a.mul_div_floor_naive(b, c);
            Some(result)
        });
    });

    c.bench_function("bench_muldiv_ceil_large", |b| {
        b.iter(|| {
            let (a, b, c) = testdata_iter.next()?;
            let result = a.mul_div_ceil(b, c);
            Some(result)
        });
    });

    c.bench_function("bench_muldiv_ceil_naive_large", |b| {
        b.iter(|| {
            let (a, b, c) = testdata_iter.next()?;
            let result = a.mul_div_ceil_naive(b, c);
            Some(result)
        });
    });
}

