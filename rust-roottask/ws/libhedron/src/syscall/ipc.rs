//! CALL and REPLY syscalls for IPC communication.

use crate::capability::CapSel;
use crate::syscall::generic::{
    generic_syscall,
    SyscallNum,
    SyscallStatus,
};

/// Performs a blocking IPC call to the specified portal.
/// Payload is transferred via the UTCB.
pub fn call(portal_sel: CapSel) -> Result<(), SyscallStatus> {
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
pub fn reply() -> ! {
    unsafe {
        generic_syscall(SyscallNum::Reply.val(), 0, 0, 0, 0)
            .map(|_x| ())
            .map_err(|e| e.0)
    }
    .unwrap();
    unreachable!("syscall reply failed?!")
}
