use std::{
    sync::{atomic::{AtomicBool, Ordering}, Arc}, ops::{DerefMut, Deref}, fmt::Debug, error::Error,
};

#[cfg(not(loom))]
use std::{sync::{Mutex, MutexGuard}, cell::UnsafeCell};

#[cfg(loom)]
use loom::{sync::{Mutex, MutexGuard}, cell::UnsafeCell};

/// A shared mutex (readers-writer lock) implementation based on the so-called
/// busy-forbidden protocol. Instead of a regular Mutex this class is Send and
/// not Sync, every thread must acquire a clone of the shared mutex and the
/// cloned instances of the same shared mutex guarantee shared access through
/// the `read` operation and exclusive access for the `write` operation of the
/// given object.
pub struct BfSharedMutex<T> {
    /// The local control bits of each instance. TODO: Maybe use pin to share the control bits among shared mutexes.
    control: Arc<SharedMutexControl>,

    /// The object that is being protected.
    object: Arc<UnsafeCell<T>>,

    /// Index into the `other` table.
    index: usize,

    /// The list of all the shared mutex instances.
    other: Arc<Mutex<Vec<Option<Arc<SharedMutexControl>>>>>,
}

// Can only be send, but is not sync
unsafe impl<T> Send for BfSharedMutex<T> {}

/// The busy and forbidden flags used to 
#[repr(align(64))]
struct SharedMutexControl {
    busy: AtomicBool,
    forbidden: AtomicBool,
}

impl<T> BfSharedMutex<T> {

    /// Constructs a new shared mutex for protecting access to the given object.
    pub fn new(object: T) -> Self {
        let control = Arc::new(SharedMutexControl {
            busy: false.into(),
            forbidden: false.into(),
        });

        Self {
            control: control.clone(),
            object: Arc::new(UnsafeCell::new(object)),
            index: 0,
            other: Arc::new(Mutex::new(vec![Some(control.clone())])),
        }
    }
}

impl<T> Clone for BfSharedMutex<T> {
    fn clone(&self) -> Self {

        // Register a new instance in the other list.
        let control = Arc::new(SharedMutexControl {
            busy: false.into(),
            forbidden: false.into(),
        });

        let mut other = self.other.lock().expect("Failed to lock mutex");
        other.push(Some(control.clone()));

        Self {
            control,
            index: other.len() - 1,
            object: self.object.clone(),
            other: self.other.clone(),
        }
    }
}

impl<T> Drop for BfSharedMutex<T> {
    fn drop(&mut self) {
        let mut other = self.other.lock().expect("Failed to lock mutex");

        // Remove ourselves from the table.
        other[self.index] = None;
    }
}

/// The guard object for exclusive access to the underlying object.
pub struct BfSharedMutexWriteGuard<'a, T> {
    mutex: &'a BfSharedMutex<T>,
    guard: MutexGuard<'a, Vec<Option<Arc<SharedMutexControl>>>>,
}

/// Allow dereferencing the underlying object.
#[cfg(not(loom))]
impl<'a, T> Deref for BfSharedMutexWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // We are the only guard after `write()`, so we can provide immutable access to the underlying object. (No mutable references the guard can exist)
        unsafe { &*self.mutex.object.get() }
    }
}

#[cfg(loom)]
impl<'a, T> Deref for BfSharedMutexWriteGuard<'a, T> {
    type Target = loom::ConstPtr;

    fn deref(&self) -> &Self::Target {
        // We are the only guard after `write()`, so we can provide immutable access to the underlying object. (No mutable references the guard can exist)
        unsafe { &*self.mutex.object.get() }
    }
}

impl<'a, T> DerefMut for BfSharedMutexWriteGuard<'a, T> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        // We are the only guard after `write()`, so we can provide mutable access to the underlying object.
        unsafe { &mut *self.mutex.object.get() }
    }
}

