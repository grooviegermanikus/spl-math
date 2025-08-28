use criterion::Criterion;
use itertools::Itertools;
use spl_math_evolved::precise_number::PreciseNumber;
use spl_math_evolved::uint::U256;

pub(crate) fn bench_pow(c: &mut Criterion) {
    const SAMPLES: u64 = 1_000_000;
    let testdata = (0..SAMPLES)
        .map(|i| {
            let one = PreciseNumber::new(1).unwrap();
            let small_number = PreciseNumber {
                value: U256::from(i * 10_000u64),
            };
            one.checked_add(&small_number).unwrap()
        }).collect_vec();

    let mut testdata_iter = testdata.into_iter().cycle();

    c.bench_function("bench_pow", |b| {
        b.iter(|| {
            let x = testdata_iter.next()?;
            let result = x.checked_pow(8).unwrap();

            Some(result)
        });
    });
}
