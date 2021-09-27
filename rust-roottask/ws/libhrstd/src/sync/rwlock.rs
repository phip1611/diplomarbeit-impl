use super::mutex::SimpleMutex;
use core::cell::UnsafeCell;
use core::ops::{
    Deref,
    DerefMut,
};
use core::sync::atomic::{
    AtomicU64,
    Ordering,
};

/// A simple read write lock. Allows either n readers or one writer.
#[derive(Debug)]
pub struct SimpleRwLock<T> {
    data: UnsafeCell<T>,
    critical_section: SimpleMutex<()>,
    write_count: AtomicU64,
    read_count: AtomicU64,
}

unsafe impl<T> Send for SimpleRwLock<T> {}
unsafe impl<T> Sync for SimpleRwLock<T> {}

impl<T> SimpleRwLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            critical_section: SimpleMutex::new(()),
            read_count: AtomicU64::new(0),
            write_count: AtomicU64::new(0),
        }
    }

    /*pub fn into_inner(self) -> T {
        if self.lock.load(Ordering::SeqCst) == LOCKED {
            panic!("Still in use!");
        }
        self.data.into_inner()
    }*/

    pub fn try_lock_read(&self) -> Result<SimpleRwLockReadGuard<T>, ()> {
        let lock = self.critical_section.lock();
        lock.execute_while_locked(&|| {
            if self.can_read() {
                Ok(SimpleRwLockReadGuard::new(self))
            } else {
                Err(())
            }
        })
    }

    pub fn try_lock_write(&self) -> Result<SimpleRwLockWriteGuard<T>, ()> {
        let lock = self.critical_section.lock();
        lock.execute_while_locked(&|| {
            if self.can_write() {
                Ok(SimpleRwLockWriteGuard::new(self))
            } else {
                Err(())
            }
        })
    }

    pub fn lock_read(&self) -> SimpleRwLockReadGuard<T> {
        loop {
            if let Ok(l) = self.try_lock_read() {
                return l;
            }
        }
    }

    pub fn lock_write(&self) -> SimpleRwLockWriteGuard<T> {
        loop {
            if let Ok(l) = self.try_lock_write() {
                return l;
            }
        }
    }

    /// NOTE THAT THIS IS JUST A SNAPSHOT DURING THE FUNCTION CALL! During the time you call
    /// "lock_write" already everything can be changed! This is useful for testing.
    fn can_write(&self) -> bool {
        self.read_count.load(Ordering::SeqCst) == 0 && self.write_count.load(Ordering::SeqCst) == 0
    }

    fn can_read(&self) -> bool {
        self.write_count.load(Ordering::SeqCst) == 0
    }
}

#[derive(Debug)]
pub struct SimpleRwLockWriteGuard<'a, T> {
    lock: &'a SimpleRwLock<T>,
}

impl<'a, T> SimpleRwLockWriteGuard<'a, T> {
    fn new(lock: &'a SimpleRwLock<T>) -> Self {
        lock.write_count.fetch_add(1, Ordering::SeqCst);
        Self { lock }
    }
}

impl<T> Deref for SimpleRwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SimpleRwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SimpleRwLockWriteGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.lock.write_count.fetch_sub(1, Ordering::SeqCst);
    }
}

#[derive(Debug)]
pub struct SimpleRwLockReadGuard<'a, T> {
    lock: &'a SimpleRwLock<T>,
}

impl<'a, T> SimpleRwLockReadGuard<'a, T> {
    fn new(lock: &'a SimpleRwLock<T>) -> Self {
        lock.read_count.fetch_add(1, Ordering::SeqCst);
        Self { lock }
    }
}

impl<T> Deref for SimpleRwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> Drop for SimpleRwLockReadGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.lock.read_count.fetch_sub(1, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::sync::Arc;
    use std::thread::spawn;
    use std::vec::Vec;

    #[test]
    fn test_rw_lock() {
        let rw_lock = Arc::new(SimpleRwLock::new(0_u64));
        {
            let _lock = rw_lock.lock_read();
        }
        {
            let _lock = rw_lock.lock_write();
        }
    }

    #[ignore] // long running..
    #[test]
    fn test_rw_lock2() {
        // 100 runs
        for _ in 0..100 {
            let rw_lock = Arc::new(SimpleRwLock::new(0_u64));
            let mut t_handles = Vec::new();

            // create 15 reader and 15 writer threads
            for _ in 0..15 {
                let rw_lock_t = rw_lock.clone();
                let h = spawn(move || {
                    for _ in 0..10_000 {
                        let lock = rw_lock_t.lock_read();
                        let _foo = *lock;
                    }
                });
                t_handles.push(h);

                let rw_lock_t = rw_lock.clone();
                let h = spawn(move || {
                    for _ in 0..10_000 {
                        let mut lock = rw_lock_t.lock_write();
                        *lock += 1;
                    }
                });
                t_handles.push(h);
            }

            let _ = t_handles
                .into_iter()
                .map(|x| x.join().unwrap())
                .collect::<Vec<_>>();

            assert_eq!(15 * 10_000, *rw_lock.lock_read());
        }
    }
}
