//! Module for [`Qpd`].

use crate::consts::NUM_PRIORITIES;
use crate::ipc_serde::__private::Formatter;
use core::fmt::Debug;

/// Quantum Priority Descriptor (QPD).
#[derive(Copy, Clone)]
pub struct Qpd(u64);

impl Qpd {
    const QUANTUM_BITSHIFT: usize = 12;
    const PRIORITY_BITMASK: usize = 0xff;

    /// Creates a new object.
    ///
    /// # Parameters
    /// * `priority` Priority between 1 and 128
    /// * `quantum` time quantum in microseconds
    pub fn new(priority: u64, quantum: u64) -> Self {
        assert!(priority > 0, "priority must be bigger than 0");
        assert!(
            priority <= NUM_PRIORITIES as u64,
            "priority must be lessequal to {}",
            NUM_PRIORITIES
        );
        assert!(quantum > 0, "quantum must be bigger than 0");
        let mut val = 0;
        val |= priority & Self::PRIORITY_BITMASK as u64;
        val |= quantum << Self::QUANTUM_BITSHIFT as u64;
        Self(val)
    }

    pub const fn quantum(self) -> u64 {
        self.0 >> Self::QUANTUM_BITSHIFT as u64
    }

    pub const fn priority(self) -> u64 {
        self.0 & Self::PRIORITY_BITMASK as u64
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
