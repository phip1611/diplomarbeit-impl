//! CALL and REPLY syscalls for IPC communication.

use crate::capability::CapSel;
use crate::consts::NUM_CAP_SEL;
use crate::syscall::{
    hedron_syscall_1,
    SyscallNum,
};
use crate::syscall::{
    SyscallError,
    SyscallResult,
};
use alloc::string::ToString;
use core::arch::asm;

/// Performs a blocking IPC syscall to the specified portal.
/// Payload is transferred via the UTCB.
///
/// # Safety
/// * This function may change the systems functionality in an unintended way,
///   if the arguments are illegal or wrong.
/// * This function is not allowed to panic.
/// * This function is strictly required to never produce any system calls! Therefore also no
///   log::trace()-stuff or similar. Otherwise, the current implementation of hybrid foreign
///   system calls will fail.
#[inline]
pub fn sys_call(portal_sel: CapSel) -> SyscallResult {
    if portal_sel >= NUM_CAP_SEL {
        Err(SyscallError::ClientArgumentError(
            "Argument `portal_sel` is too big".to_string(),
        ))
    } else {
        let mut arg1 = 0;
        arg1 |= SyscallNum::Call.val();

        #[allow(unused)]
        const BLOCKING: usize = 0;
        #[allow(unused)]
        const NON_BLOCKING: usize = 1;

        let flags = BLOCKING as u64;
        arg1 |= (flags << 8) & 0xf00;

        arg1 |= portal_sel << 12;

        unsafe {
            hedron_syscall_1(arg1)
                .map(|_x| ())
                .map_err(|e| SyscallError::HedronStatusError(e.0))
        }
    }
}

/// Syscall used in invoked portals. Replies to the caller of a portal,
/// i.e. the kernel that send an exception message or a normal user application
/// (IPC). The data is transferred via the UTCB.
///
/// Pitfall: Hedron doesn't reset the RSP of the local EC that handles calls.
/// Therefore, during a reply, the userland has to do this by itself, in order
/// to fulfill the next request as expected.
///
/// # Safety
/// * This function may change the systems functionality in an unintended way,
///   if the arguments are illegal or wrong.
/// * This function is not allowed to panic.
/// * This function is strictly required to never produce any side effect system calls! Therefore,
///   also no log::trace()-stuff or similar. Otherwise, the current implementation of hybrid
///   foreign system calls will fail.
#[inline]
pub fn sys_reply(local_ec_stack_top: u64) -> ! {
    #[cfg(not(feature = "foreign_rust_rt"))]
    if local_ec_stack_top == 0 {
        log::error!("local_ec_stack_top is 0!")
    }
    unsafe {
        asm!(
            "mov rsp, {0}",
            "syscall",
            in(reg) local_ec_stack_top,
            in("rdi") SyscallNum::Reply.val(),
            // no clobbers here, because there isn't code after this anyway
            options(nostack, noreturn) // probably no effect, but strictly speaking correct
        )
    };
    // unreachable!("syscall reply failed?!")
}
