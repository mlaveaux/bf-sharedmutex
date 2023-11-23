use std::cmp::max;
use std::{alloc::Layout, ptr, sync::atomic::AtomicUsize};
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
    /// Create a new vector with zero capacity.
    pub fn new() -> BfVec<T> { 

        BfVec {
            shared: BfSharedMutex::new(BfVecShared {
                buffer: ptr::null_mut(),
                capacity: 0,
                len: AtomicUsize::new(0),
            }),
        }
    }

    pub fn push(&self, value: T) {
        let mut read = self.shared.read().unwrap();
        
        // Reserve an index for the new element.
        let last_index = read.len.fetch_add(1, std::sync::atomic::Ordering::Release);

        // Vector needs to be resized.
        if last_index >= read.capacity {
            let new_capacity = max(read.capacity * 2, 8);
            drop(read);
            self.reserve(new_capacity);
            read = self.shared.read().unwrap();
        }

        // Write the element on the specified index.
        unsafe {
            let end = read.buffer.add(last_index);
            ptr::write(end, value);
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

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Drops the elements in the Vec, but keeps the capacity.
    pub fn clear(&self) {
        let mut write = self.shared.write().unwrap();
        write.clear();
    }

    /// Reserve the given capacity.
    fn reserve(&self, capacity: usize) {
        let mut write = self.shared.write().unwrap();

        // A reserve could have happened in the meantime which makes this call obsolete
        if capacity <= write.capacity {
            return;
        }

        let old_layout = Layout::array::<T>(write.capacity).unwrap();
        let layout = Layout::array::<T>(capacity).unwrap();

        unsafe {
            let new_buffer = alloc::alloc(layout) as *mut T;            
            if new_buffer.is_null() {
                alloc::handle_alloc_error(layout); 
            }

            if !write.buffer.is_null() {
                ptr::copy_nonoverlapping(
                    write.buffer,
                    new_buffer,
                    write.len.load(Ordering::Relaxed),
                );            
    
                // Clean up the old buffer.
                alloc::dealloc(write.buffer as *mut u8, old_layout);
            }

            write.capacity = capacity;
            write.buffer = new_buffer;
        }
    }
}

impl<T> Default for BfVec<T> {
    fn default() -> Self {
        Self::new()
    }
}


impl<T> BfVecShared<T> {
     
    pub fn clear(&mut self) {
        // Only drop items within the 0..len range since the other values are not initialised.
        for i in 0..self.len.load(Ordering::Relaxed) {
            unsafe {
                // We have exclusive access so dropping is safe.
                let ptr = self.buffer.add(i);

                ptr::drop_in_place(ptr);
            }
        }
    }
}

impl<T> Drop for BfVecShared<T> {
    fn drop(&mut self) {
        self.clear();

        unsafe {
            // Deallocate the underlying storage.
            let layout = Layout::array::<T>(self.capacity).unwrap();
            alloc::dealloc(self.buffer as *mut u8, layout);
        }
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

        let shared_vector = BfVec::<u32>::new();
        let num_threads = 10;
        let num_iterations = 100000;

        for _ in 0..num_threads {
            let shared_vector = shared_vector.share();
            threads.push(thread::spawn(move || {
                for _ in 0..num_iterations {
                    shared_vector.push(1);
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
