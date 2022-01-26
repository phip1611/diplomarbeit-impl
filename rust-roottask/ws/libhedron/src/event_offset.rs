//! An Event Offset describes the offset into occurrences of a specific domain from the event base
//! of the EC. For example, the roottask uses zero as event base. When we create a local EC
//! with the event base, `capability_space[base + excp_offset]` will select a capability for
//! a portal. If present (not null capability), this portal will handle the exception.
//!
//! Possible domains:
//! - exceptions (for regular ECs)
//! - vmx intercepts (for virtual CPUs)  (not required for my work/thesis)
//! - svm intercepts                     (not required for my work/thesis)

use core::convert::TryFrom;
use enum_iterator::IntoEnumIterator;

/// Offsets from event base for x86 exceptions.
/// See <https://wiki.osdev.org/Exceptions>.
///
/// # Exception Types
/// ## Faults
/// These can be corrected and the program may continue as if nothing happened.
/// ## Aborts
/// Some severe unrecoverable error.
/// ## Traps
/// Traps are reported immediately after the execution of the trapping instruction.
#[derive(Debug, Copy, Clone, PartialEq, IntoEnumIterator)]
#[repr(u64)]
pub enum ExceptionEventOffset {
    /// Divided by Zero error (#DE).
    DivideByZeroFault = 0,
    /// Debug (#DB).
    DebugTrap = 1,
    /// Non-maskable interrupt
    NonMaskableInterrupt = 2,
    /// Breakpoint (#BP).
    BreakpointTrap = 3,
    /// Overflow (#OF).
    OverflowTrap = 4,
    /// Bound Range Exceeded (#BR).
    BoundRangeExceededFault = 5,
    /// Invalid Opcode (#UD).
    InvalidOpcodeFault = 6,
    /// Device Not Available (#NM).
    /// The Device Not Available exception occurs when an FPU
    /// instruction is attempted but there is no FPU. This is not
    /// likely, as modern processors have built-in FPUs. However,
    /// there are flags in the CR0 register that disable the
    /// FPU/MMX/SSE instructions, causing this exception when they
    /// are attempted. This feature is useful because the operating
    /// system can detect when a user program uses the FPU or XMM
    /// registers and then save/restore them appropriately when
    /// multitasking.
    DeviceNotAvailableFault = 7,
    /// Double Fault (#DF).
    DoubleFaultAbort = 8,
    /// Legacy and handled by #GP for a longer time already.
    _CoProcessorSegmentOverrunFault = 9,
    /// Invalid TSS (#TS).
    InvalidTssFault = 10,
    /// Segment Not Present (#NP).
    SegmentNotPresentFault = 11,
    /// Stack Segment Fault (#SS).
    StackSegmentFault = 12,
    /// General Protection fault (#GP).
    /// I/O port violation for example.
    GeneralProtectionFault = 13,
    /// Page Fault (#PF).
    PageFault = 14,
    /// Unused.
    _Reserved = 15,
    /// x87 Floating Point Exception (#MF).
    X87FloatingPointFault = 16,
    /// Alignment Check
    AlignmentCheckFault = 17,
    /// Machine Check
    MachineCheckAbort = 18,
    /// SIMD Floating Point Exception
    SimdFloatingPointFault = 19,
    /// Virtualization Exception (#VE).
    VirtualizationFault = 20,

    _Unknown21 = 21,
    _Unknown22 = 22,
    _Unknown23 = 23,
    _Unknown24 = 24,
    _Unknown25 = 25,
    _Unknown26 = 26,
    _Unknown27 = 27,
    _Unknown28 = 28,
    _Unknown29 = 29,

    /// Triggered only by global non-Roottask ECs, when they are created. Enables the
    /// runtime to set-up the initial CPU state, for example the `rip`.
    HedronGlobalEcStartup = 30, /* 0x1e */

    /// Hedron-Specific
    /// TODO talk with julian
    /// asynchrone events (nur für vcpus um sie zum exiten zu zwingen, nicht für mich wichtig)
    HedronRecall = 31,
}

impl ExceptionEventOffset {
    /// Returns the value of the enum variant.
    pub fn val(self) -> u64 {
        self as u64
    }
}

impl TryFrom<u64> for ExceptionEventOffset {
    type Error = ();

    fn try_from(val: u64) -> Result<Self, Self::Error> {
        for exc in Self::into_enum_iter() {
            if exc.val() == val {
                return Ok(exc);
            }
        }
        Err(())
    }
}

/// Possible exceptions of VMs (vCPUS) on Hedron.
#[derive(Debug, Copy, Clone, PartialEq, IntoEnumIterator)]
#[repr(u64)]
pub enum VMExceptionEventOffset {
    _Todo,
}

#[cfg(test)]
mod tests {
    use crate::event_offset::ExceptionEventOffset;
    use core::convert::TryFrom;

    #[test]
    fn test() {
        let expected = ExceptionEventOffset::GeneralProtectionFault;
        let input = 13;
        let actual = ExceptionEventOffset::try_from(input).unwrap();
        assert_eq!(expected, actual);
    }
}
