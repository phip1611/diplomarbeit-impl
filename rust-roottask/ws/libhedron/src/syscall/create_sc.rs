//! create_sc syscall

use crate::capability::CapSel;
use crate::qpd::Qpd;
use crate::syscall::generic::{
    generic_syscall,
    SyscallNum,
    SyscallStatus,
};

pub fn create_sc(
    cap_sel: CapSel,
    owned_pd_sel: CapSel,
    bound_ec_sel: CapSel,
    scheduling_params: Qpd,
) -> Result<(), SyscallStatus> {
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
