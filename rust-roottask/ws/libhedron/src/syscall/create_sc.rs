//! create_sc syscall

use crate::capability::CapSel;
use crate::consts::NUM_CAP_SEL;
use crate::qpd::Qpd;
use crate::syscall::generic::{
    generic_syscall,
    SyscallNum,
    SyscallStatus,
};

/// Creates a SC object for a global EC.
pub fn create_sc(
    cap_sel: CapSel,
    owned_pd_sel: CapSel,
    bound_ec_sel: CapSel,
    scheduling_params: Qpd,
) -> Result<(), SyscallStatus> {
    assert!(
        cap_sel < NUM_CAP_SEL,
        "maximum cap sel for object capabilities exceeded!"
    );
    assert!(
        bound_ec_sel < NUM_CAP_SEL,
        "maximum cap sel for object capabilities exceeded!"
    );
    assert!(
        owned_pd_sel < NUM_CAP_SEL,
        "maximum cap sel for object capabilities exceeded!"
    );
    log::trace!(
        "syscall create_sc: sel={}, pd={}, ec={}",
        cap_sel,
        owned_pd_sel,
        bound_ec_sel
    );
    let mut arg1 = 0;
    arg1 |= SyscallNum::CreateSc.val() & 0xf;
    arg1 |= cap_sel << 8;
    let arg2 = owned_pd_sel;
    let arg3 = bound_ec_sel;
    let arg4 = scheduling_params.val();
    unsafe {
        generic_syscall(arg1, arg2, arg3, arg4, 0)
            .map(|_x| ())
            .map_err(|e| e.0)
    }
}
