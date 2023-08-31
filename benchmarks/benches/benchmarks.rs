use std::{hint::black_box, thread, sync::{RwLock, Arc}};

use criterion::{criterion_group, criterion_main, Criterion};

use rand::prelude::*;
use bf_sharedmutex::BfSharedMutex;
    
pub fn benchmark_sharedmutexes(c: &mut Criterion) {

    let read_percentage = 0.99;
    let num_threads = 20;
    let num_iterations = 100000;

    // This executes the same code sequentially for comparison of total time.
    c.bench_function("sequential", |bencher| {

        bencher.iter(|| {        
            let mut vector = vec![];

            for _ in 1..num_threads {
                let mut rng = rand::thread_rng();  

                for _ in 0..num_iterations {
                    if rng.gen_bool(read_percentage) {
                        // Read a random index.
                        let read = &vector;
                        if read.len() > 0 {
                            let index = rng.gen_range(0..read.len());
                            black_box(assert_eq!(read[index], 5));
                        }
                    } else {
                        // Add a new vector element.
                        vector.push(5);
                    }
                };
            }
        });
    });

    c.bench_function("bf-sharedmutex 20", |bencher| {

        bencher.iter(|| {  
            let shared_vector = BfSharedMutex::new(vec![]);

            let mut threads = vec![];
            for _ in 1..num_threads {
                let shared_vector = shared_vector.clone();
                threads.push(thread::spawn(move || {
                    let mut rng = rand::thread_rng();  

                    for _ in 0..num_iterations {
                        if rng.gen_bool(read_percentage) {
                            // Read a random index.
                            let read = shared_vector.read().unwrap();
                            if read.len() > 0 {
                                let index = rng.gen_range(0..read.len());
                                black_box(assert_eq!(read[index], 5));
                            }
                        } else {
                            // Add a new vector element.
                            shared_vector.write().unwrap().push(5);
                        }
                    }
                
                }));
            }

            // Check whether threads have completed succesfully.
            for thread in threads {
                thread.join().unwrap();
            }
        });
    });

    c.bench_function("RwLock 20", |bencher| {
        
        bencher.iter(|| {  
            let shared_vector = Arc::new(RwLock::new(vec![]));

            let mut threads = vec![];
            for _ in 1..num_threads {
                let shared_vector = shared_vector.clone();
                threads.push(thread::spawn(move || {
                    let mut rng = rand::thread_rng();  

                    for _ in 0..num_iterations {
                        if rng.gen_bool(read_percentage) {
                            // Read a random index.
                            let read = shared_vector.read().unwrap();
                            if read.len() > 0 {
                                let index = rng.gen_range(0..read.len());
                                black_box(assert_eq!(read[index], 5));
                            }
                        } else {
                            // Add a new vector element.
                            shared_vector.write().unwrap().push(5);
                        }
                    }
                
                }));
            }

            // Check whether threads have completed succesfully.
            for thread in threads {
                thread.join().unwrap();
            }
        })
    });

    c.bench_function("parking_lot RwLock 20", |bencher| {
        
        bencher.iter(|| {  
            let shared_vector = Arc::new(parking_lot::RwLock::new(vec![]));

            let mut threads = vec![];
            for _ in 1..num_threads {
                let shared_vector = shared_vector.clone();
                threads.push(thread::spawn(move || {
                    let mut rng = rand::thread_rng();  

                    for _ in 0..num_iterations {
                        if rng.gen_bool(read_percentage) {
                            // Read a random index.
                            let read = shared_vector.read();
                            if read.len() > 0 {
                                let index = rng.gen_range(0..read.len());
                                black_box(assert_eq!(read[index], 5));
                            }
                        } else {
                            // Add a new vector element.
                            shared_vector.write().push(5);
                        }
                    }
                
                }));
            }

            // Check whether threads have completed succesfully.
            for thread in threads {
                thread.join().unwrap();
            }
        })
    });

    c.bench_function("tokio RwLock 20", |bencher| {
        
        bencher.iter(|| {  
            let shared_vector = Arc::new(tokio::sync::RwLock::<Vec::<i32>>::new(vec![]));

            let mut tasks = tokio::task::JoinSet::new();
            for _ in 1..num_threads {
                let shared_vector = shared_vector.clone();
                tasks.spawn(async move {
                    for _ in 0..num_iterations {
                        let random = {
                            let mut rng = rand::thread_rng();
                            rng.gen_bool(read_percentage)
                        };

                        if random {
                            // Read a random index.
                            let read = shared_vector.read().await;
                            if read.len() > 0 {
                                let mut rng = rand::thread_rng();  
                                let index = rng.gen_range(0..read.len());
                                black_box(assert_eq!(read[index], 5));
                            }
                        } else {
                            // Add a new vector element.
                            shared_vector.write().await.push(5);
                        }
                    }
                });
            };

            async move {            
                while let Some(_result) = tasks.join_next().await {
                    // Ignore the results.
                }
            }
        })
    });

}

criterion_group!(
    benches,
    benchmark_sharedmutexes,
);
criterion_main!(benches);