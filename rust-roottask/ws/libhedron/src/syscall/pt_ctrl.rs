//! PT CTRL-syscalls.

use crate::capability::CapSel;
use crate::consts::NUM_CAP_SEL;
use crate::syscall::{
    sys_generic_5,
    SyscallNum,
};
use crate::syscall::{
    SyscallError,
    SyscallResult,
};
use alloc::string::ToString;

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

///
/// This function never panics.
pub fn sys_pt_ctrl(pt_sel: CapSel, callback_argument: u64) -> SyscallResult {
    if pt_sel >= NUM_CAP_SEL {
        Err(SyscallError::ClientArgumentError(
            "Argument `pt_sel` is too big".to_string(),
        ))
    } else {
        let mut arg1 = 0;
        arg1 |= SyscallNum::PtCtrl.val() & 0xff;
        arg1 |= pt_sel << 12;
        unsafe {
            sys_generic_5(arg1, callback_argument, 0, 0, 0)
                .map(|_x| ())
                .map_err(|e| SyscallError::HedronStatusError(e.0))
        }
    }
}
