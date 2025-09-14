use criterion::Criterion;
use itertools::Itertools;
use spl_math_evolved::muldiv::{fast, slow};
use spl_math_evolved::precise_number::PreciseNumber;
use spl_math_evolved::uint::U256;

pub(crate) fn bench_signum3(c: &mut Criterion) {

    let testdata = (0..10_000)
        .map(|i| ( -123 + i, 456i32 - i, -789i32 + i) )
        .collect_vec();

    let mut testdata_iter = testdata.into_iter().cycle();

    c.bench_function("bench_signum3", |b| {
        b.iter(|| {
            let (a, b, c) = testdata_iter.next()?;
            let result = slow(a, b, c);
            Some(result)
        });
    });


}


pub(crate) fn bench_signum3_fast(c: &mut Criterion) {

    let testdata = (0..10_000)
        .map(|i| ( -123 + i, 456i32 - i, -789i32 + i) )
        .collect_vec();

    let mut testdata_iter = testdata.into_iter().cycle();

    c.bench_function("bench_signum3_fast", |b| {
        b.iter(|| {
            let (a, b, c) = testdata_iter.next()?;
            let result = fast(a, b, c);
            Some(result)
        });
    });



}


