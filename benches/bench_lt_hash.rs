use criterion::{criterion_group, criterion_main, Criterion};
use spl_math::PreciseNumber;

#[inline(never)]
fn call_it1() {

    PreciseNumber::new(1_000_000_000_000u128).sqrt().unwrap();


}

#[inline(never)]
fn call_it2() {

    spl_math::approximations::sqrt(10000);

}

fn bench_newton(c: &mut Criterion) {
    c.bench_function("bench_newton", |b| {
        b.iter(|| call_it1());
    });
}

fn bench_approx(c: &mut Criterion) {
    c.bench_function("sqrt_approx", |b| {
        b.iter(|| call_it2());
    });
}

criterion_group!(
    benches22,
    bench_newton,
    bench_approx,
);
criterion_main!(benches22);

