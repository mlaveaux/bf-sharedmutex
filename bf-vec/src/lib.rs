use std::{alloc::Layout, mem, ptr, sync::atomic::AtomicUsize};
use std::sync::atomic::Ordering;

use bf_sharedmutex::BfSharedMutex;
use std::alloc;

pub struct BfVecShared<T> {
    buffer: *mut T,
    capacity: usize,
    len: AtomicUsize,
}

/// An implementation of [Vec<T, A>] based on the [BfSharedMutex] implementation
/// that can be safely send between threads. Elements in the vector can be written
/// concurrently iff type T is [Sync].
pub struct BfVec<T> {
    shared: BfSharedMutex<BfVecShared<T>>,
}

impl<T> BfVec<T> {
    /// Create a new vector with 8 initial capacity.
    pub fn new() -> BfVec<T> { 

        let initial_size = 8;

        let layout =
            Layout::from_size_align(initial_size, mem::align_of::<T>()).expect("Bad layout");

        let new_buffer = unsafe {
            alloc::alloc(layout) as *mut T
        };

        BfVec {
            shared: BfSharedMutex::new(BfVecShared {
                buffer: new_buffer,
                capacity: initial_size,
                len: AtomicUsize::new(0),
            }),
        }
    }

    pub fn push(&self, value: T) {
        let read = self.shared.read().unwrap();
        
        // Insert the element and update the length.
        unsafe {
            let end = read.buffer.add(read.len.load(Ordering::Relaxed));
            ptr::write(end, value);
        }

        read.len.fetch_add(1, std::sync::atomic::Ordering::Release);

        // Vector needs to be resized. We do this prematurely because otherwise
        // we would need to upgrade the read into a write lock, or acquire read
        // access twice.
        if read.len.load(std::sync::atomic::Ordering::Acquire) == read.capacity {
            let new_capacity = read.capacity * 2;
            drop(read);
            self.reserve(new_capacity);
        }
    }

    /// Obtain another view on the vector to share among threads.
    pub fn share(&self) -> BfVec<T> {
        BfVec {
            shared: self.shared.clone()
        }
    }

    /// Obtain the number of elements in the vector.
    pub fn len(&self) -> usize {
        self.shared.read().unwrap().len.load(Ordering::Relaxed)
    }

    /// Allocate the vector to be twice as long.
    fn reserve(&self, capacity: usize) {
        let mut write = self.shared.write().unwrap();

        let layout =
            Layout::from_size_align(capacity, mem::align_of::<T>()).expect("Bad layout");

        unsafe {
            let new_buffer = alloc::alloc(layout) as *mut T;

            ptr::copy_nonoverlapping(
                write.buffer,
                new_buffer,
                write.len.load(Ordering::Relaxed),
            );

            write.capacity = capacity;
            write.buffer = new_buffer;
        }
    }
}

impl<T> Drop for BfVec<T> {
    fn drop(&mut self) {
        // Only drop items within the 0..len range since the other values are not initialised.

    }
}

unsafe impl<T> Send for BfVec<T> {}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use std::thread;

    use super::*;

    // These are just simple tests.
    #[test]
    fn test_push() {
        let mut threads = vec![];

        let shared_vector = BfVec::<()>::new();
        let num_threads = 20;
        let num_iterations = 5000;

        for _ in 0..num_threads {
            let shared_vector = shared_vector.share();
            threads.push(thread::spawn(move || {
                for _ in 0..num_iterations {
                    shared_vector.push(());
                }
            }));
        }

        // Check whether threads have completed succesfully.
        for thread in threads {
            thread.join().unwrap();
        }

        assert_eq!(shared_vector.len(), num_threads * num_iterations);
    }
}
