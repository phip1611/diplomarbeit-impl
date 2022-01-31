use crate::time::Duration;
use core::ops::Sub;

/// Wrapper around `rdtscp` to measure performance
/// in clock ticks. Currently, there is no way to convert
/// this to wall clock time.
#[derive(Debug)]
pub struct Instant {
    begin_time: u64,
}

impl Instant {
    pub fn now() -> Self {
        Self {
            begin_time: unsafe { x86::time::rdtscp() },
        }
    }

    /// Returns the value retrieved from `rdtscp`.
    pub const fn val(&self) -> u64 {
        self.begin_time
    }
}

impl Sub for Instant {
    type Output = Duration;

    fn sub(self, rhs: Instant) -> Self::Output {
        self.val() - rhs.val()
    }
}
