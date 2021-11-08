//! CALL and REPLY syscalls for IPC communication.

use crate::capability::CapSel;
use crate::consts::NUM_CAP_SEL;
use crate::syscall::generic::{
    generic_syscall,
    SyscallNum,
    SyscallStatus,
};

/// Performs a blocking IPC call to the specified portal.
/// Payload is transferred via the UTCB.
pub fn call(portal_sel: CapSel) -> Result<(), SyscallStatus> {
    assert!(
        portal_sel < NUM_CAP_SEL,
        "maximum cap sel for object capabilities exceeded!"
    );
    let mut arg1 = 0;
    arg1 |= SyscallNum::Call.val();

    #[allow(unused)]
    const BLOCKING: usize = 0;
    #[allow(unused)]
    const NON_BLOCKING: usize = 1;

    let flags = BLOCKING as u64;
    arg1 |= (flags << 4) & 0xf0;

    arg1 |= portal_sel << 8;

    unsafe {
        generic_syscall(arg1, 0, 0, 0, 0)
            .map(|_x| ())
            .map_err(|e| e.0)
    }
}

/// Syscall used in invoked portals. Replies to the caller of a portal,
/// i.e. the kernel that send an exception message or a normal user application
/// (IPC). The data is transferred via the UTCB.
///
/// Pitfall: Hedron doesn't reset the RSP of the local EC that handles calls.
/// Therefore, during a reply, the userland has to do this by itself, in order
/// to fulfill the next request as expected.
pub fn reply(local_ec_stack_top: u64) -> ! {
    unsafe {
        asm!(
            "mov rsp, {0}",
            "syscall",
            in(reg) local_ec_stack_top,
            in("rdi") SyscallNum::Reply.val(),
            // no clobbers here, because there isn't code after this anyway
            options(nostack) // probably no effect, but strictly speaking correct
        )
    };
    unreachable!("syscall reply failed?!")
}
