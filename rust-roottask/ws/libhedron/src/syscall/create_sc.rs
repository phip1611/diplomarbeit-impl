//! create_sc syscall

use crate::capability::CapSel;
use crate::consts::NUM_CAP_SEL;
use crate::qpd::Qpd;
use crate::syscall::{
    sys_generic_5,
    SyscallNum,
};
use crate::syscall::{
    SyscallError,
    SyscallResult,
};
use alloc::string::ToString;

/// Creates a SC object for a global EC.
///
/// This function never panics.
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
        log::trace!(
            "syscall create_sc: sel={}, pd={}, ec={}",
            cap_sel,
            owned_pd_sel,
            bound_ec_sel
        );
        let mut arg1 = 0;
        arg1 |= SyscallNum::CreateSc.val() & 0xf;
        arg1 |= cap_sel << 12;
        let arg2 = owned_pd_sel;
        let arg3 = bound_ec_sel;
        let arg4 = scheduling_params.val();
        unsafe {
            sys_generic_5(arg1, arg2, arg3, arg4, 0)
                .map(|_x| ())
                .map_err(|e| SyscallError::HedronStatusError(e.0))
        }
    }
}
