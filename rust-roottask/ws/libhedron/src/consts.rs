//! Constants specific for Hedron.
//! See "config.hpp" in Hedron for reference.

use crate::capability::CapSel;

/// Number of supported CPUs.
pub const NUM_CPUS: usize = 64;

pub const NUM_PRIORITIES: usize = 128;

pub const NUM_IOAPICS: usize = 9;

/// Number of exceptions for global ECs.
pub const NUM_EXC: usize = 32;

/// Number of exceptions for vCPUs.
pub const NUM_VM_EXC: usize = 256;

/// Maximum 2^52 = 4503599627370496.
pub const NUM_CAP_SEL: CapSel = 0x0010_0000_0000_0000_u64;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_num_cap_sel() {
        assert_eq!(NUM_CAP_SEL, 4503599627370496);
    }
}
