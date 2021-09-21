//! PT CTRL-syscalls.

use crate::hedron::capability::CapSel;
use crate::syscall::generic::{
    generic_syscall,
    SyscallNum,
    SyscallStatus,
};

/// Attaches a specific argument to the callback function of a portal. When the portal gets
/// called, this argument gets passed as first and only argument into the callback function
/// specified, when the portal was created.
///
/// Typically usage: assign the x86 exception or a known ID from a enum, which
/// tells you about the context.
///
/// This implies that you need N portals for N exceptions.
///
/// callback_argument is also called Portal ID in spec and supernova.
pub fn pt_ctrl(pt_sel: CapSel, callback_argument: u64) -> Result<(), SyscallStatus> {
    let mut arg1 = 0;
    arg1 |= SyscallNum::PtCtrl.val() & 0xf;
    arg1 |= pt_sel << 8;
    unsafe {
        generic_syscall(arg1, callback_argument, 0, 0, 0)
            .map(|_x| ())
            .map_err(|e| e.0)
    }
}
