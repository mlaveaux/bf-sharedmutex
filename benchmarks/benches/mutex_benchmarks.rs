use std::{
    hint::black_box,
    sync::{atomic::AtomicBool, Arc, Barrier, RwLock},
    thread::{self},
};

use criterion::Criterion;

use bf_sharedmutex::BfSharedMutex;
use rand::{prelude::*, distributions::Bernoulli};

// Settings for the benchmarks.
pub const NUM_ITERATIONS: usize = 100000;
pub const THREADS: [usize; 6] = [1, 2, 4, 8, 16, 20];
pub const READ_RATIOS: [u32; 5] = [10, 100, 1000, 10000, 100000];

/// Execute the benchmarks for a given readers-writer lock implementation.
pub fn benchmark_lock<T, R, W>(
    c: &mut Criterion,
    name: &str,
    shared: T,
    read: R,
    write: W,
    num_threads: usize,
    num_iterations: usize,
    read_ratio: u32,
) where
    T: Clone + Send + 'static,
    R: FnOnce(&T) -> () + Send + Copy + 'static,
    W: FnOnce(&T) -> () + Send + Copy + 'static,
{
    // Share threads to avoid overhead.
    let mut threads = vec![];

    #[derive(Clone)]
    struct ThreadInfo<T> {
        busy: Arc<AtomicBool>,
        begin_barrier: Arc<Barrier>,
        end_barrier: Arc<Barrier>,
        dist: Bernoulli,
        shared: T,
    }

    let info = ThreadInfo {
        busy: Arc::new(AtomicBool::new(true)),
        begin_barrier: Arc::new(Barrier::new(num_threads + 1)),
        end_barrier: Arc::new(Barrier::new(num_threads + 1)),
        dist: Bernoulli::from_ratio(1, read_ratio).unwrap(),
        shared,
    };

    for _ in 0..num_threads {
        let info = info.clone();
        threads.push(thread::spawn(move || {
            let mut rng = rand::thread_rng();

            loop {
                info.begin_barrier.wait();

                if !info.busy.load(std::sync::atomic::Ordering::SeqCst) {
                    // Quit the thread.
                    break;
                }

                // We execute it a fixed number of times.
                for _ in 0..num_iterations {
                    if info.dist.sample(&mut rng) {
                        black_box(write(&info.shared));
                    } else {
                        black_box(read(&info.shared));
                    }
                }

                info.end_barrier.wait();
            }
        }));
    }

    c.bench_function(
        format!(
            "{} {} {} {}",
            name, num_threads, num_iterations, read_ratio
        )
        .as_str(),
        |bencher| {
            bencher.iter(|| {
                info.begin_barrier.wait();

                info.end_barrier.wait();
            });
        },
    );

    // Tell the threads to quit and wait for them to join.
    info.busy.store(false, std::sync::atomic::Ordering::SeqCst);
    info.begin_barrier.wait();

    for thread in threads {
        thread.join().unwrap();
    }
}

/// Benchmark the bfsharedmutex implementation
pub fn benchmark_bfsharedmutex(c: &mut Criterion) {
    for num_threads in THREADS {
        for read_ratio in READ_RATIOS {

            // Benchmark various configurations.
            benchmark_lock(
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
        for read_ratio in READ_RATIOS {
            benchmark_lock(
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

            benchmark_lock(
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

            benchmark_lock(
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
            // benchmark_lock(
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

            benchmark_lock(
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

            benchmark_lock(
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

            benchmark_lock(
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

            benchmark_lock(
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
            // benchmark_lock(c,
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