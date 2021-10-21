//! Constants specific for Hedron.
//! See "config.hpp" in Hedron for reference.

/// Number of supported CPUs.
pub const NUM_CPUS: usize = 64;

pub const NUM_PRIORITIES: usize = 128;

pub const NUM_IOAPICS: usize = 9;

/// Number of exceptions for global ECs.
pub const NUM_EXC: usize = 32;

/// Number of exceptions for vCPUs.
pub const NUM_VM_EXC: usize = 256;
