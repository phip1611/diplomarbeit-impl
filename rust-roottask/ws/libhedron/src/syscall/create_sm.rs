//! create_sm syscall

use crate::capability::CapSel;
use crate::consts::NUM_CAP_SEL;
use crate::syscall::{
    hedron_syscall_4,
    SyscallNum,
};
use crate::syscall::{
    SyscallError,
    SyscallResult,
};
use alloc::string::ToString;

/// Creates a SM object.
///
/// # Parameters
/// - `cap_sel` Cap Sel that will point to the new SM object
/// - `owned_pd_sel` Cap Sel of the owning PD
/// - `count` Initial count for the semaphore
///
/// # Safety
/// * This function may change the systems functionality in an unintended way,
///   if the arguments are illegal or wrong.
/// * This function is not allowed to panic.
/// * This function is strictly required to never produce any side effect system calls! Therefore,
///   also no log::trace()-stuff or similar. Otherwise, the current implementation of hybrid
///   foreign system calls will fail.
#[inline]
pub fn sys_create_sm(cap_sel: CapSel, owned_pd_sel: CapSel, count: u64) -> SyscallResult {
    if cap_sel >= NUM_CAP_SEL {
        Err(SyscallError::ClientArgumentError(
            "Argument `cap_sel` is too big".to_string(),
        ))
    } else if owned_pd_sel >= NUM_CAP_SEL {
        Err(SyscallError::ClientArgumentError(
            "Argument `owned_pd_sel` is too big".to_string(),
        ))
    } else {
        let mut arg1 = 0;
        arg1 |= SyscallNum::CreateSm.val() & 0xf;
        arg1 |= cap_sel << 12;
        let arg2 = owned_pd_sel;
        let arg3 = count;
        // currently needs to be zero; this was used for the signal mechanism
        // that is about to be removed from Hedron
        let arg4 = 0;
        unsafe {
            hedron_syscall_4(arg1, arg2, arg3, arg4)
                .map(|_x| ())
                .map_err(|e| SyscallError::HedronStatusError(e.0))
        }
    }
}
