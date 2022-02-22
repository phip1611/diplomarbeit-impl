//! Module for [`Qpd`].

use crate::consts::NUM_PRIORITIES;
use crate::ipc_serde::__private::Formatter;
use core::fmt::Debug;

/// Quantum Priority Descriptor (QPD).
#[derive(Copy, Clone)]
pub struct Qpd(u64);

impl Qpd {
    /// Default time quantum is 10ms (10000µs).
    pub const DEFAULT_QUANTUM: u64 = 10_000;
    const QUANTUM_BITSHIFT: u64 = 12;
    const PRIORITY_BITMASK: u64 = 0xff;

    /// Creates a new object.
    ///
    /// # Parameters
    /// * `priority` Priority between 1 and 128
    /// * `preferred_quantum` time quantum in microseconds. If None, it falls back to [`Self::DEFAULT_QUANTUM`].
    pub fn new(priority: u64, preferred_quantum: Option<u64>) -> Self {
        assert!(priority > 0, "priority must be bigger than 0");
        assert!(
            priority <= NUM_PRIORITIES as u64,
            "priority must be lessequal to {}",
            NUM_PRIORITIES
        );
        if let Some(quantum) = preferred_quantum {
            assert!(quantum > 0, "quantum must be bigger than 0");
        }
        let quantum = preferred_quantum.unwrap_or(Self::DEFAULT_QUANTUM);
        let mut val = 0;
        val |= priority & Self::PRIORITY_BITMASK;
        val |= quantum << Self::QUANTUM_BITSHIFT;
        Self(val)
    }

    pub const fn quantum(self) -> u64 {
        self.0 >> Self::QUANTUM_BITSHIFT
    }

    pub const fn priority(self) -> u64 {
        self.0 & Self::PRIORITY_BITMASK
    }

    pub const fn val(self) -> u64 {
        self.0
    }
}

impl Debug for Qpd {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Qpd")
            .field("quantum", &self.quantum())
            .field("priority", &self.priority())
            .field("val", &self.val())
            .finish()
    }
}
