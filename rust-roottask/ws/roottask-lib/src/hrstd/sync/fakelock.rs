use core::cell::UnsafeCell;

/// A fake lock which helps to signal Rust memory safety on global static mutable vars.
/// **This should only be used during the boot process as long as only a single core
/// (the boot processor) is used**.
///
/// Even tho we could use a real mutex, this makes it simpler to cope with nested locking, e.g.
/// the panic handler needs access to a variable, but the panic'ed code still holds the lock.
#[derive(Debug)]
pub struct FakeLock<T> {
    data: UnsafeCell<T>,
}

// tell Rust this is safe - use with caution!
unsafe impl<T> Send for FakeLock<T> {}
unsafe impl<T> Sync for FakeLock<T> {}

impl<T> FakeLock<T> {
    /// Creates a new lock. Constant function, can be used in global statics.
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    /// Returns read only reference to the data.
    pub fn get(&self) -> &T {
        unsafe { &*self.data.get() }
    }

    /// Returns mutable reference to the data.
    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static GLOBAL_TEST: FakeLock<String> = FakeLock::new(String::new());

    #[test]
    fn test_boot_lock() {
        fn use_static_str(_static_str: &'static str) {}
        assert_eq!("", GLOBAL_TEST.get());
        GLOBAL_TEST.get_mut().push_str("Moin");
        assert_eq!("Moin", GLOBAL_TEST.get());
        use_static_str(GLOBAL_TEST.get().as_str())
    }
}
