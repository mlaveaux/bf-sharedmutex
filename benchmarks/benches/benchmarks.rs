use criterion::{criterion_group, criterion_main};

mod async_benchmarks;
mod mutex_benchmarks;

criterion_group!(
    benches,
    mutex_benchmarks::benchmark_bfsharedmutex,
    mutex_benchmarks::benchmark_othermutexes,
    async_benchmarks::benchmark_async,
);
criterion_main!(benches);