use std::{hint::black_box, sync::Arc};

// This is a struct that tells Criterion.rs to use the "futures" crate's current-thread executor
use criterion::Criterion;

use rand::prelude::*;
use tokio::runtime::Runtime;

pub fn benchmark_async(c: &mut Criterion) {
    let read_percentage = 0.99;
    let num_threads = 20;
    let num_iterations = 100000;

    c.bench_function(&format!("tokio::sync::RwLock {} {} {}", num_threads, num_iterations, read_percentage), |bencher| {
        
        bencher.iter(|| {  
            let rt = Runtime::new().unwrap();
            let shared_vector = Arc::new(tokio::sync::RwLock::<Vec::<i32>>::new(vec![]));

            let mut tasks = vec![];
            for _ in 1..num_threads {
                let shared_vector = shared_vector.clone();
                tasks.push(rt.spawn(async move {
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
