//! create_sc syscall

use crate::capability::CapSel;
use crate::consts::NUM_CAP_SEL;
use crate::qpd::Qpd;
use crate::syscall::{
    hedron_syscall_4,
    SyscallNum,
};
use crate::syscall::{
    SyscallError,
    SyscallResult,
};
use alloc::string::ToString;

/// Creates a SC object for a global EC.
///
/// # Safety
/// * This function may change the systems functionality in an unintended way,
///   if the arguments are illegal or wrong.
/// * This function is not allowed to panic.
/// * This function is strictly required to never produce any side effect system calls! Therefore,
///   also no log::trace()-stuff or similar. Otherwise, the current implementation of hybrid
///   foreign system calls will fail.
#[inline]
pub fn sys_create_sc(
    cap_sel: CapSel,
    owned_pd_sel: CapSel,
    bound_ec_sel: CapSel,
    scheduling_params: Qpd,
) -> SyscallResult {
    if cap_sel >= NUM_CAP_SEL {
        Err(SyscallError::ClientArgumentError(
            "Argument `cap_sel` is too big".to_string(),
        ))
    } else if owned_pd_sel >= NUM_CAP_SEL {
        Err(SyscallError::ClientArgumentError(
            "Argument `owned_pd_sel` is too big".to_string(),
        ))
    } else if bound_ec_sel >= NUM_CAP_SEL {
        Err(SyscallError::ClientArgumentError(
            "Argument `bound_ec_sel` is too big".to_string(),
        ))
    } else {
        /*#[cfg(not(feature = "foreign_rust_rt"))]
        log::trace!(
            "syscall create_sc: sel={}, pd={}, ec={}",
            cap_sel,
            owned_pd_sel,
            bound_ec_sel
        );*/

        let mut arg1 = 0;
        arg1 |= SyscallNum::CreateSc.val() & 0xf;
        arg1 |= cap_sel << 12;
        let arg2 = owned_pd_sel;
        let arg3 = bound_ec_sel;
        let arg4 = scheduling_params.val();
        unsafe {
            hedron_syscall_4(arg1, arg2, arg3, arg4)
                .map(|_x| ())
                .map_err(|e| SyscallError::HedronStatusError(e.0))
        }
    }
}
