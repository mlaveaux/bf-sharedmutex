use std::sync::{Arc, RwLock, Mutex};

use criterion::Criterion;

use bf_sharedmutex::BfSharedMutex;

use benchmarks::{benchmark, NUM_ITERATIONS, READ_RATIOS, THREADS};

/// Benchmark the bfsharedmutex implementation
pub fn benchmark_bfsharedmutex(c: &mut Criterion) {
    for num_threads in THREADS {
        for read_ratio in READ_RATIOS {

            // Benchmark various configurations.
            benchmark(
                c,
                "bf-sharedmutex::BfSharedMutex",
                BfSharedMutex::new(()),
                |shared| {
                    let _guard = shared.read().unwrap();
                },
                |shared| {
                    let _guard = shared.write().unwrap();
                },
                num_threads,
                NUM_ITERATIONS,
                read_ratio,
            );
        }
    }
}

// Split up to first do our own benchmarks since than we can update the implementation easily.   
pub fn benchmark_othermutexes(c: &mut Criterion) {
    for num_threads in THREADS {
        benchmark(
            c,
            "std::sync::Mutex",
            Arc::new(Mutex::new(())),
            |_| {
            },
            |shared| {
                let _guard = shared.lock();
            },
            num_threads,
            NUM_ITERATIONS,
            1,
        );

        for read_ratio in READ_RATIOS {

            benchmark(
                c,
                "std::sync::RwLock",
                Arc::new(RwLock::new(())),
                |shared| {
                    let _guard = shared.read().unwrap();
                },
                |shared| {
                    let _guard = shared.write().unwrap();
                },
                num_threads,
                NUM_ITERATIONS,
                read_ratio,
            );

            benchmark(
                c,
                "parking_lot::RwLock",
                Arc::new(parking_lot::RwLock::new(())),
                |shared| {
                    let _guard = shared.read();
                },
                |shared| {
                    let _guard = shared.write();
                },
                num_threads,
                NUM_ITERATIONS,
                read_ratio,
            );

            benchmark(
                c,
                "spin::RwLock",
                Arc::new(spin::RwLock::new(())),
                |shared| {
                    shared.read();
                },
                |shared| {
                    shared.write();
                },
                num_threads,
                NUM_ITERATIONS,
                read_ratio,
            );

            // This lock seems to deadlock.
            // benchmark(
            //     c,
            //     "widerwlock::WideRwLock",
            //     Arc::new(widerwlock::WideRwLock::new(())),
            //     |shared| {
            //         shared.read();
            //     },
            //     |shared| {
            //         shared.write();
            //     },
            //     num_threads,
            //     NUM_ITERATIONS,
            //     read_ratio,
            // );

            benchmark(
                c,
                "shared_mutex::SharedMutex",
                Arc::new(shared_mutex::SharedMutex::new(())),
                |shared| {
                    let _guard = shared.read();
                },
                |shared| {
                    let _guard = shared.write();
                },
                num_threads,
                NUM_ITERATIONS,
                read_ratio,
            );

            benchmark(
                c,
                "pairlock::RwLock",
                Arc::new(pairlock::PairLock::with_default(())),
                |shared| {
                    shared.view(|_| {});
                },
                |shared| {
                    shared.update();
                },
                num_threads,
                NUM_ITERATIONS,
                read_ratio,
            );

            benchmark(
                c,
                "pflock::PFLock",
                Arc::new(pflock::PFLock::new(())),
                |shared| {
                    let _guard = shared.read();
                },
                |shared| {
                    let _guard = shared.write();
                },
                num_threads,
                NUM_ITERATIONS,
                read_ratio,
            );

            benchmark(
                c,
                "crossbeam::ShardedLock",
                Arc::new(crossbeam::sync::ShardedLock::new(())),
                |shared| {
                    let _guard = shared.read().unwrap();
                },
                |shared| {
                    let _guard = shared.write().unwrap();
                },
                num_threads,
                NUM_ITERATIONS,
                read_ratio,
            );

            // This library might work on linux, but even there it does seem to have a weird API.
            // #[cfg(target_os = "linux")]
            // benchmark(c,
            //     "process_sync::SharedMemoryObject",
            //     Arc::new(process_sync::SharedMemoryObject::new(5).unwrap()),
            //     |shared| {
            //         shared.get();
            //     }, |shared| {
            //         shared.get_mut();
            //     },
            //     num_threads,
            //     NUM_ITERATIONS,
            //     read_ratio);
        }
    }
}