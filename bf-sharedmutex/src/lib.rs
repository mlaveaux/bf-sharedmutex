pub mod bf_sharedmutex;

pub use bf_sharedmutex::*;

// trait SharedMutex<T> {
//     type Guard : DerefMut<Target = T>;

//     /// Locks the shared mutex for readers (shared access)
//     fn read(&self) -> &T;

//     /// Locks the shared mutex for a writer (exclusive access)
//     fn write<'a>(&'a self) -> Self::Guard<'a>;
// }