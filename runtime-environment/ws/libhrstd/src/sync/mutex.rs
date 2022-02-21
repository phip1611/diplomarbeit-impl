use core::cell::UnsafeCell;
use core::ops::{
    Deref,
    DerefMut,
};
use core::sync::atomic::{
    compiler_fence,
    AtomicBool,
    Ordering,
};

const UNLOCKED: bool = false;
const LOCKED: bool = true;

/// A simple mutex. The core library doesn't have this, therefore I have to build
/// it by myself.
#[derive(Debug)]
pub struct SimpleMutex<T> {
    data: UnsafeCell<T>,
    lock: AtomicBool,
}

// TODO fix: <T: Send>  instead of <T>, otherwise Rc can be shared
unsafe impl<T> Send for SimpleMutex<T> {}
unsafe impl<T> Sync for SimpleMutex<T> {}

impl<T> SimpleMutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            lock: AtomicBool::new(UNLOCKED),
        }
    }

    pub fn into_inner(self) -> T {
        if self.lock.load(Ordering::SeqCst) == LOCKED {
            panic!("Still in use!");
        }
        self.data.into_inner()
    }

    pub fn lock(&self) -> SimpleMutexGuard<T> {
        loop {
            let lock_obtained =
                self.lock
                    .compare_exchange(UNLOCKED, LOCKED, Ordering::SeqCst, Ordering::SeqCst);
            if lock_obtained.is_ok() {
                break;
            }
        }
        SimpleMutexGuard { lock: &self }
    }
}

impl<T: Default> Default for SimpleMutex<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

#[derive(Debug)]
pub struct SimpleMutexGuard<'a, T> {
    lock: &'a SimpleMutex<T>,
}

impl<'a, T> SimpleMutexGuard<'a, T> {
    /// This method is convenient, when you want to execute code while the lock is held
    /// and the lock doesn't hold the data. This is useful for advisory locks, like
    /// `SimpleMutex<()>`.
    pub fn execute_while_locked<U, R>(&self, actions: U) -> R
    where
        U: FnOnce() -> R,
    {
        compiler_fence(Ordering::SeqCst);
        let res = actions();
        compiler_fence(Ordering::SeqCst);
        res
    }
}

impl<T> Deref for SimpleMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SimpleMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SimpleMutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.lock.lock.store(UNLOCKED, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore] // long running..
    #[test]
    fn test_simple_mutex() {
        let std_mutex = std::sync::Mutex::new(0);
        let my_mutex = SimpleMutex::new(0);

        for _i in 0..1_000_000 {
            let mut std_lock = std_mutex.lock().unwrap();
            let mut my_lock = my_mutex.lock();

            *std_lock = *std_lock + 1;
            *my_lock = *my_lock + 1;
        }

        assert_eq!(1_000_000, *std_mutex.lock().unwrap());
        assert_eq!(1_000_000, *my_mutex.lock());
    }
}
