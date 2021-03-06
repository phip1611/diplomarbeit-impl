//! Syscall or also called Hypercall in NOVA/Hedron.
//!
//! Covers the low-level part. Just the raw system calls with nice typings.

use core::arch::asm;
use enum_iterator::IntoEnumIterator;

/// Does a Hedron syscall with 5 arguments. On success, the "out2"-value is returned.
/// On failure, the error code ("out1") is returned together with "out2".
///
/// This function never panics.
///
/// # Safety
/// * This function may change the systems functionality in an unintended way,
///   if the arguments are illegal or wrong.
/// * This function is not allowed to panic.
/// * This function is strictly required to never produce any side effect system calls! Therefore,
///   also no log::trace()-stuff or similar. Otherwise, the current implementation of hybrid
///   foreign system calls will fail.
#[inline]
pub(super) unsafe fn hedron_syscall_5(
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
) -> Result<u64, (SyscallStatus, u64)> {
    let out1: u64;
    let out2;
    asm!(
        "syscall",
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

/// Does a Hedron syscall with 4 arguments. On success, the "out2"-value is returned.
/// On failure, the error code ("out1") is returned together with "out2".
///
/// This function never panics.
///
/// # Safety
/// * This function may change the systems functionality in an unintended way,
///   if the arguments are illegal or wrong.
/// * This function is not allowed to panic.
/// * This function is strictly required to never produce any side effect system calls! Therefore,
///   also no log::trace()-stuff or similar. Otherwise, the current implementation of hybrid
///   foreign system calls will fail.
#[inline]
pub(super) unsafe fn hedron_syscall_4(
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
) -> Result<u64, (SyscallStatus, u64)> {
    let out1: u64;
    let out2;
    asm!(
        "syscall",
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("rax") arg4,
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

/// Does a Hedron syscall with 3 arguments. On success, the "out2"-value is returned.
/// On failure, the error code ("out1") is returned together with "out2".
///
/// This function never panics.
///
/// # Safety
/// * This function may change the systems functionality in an unintended way,
///   if the arguments are illegal or wrong.
/// * This function is not allowed to panic.
/// * This function is strictly required to never produce any system calls! Therefore also no
///   log::trace()-stuff or similar. Otherwise, the current implementation of hybrid foreign
///   system calls will fail.
#[allow(unused)]
#[inline]
pub(super) unsafe fn hedron_syscall_3(
    arg1: u64,
    arg2: u64,
    arg3: u64,
) -> Result<u64, (SyscallStatus, u64)> {
    let out1: u64;
    let out2;
    asm!(
        "syscall",
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
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

/// Does a Hedron syscall with 2 arguments. On success, the "out2"-value is returned.
/// On failure, the error code ("out1") is returned together with "out2".
///
/// This function never panics.
///
/// # Safety
/// * This function may change the systems functionality in an unintended way,
///   if the arguments are illegal or wrong.
/// * This function is not allowed to panic.
/// * This function is strictly required to never produce any system calls! Therefore also no
///   log::trace()-stuff or similar. Otherwise, the current implementation of hybrid foreign
///   system calls will fail.
#[inline]
pub(super) unsafe fn hedron_syscall_2(arg1: u64, arg2: u64) -> Result<u64, (SyscallStatus, u64)> {
    let out1: u64;
    let out2;
    asm!(
        "syscall",
        in("rdi") arg1,
        in("rsi") arg2,
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

/// Does a Hedron syscall with 2 arguments. On success, the "out2"-value is returned.
/// On failure, the error code ("out1") is returned together with "out2".
///
/// This function never panics.
///
/// # Safety
/// * This function may change the systems functionality in an unintended way,
///   if the arguments are illegal or wrong.
/// * This function is not allowed to panic.
/// * This function is strictly required to never produce any system calls! Therefore also no
///   log::trace()-stuff or similar. Otherwise, the current implementation of hybrid foreign
///   system calls will fail.
#[inline]
pub(super) unsafe fn hedron_syscall_1(arg1: u64) -> Result<u64, (SyscallStatus, u64)> {
    let out1: u64;
    let out2;
    asm!(
        "syscall",
        in("rdi") arg1,
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
    pub const fn val(self) -> u64 {
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
    pub const fn val(self) -> u64 {
        self as u64
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u64)]
pub enum EcCtrlSubSyscall {
    EcCtrlRecall = 0,
}

impl EcCtrlSubSyscall {
    pub const fn val(self) -> u64 {
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
    pub const fn val(self) -> u64 {
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
    /// Constructs a SyscallStatus with respect to [`Self::SYSCALL_STATUS_BITMASK`].
    fn from(val: u64) -> Self {
        let val = val & Self::SYSCALL_STATUS_BITMASK;

        // I chose the variant with transmute for maximum performance during syscalls
        const MAX_STATUS_VAL: u64 = 8;
        if val > MAX_STATUS_VAL {
            panic!("invalid variant! id={}", val);
        }

        // for maximum syscall performance in benchmarks I prefere thos
        unsafe { core::mem::transmute::<u64, SyscallStatus>(val) }
    }
}

impl SyscallStatus {
    /// Only the lowest 8 bits are used to encode the status.
    const SYSCALL_STATUS_BITMASK: u64 = 0xff;

    pub const fn val(self) -> u64 {
        self as u64
    }
}

#[cfg(test)]
mod tests {
    use crate::syscall::SyscallStatus;

    #[ignore]
    #[test]
    fn test_syscall_status() {
        // text that bitmask gets used
        assert_eq!(SyscallStatus::from(0x2500), SyscallStatus::Success);
        assert_eq!(SyscallStatus::from(1), SyscallStatus::Timeout);
    }
}
