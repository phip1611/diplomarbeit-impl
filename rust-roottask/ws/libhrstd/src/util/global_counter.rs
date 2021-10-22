//! Module for [`GlobalUniqueIncrementingCounter`].

use core::sync::atomic::{
    AtomicU64,
    Ordering,
};

/// Atomic, incrementing counter with unique values to identify entities.
#[derive(Debug)]
pub struct GlobalIncrementingCounter {
    counter: AtomicU64,
}

impl GlobalIncrementingCounter {
    /// New counter.
    pub const fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
        }
    }

    /// Gets the next value and automatically increments the internal value.
    pub fn next(&self) -> u64 {
        let val = self.counter.load(Ordering::SeqCst);
        self.counter.store(val + 1, Ordering::SeqCst);
        val
    }
}
