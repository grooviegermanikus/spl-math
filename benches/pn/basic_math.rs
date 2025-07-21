use criterion::Criterion;
use itertools::Itertools;
use spl_math::PreciseNumber;
use spl_math::uint::U256;

pub(crate) fn bench_add(c: &mut Criterion) {

    const SAMPLES: usize = 1_000_000;

    // note: must be infinite
    let mut testdata_iter = (0..SAMPLES)
        .map(|i| i).cycle();

    c.bench_function("bench_add", |b| {
        b.iter(|| {
            let i = testdata_iter.next()?;
            let a = PreciseNumber { value: U256::from(i as u64) };
            let b = PreciseNumber { value: U256::from(1_000_000_000_000u64) };
            let result = a.checked_add(&b).unwrap();

            Some(result)
        });
    });
}
