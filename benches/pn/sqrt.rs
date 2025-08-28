use criterion::Criterion;
use spl_math_evolved::precise_number::PreciseNumber;

pub(crate) fn bench_sqrt(c: &mut Criterion) {

    let a = PreciseNumber::new(10u128).unwrap();
    c.bench_function("bench_sqrt_small", |b| {
        b.iter(|| {
            Some(a.sqrt())
        });
    });

    let a = PreciseNumber::new(50_000_000_000_000u128).unwrap();
    c.bench_function("bench_sqrt_50", |b| {
        b.iter(|| {
            Some(a.sqrt())
        });
    });

    let a = PreciseNumber::new(50_000_000_000_000_000_000_000u128).unwrap();
    c.bench_function("bench_sqrt_50bn", |b| {
        b.iter(|| {
            Some(a.sqrt())
        });
    });

    let a = PreciseNumber::new(110_359_921_541_836_653_504_517_256_210_928_999_005u128).unwrap();
    c.bench_function("bench_sqrt_very_large", |b| {
        b.iter(|| {
            Some(a.sqrt())
        });
    });

}
