//! Syscall or also called Hypercall in NOVA/Hedron.
//!
//! Covers the low-level part. Just the raw system calls with nice typings.

use core::arch::asm;
use enum_iterator::IntoEnumIterator;

/// Does a NOVA/Hedron syscall with 5 arguments.
/// On success, the "out2"-value is returned.
/// On failure, the error code ("out1") is returned
/// together with "out2".
///
/// This function never panics.
pub unsafe fn generic_syscall(
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
) -> Result<u64, (SyscallStatus, u64)> {
    let out1: u64;
    let out2;
    asm!(
        // there is no need to write "mov"-instructions, see below
        "syscall",
        // from 'in("rax")' the compiler will
        // generate corresponding 'mov'-instructions
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("rax") arg4,
        in("r8") arg5,
        lateout("rdi") out1,
        lateout("rsi") out2,
        // mark as clobbered
        // https://doc.rust-lang.org/beta/unstable-book/library-features/asm.html
        // NOVA/Hedron spec lists all registers that may be altered
        lateout("r11") _,
        lateout("rcx") _,
        // Memory Clobber not necessary, because this is the default in Rust
        options(nostack) // probably no effect, but strictly speaking correct
    );
    let (out1, out2) = (SyscallStatus::from(out1), out2);
    if out1 == SyscallStatus::Success {
        Ok(out2)
    } else {
        Err((out1, out2))
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u64)]
pub enum SyscallNum {
    Call = 0,
    Reply = 1,
    CreatePd = 2,
    CreateEc = 3,
    CreateSc = 4,
    CreatePt = 5,
    CreateSm = 6,
    Revoke = 7,
    PdCtrl = 8,
    EcTrl = 9,
    ScCtrl = 10,
    PtCtrl = 11,
    SmCtrl = 12,
    AssignPci = 13,
    AssignGsi = 14,
    MachineCtrl = 15,
}

impl SyscallNum {
    pub fn val(self) -> u64 {
        self as u64
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u64)]
pub enum PdCtrlSubSyscall {
    PdCtrlDelegate = 2,
    PdCtrlMsgAccess = 3,
}

impl PdCtrlSubSyscall {
    pub fn val(self) -> u64 {
        self as u64
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u64)]
pub enum EcCtrlSubSyscall {
    EcCtrlRecall = 0,
}

impl EcCtrlSubSyscall {
    pub fn val(self) -> u64 {
        self as u64
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u64)]
pub enum MachineCtrlSubSyscall {
    MachineCtrlSuspend = 0,
    MachineCtrlUpdateMicrocode = 1,
}

impl MachineCtrlSubSyscall {
    pub fn val(self) -> u64 {
        self as u64
    }
}

/// Possible return values from the syscall.
/// All except the 0 value are error codes.
#[derive(Debug, Copy, Clone, PartialEq, IntoEnumIterator)]
#[repr(u64)]
pub enum SyscallStatus {
    /// The operation completed successfully
    Success = 0,
    /// The operation timed out
    Timeout = 1,
    /// The operation was aborted
    Abort = 2,
    /// An invalid hypercall was called
    BadHyp = 3,
    /// A hypercall referred to an empty or otherwise invalid capability
    BadCap = 4,
    /// A hypercall used invalid parameters
    BadPar = 5,
    /// An invalid feature was requested
    BadFtr = 6,
    /// A portal capability was used on the wrong CPU
    BadCpu = 7,
    /// An invalid device ID was passed
    BadDev = 8,
}

impl From<u64> for SyscallStatus {
    /// Constructs a SyscallStatus with respect to [`SYSCALL_STATUS_BITMASK`].
    fn from(val: u64) -> Self {
        let val = val & Self::SYSCALL_STATUS_BITMASK;

        // generated during compile time; probably not recognized by IDE
        for variant in Self::into_enum_iter() {
            if variant.val() == val {
                return variant;
            }
        }

        panic!("invalid variant! id={}", val);
    }
}

impl SyscallStatus {
    /// Only the lowest 8 bits are used to encode the status.
    const SYSCALL_STATUS_BITMASK: u64 = 0xff;

    pub fn val(self) -> u64 {
        self as u64
    }
}

#[cfg(test)]
mod tests {
    use crate::syscall::generic::SyscallStatus;

    #[test]
    fn test_syscall_status() {
        // text that bitmask gets used
        assert_eq!(SyscallStatus::from(0x2500), SyscallStatus::Success);
        assert_eq!(SyscallStatus::from(1), SyscallStatus::Timeout);
    }
}