impl<'a, T> Drop for BfSharedMutexWriteGuard<'a, T> {
    fn drop(&mut self) {

        // Allow other threads to acquire access to the shared mutex.        
        for control in self.guard.iter().flatten() {
            control.forbidden.store(false, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

pub struct BfSharedMutexReadGuard<'a, T> {
    mutex: &'a BfSharedMutex<T>,
}

/// Allow dereferences the underlying object.
impl<'a, T> Deref for BfSharedMutexReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // There can only be shared guards, which only provide immutable access to the object.
        unsafe { &*self.mutex.object.get() }
    }
}

impl<'a, T> Drop for BfSharedMutexReadGuard<'a, T> {
    fn drop(&mut self) {
        debug_assert!(self.mutex.control.busy.load(Ordering::SeqCst), "Cannot unlock shared lock that was not acquired");

        self.mutex.control.busy.store(false, Ordering::SeqCst);
    }
}

impl<T> BfSharedMutex<T> {

    /// Provides read access to the underlying object, allowing multiple immutable references to it.
    pub fn read<'a>(&'a self) -> Result<BfSharedMutexReadGuard<'a, T>, Box<dyn Error + 'a>> {
        debug_assert!(!self.control.busy.load(Ordering::SeqCst), "Cannot acquire read() access twice");

        self.control.busy.store(true, Ordering::SeqCst);

        while self.control.forbidden.load(Ordering::SeqCst) {
            self.control.busy.store(false, Ordering::SeqCst);

            // The mutex guard is dropped, but this is only for synchronisation purposes.
            let _unused = self.other.lock()?;
            self.control.busy.store(true, Ordering::SeqCst);
        }

        // We now have immutable access to the object due to the protocol.
        Ok(BfSharedMutexReadGuard {
            mutex: self,
        })
    }

    /// Provide write access to the underlying object, only a single mutable reference to the object exists.
    pub fn write<'a>(&'a self) -> Result<BfSharedMutexWriteGuard<'a, T>, Box<dyn Error + 'a>> {

        let other = self.other.lock()?;

        debug_assert!(!self.control.busy.load(std::sync::atomic::Ordering::Relaxed), 
            "Can only exclusive lock outside of a shared lock, no upgrading!");
        debug_assert!(!self.control.forbidden.load(std::sync::atomic::Ordering::Relaxed), 
            "Can not acquire exclusive lock inside of exclusive section");

        // Make all instances wait due to forbidden access.
        for control in other.iter().flatten() {
            debug_assert!(!control.forbidden.load(std::sync::atomic::Ordering::Relaxed), 
                "Other instance is already forbidden, this cannot happen");

            control.forbidden.store(true, std::sync::atomic::Ordering::SeqCst);
        }

        // Wait for the instances to exit their busy status.
        for (index, option) in other.iter().enumerate() {
            if index != self.index {

                if let Some(object) = option {
                    while object.busy.load(std::sync::atomic::Ordering::SeqCst) { std::hint::spin_loop(); }
                }
            }            
        }

        // We now have exclusive access to the object according to the protocol
        Ok(BfSharedMutexWriteGuard {
            mutex: self,
            guard: other,
        })
    }
}

impl<T: Debug> Debug for BfSharedMutex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        f.debug_map().entry(&"busy", &self.control.busy.load(Ordering::Relaxed))
        .entry(&"forbidden", &self.control.forbidden.load(Ordering::Relaxed))
        .entry(&"index", &self.index)
        .entry(&"len(other)", &self.other.lock().unwrap().len())
        .finish()?;

        writeln!(f)?;
        writeln!(f, "other values: [")?;
        for control in self.other.lock().unwrap().iter().flatten() {
            f.debug_map().entry(&"busy", &control.busy.load(Ordering::Relaxed))
            .entry(&"forbidden", &control.forbidden.load(Ordering::Relaxed))
            .finish()?;
            writeln!(f)?;
        }

        writeln!(f, "]")
    }
}
