use std::{hint::black_box, thread, sync::{RwLock, Arc}};

use criterion::Criterion;

use rand::prelude::*;
use bf_sharedmutex::BfSharedMutex;
    
pub fn benchmark_sharedmutexes(c: &mut Criterion) {

    let read_percentage = 0.99;
    let num_threads = 20;
    let num_iterations = 100000;

    // This executes the same code sequentially for comparison of total time.
    c.bench_function(&format!("sequential {} {} {}", num_threads, num_iterations, read_percentage), |bencher| {

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

    c.bench_function(&format!("bf-sharedmutex::BfSharedMutex {} {} {}", num_threads, num_iterations, read_percentage), |bencher| {

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

    c.bench_function(&format!("std::sync::RwLock {} {} {}", num_threads, num_iterations, read_percentage), |bencher| {
        
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

    c.bench_function(&format!("parking_lot::RwLock {} {} {}", num_threads, num_iterations, read_percentage), |bencher| {
        
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

    c.bench_function(&format!("spin::RwLock {} {} {}", num_threads, num_iterations, read_percentage), |bencher| {
        
        bencher.iter(|| {  
            let shared_vector = Arc::new(spin::RwLock::new(vec![]));

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
    
    c.bench_function(&format!("widerwlock::WideRwLock {} {} {}", num_threads, num_iterations, read_percentage), |bencher| {
        
        bencher.iter(|| {  
            let shared_vector = Arc::new(widerwlock::WideRwLock::new(vec![]));

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
    
    c.bench_function(&format!("shared_mutex::SharedMutex {} {} {}", num_threads, num_iterations, read_percentage), |bencher| {
        
        bencher.iter(|| {  
            let shared_vector = Arc::new(shared_mutex::SharedMutex::new(vec![]));

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
    
    c.bench_function(&format!("pairlock::RwLock {} {} {}", num_threads, num_iterations, read_percentage), |bencher| {
        
        bencher.iter(|| {  
            let shared_vector = Arc::new(pairlock::PairLock::with_default(vec![]));

            let mut threads = vec![];
            for _ in 1..num_threads {
                let shared_vector = shared_vector.clone();
                threads.push(thread::spawn(move || {
                    let mut rng = rand::thread_rng();  

                    for _ in 0..num_iterations {
                        if rng.gen_bool(read_percentage) {
                            // Read a random index.
                            shared_vector.view(|read| {
                                if read.len() > 0 {
                                    let index = rng.gen_range(0..read.len());
                                    black_box(assert_eq!(read[index], 5));
                                }

                            });
                        } else {
                            // Add a new vector element.
                            shared_vector.update().push(5);
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


    c.bench_function(&format!("pflock::PFLock {} {} {}", num_threads, num_iterations, read_percentage), |bencher| {
        
        bencher.iter(|| {  
            let shared_vector = Arc::new(pflock::PFLock::new(vec![]));

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

    // c.bench_function(&format!("process_sync::SharedMemoryObject {} {} {}", num_threads, num_iterations, read_percentage), |bencher| {
        
    //     bencher.iter(|| {  
    //         let shared_vector = Arc::new(process_sync::SharedMemoryObject::new(vec![]));

    //         let mut threads = vec![];
    //         for _ in 1..num_threads {
    //             let shared_vector = shared_vector.clone();
    //             threads.push(thread::spawn(move || {
    //                 let mut rng = rand::thread_rng();  

    //                 for _ in 0..num_iterations {
    //                     if rng.gen_bool(read_percentage) {
    //                         // Read a random index.
    //                         let read = shared_vector.get().unwrap();
    //                         if read.len() > 0 {
    //                             let index = rng.gen_range(0..read.len());
    //                             black_box(assert_eq!(read[index], 5));
    //                         }
    //                     } else {
    //                         // Add a new vector element.
    //                         shared_vector.get_mut().push(5);
    //                     }
    //                 }
                
    //             }));
    //         }

    //         // Check whether threads have completed succesfully.
    //         for thread in threads {
    //             thread.join().unwrap();
    //         }
    //     })
    // });

}
