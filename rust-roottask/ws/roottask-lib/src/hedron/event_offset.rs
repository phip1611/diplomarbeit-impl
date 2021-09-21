//! An Event Offset describes the offset into occurrences of a specific domain from the event base
//! of the EC. For example, the roottask uses zero as event base. When we create a local EC
//! with the event base, `capabilitie_space[base + excp_offset]` will select a capability for
//! a portal. If present (not null capability), this portal will handle the exception.
//!
//! Possible domains:
//! - exceptions (for regular ECs)
//! - vmx intercepts (for virtual CPUs)  [not required for my work/thesis]
//! - svm intercepts                     [not required for my work/thesis]

use enum_iterator::IntoEnumIterator;
use core::convert::TryFrom;

/// Offsets from event base for x86 exceptions.
/// See https://wiki.osdev.org/Exceptions
#[derive(Debug, Copy, Clone, PartialEq, IntoEnumIterator)]
#[repr(u64)]
pub enum ExceptionEventOffset {
    /// Divided by Zero error
    DE = 0x00,
    /// Debug
    DB = 0x01,
    /// Breakpoint
    BP = 0x03,
    /// Overflow
    OF = 0x04,
    /// Bound Range Exceeded
    BR = 0x05,
    /// Invalid Opcode
    UD = 0x06,
    /// Device Not Available
    NM = 0x07,
    /// Double Fault.
    DF = 0x08,
    /// Invalid TSS
    TS = 0x0a,
    /// Segment Not Present
    NP = 0x0b,
    /// Stack Segment Fault
    SS = 0x0c,
    /// General Protection fault,
    /// i.e. I/O port violation or bad memory access.
    GP = 0x0d,
    /// Page Fault.
    PF = 0x0e,
    /// x87 Floating Point Exception
    MF = 0x10,
    /// Alignment Check
    AC = 0x11,
    /// Machine Check
    MC = 0x12,
    /// SIMD Floating Point Exception
    XM = 0x13,

    /// Hedron-Specific: Startup of an EC
    /// TODO talk with julian
    STARTUP = 0x1e,

    /// Hedron-Specific
    /// TODO talk with julian
    RECALL = 0x1f,
}

impl ExceptionEventOffset {
    /// Returns the value of the enum variant.
    pub fn val(self) -> u64 {
        self as u64
    }
}

/*impl From<u64> for ExceptionEventOffset {
    fn from(val: u64) -> Self {
        for exc in ExceptionEventOffset::into_enum_iter() {
            if exc.val() == val {
                return exc;
            }
        }
        panic!("invalid exception variant! id={}", val);
    }
}*/

impl TryFrom<u64> for ExceptionEventOffset {
    type Error = ();

    fn try_from(val: u64) -> Result<Self, Self::Error> {
        for exc in ExceptionEventOffset::into_enum_iter() {
            if exc.val() == val {
                return Ok(exc);
            }
        }
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use crate::hedron::event_offset::ExceptionEventOffset;
    use core::convert::TryFrom;

    #[test]
    fn test() {
        let expected = ExceptionEventOffset::GP;
        let input = 13;
        let actual = ExceptionEventOffset::try_from(input).unwrap();
        assert_eq!(expected, actual);
    }
}
