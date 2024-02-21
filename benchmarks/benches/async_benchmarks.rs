use std::{hint::black_box, sync::Arc};

// This is a struct that tells Criterion.rs to use the "futures" crate's current-thread executor
use criterion::Criterion;

use rand::{prelude::*, distributions::Bernoulli};
use tokio::runtime::Runtime;

use benchmarks::{THREADS, READ_RATIOS, NUM_ITERATIONS};

pub fn benchmark_async(c: &mut Criterion) {
    
    for num_threads in THREADS {
        for read_ratio in READ_RATIOS {
            c.bench_function(&format!("tokio::sync::RwLock {} {} {}", num_threads, NUM_ITERATIONS, read_ratio), |bencher| {
                
                bencher.iter(|| {  
                    let rt = Runtime::new().unwrap();
                    let shared_vector = Arc::new(tokio::sync::RwLock::<Vec::<i32>>::new(vec![]));
                    let dist = Bernoulli::from_ratio(1, read_ratio).unwrap();

                    let mut tasks = vec![];
                    for _ in 1..num_threads {
                        let shared_vector = shared_vector.clone();
                        tasks.push(rt.spawn(async move {
                            for _ in 0..NUM_ITERATIONS {
                                let random = {
                                    let mut rng = rand::thread_rng();
                                    dist.sample(&mut rng)
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
                        }));
                    };

                    // Wait for tasks to complete.
                    rt.block_on(async move { 
                        for task in tasks {
                            task.await.unwrap()
                        }
                    });
                })
            });
        }
    }
}
