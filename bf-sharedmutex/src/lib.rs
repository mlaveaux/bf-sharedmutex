pub mod bf_sharedmutex;

pub use bf_sharedmutex::*;

// trait SharedMutex<T> {
//     type Guard : DerefMut<Target = T>;

//     /// Locks the shared mutex for readers (shared access)
//     fn read(&self) -> &T;

//     /// Locks the shared mutex for a writer (exclusive access)
//     fn write<'a>(&'a self) -> Self::Guard<'a>;
// }

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use std::{thread, hint::black_box};
    use rand::prelude::*;

    use crate::bf_sharedmutex::BfSharedMutex;

    // These are just simple tests.
    #[test]
    fn exclusive_test() {
        
        let mut threads = vec![];

        let shared_number = BfSharedMutex::new(5);

        for _ in 1..300 {
            let shared_number = shared_number.clone();
            threads.push(thread::spawn(move || {
                *shared_number.write().unwrap() += 5;                
            }));
        }

        // Check whether threads have completed succesfully.
        for thread in threads {
            thread.join().unwrap();
        }

        assert_eq!(*shared_number.write().unwrap(), 300*5);
    }

    #[test]
    fn shared_test() {
        let shared_vector = BfSharedMutex::new(vec![]);

        let mut threads = vec![];
        for _ in 1..20 {
            let shared_vector = shared_vector.clone();
            threads.push(thread::spawn(move || {
                let mut rng = rand::thread_rng();  

                for _ in 0..100000 {
                    if rng.gen_bool(0.95) {
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
    }
}

#[cfg(test)]
#[cfg(loom)]
mod loom_tests{
    use loom::thread;
    
    use crate::bf_sharedmutex::BfSharedMutex;

    // This is a forced interleaving test using Loom
    #[test]
    fn test_loom() {
        loom::model(|| {
            let shared_vector = BfSharedMutex::new(vec![]);

            let mut threads = vec![];
            for _ in 1..20 {
                let shared_vector = shared_vector.clone();
                threads.push(thread::spawn(move || {
                    let mut rng = rand::thread_rng();  
    
                    for _ in 0..100 {
                        // Read a random index.
                        let read = shared_vector.read().unwrap();
                        if read.len() > 0 {
                            let index = rng.gen_range(0..read.len());
                            black_box(assert_eq!(read[index], 5));
                        }

                        // Add a new vector element.
                        shared_vector.write().unwrap().push(5);
                    }
                 
                }));
            }
    
            // Check whether threads have completed succesfully.
            for thread in threads {
                thread.join().unwrap();
            }
        });
    }
}