use crate::capability::CapSel;
use crate::consts::NUM_CAP_SEL;
use crate::mtd::Mtd;
use crate::syscall::generic::{
    sys_generic_5,
    SyscallNum,
};
use crate::syscall::{
    SyscallError,
    SyscallResult,
};
use alloc::string::ToString;

/// Creates a new portal and attaches it to the owning local EC.
/// It is up to the caller to pass a new, yet unused capability selector.
/// If the call is successful, the kernel will install this kernel object
/// into the capability space of the PD.
///
/// This function never panics.
pub fn sys_create_pt(
    // Free selector (must refer to a null capability).
    // The portal is installed at this [`CapSel`].
    new_pt_cap_sel: CapSel,
    // Target PD for the PT. Depends on use case (own or foreign PD).
    own_pd_sel: CapSel,
    // Generally the [`CapSel`] of the new EC that you just created.
    bound_ec_sel: CapSel,
    // See [`Mtd`].
    mtd: Mtd,
    // Instruction pointer of the portal (entry function).
    // The function can take one argument. To specify the argument,
    // see [`super::pt_ctrl::pt_ctrl`]
    instruction_pointer: *const u64,
) -> SyscallResult {
    if new_pt_cap_sel >= NUM_CAP_SEL {
        Err(SyscallError::ClientArgumentError(
            "Argument `new_pt_cap_sel` is too big".to_string(),
        ))
    } else if own_pd_sel >= NUM_CAP_SEL {
        Err(SyscallError::ClientArgumentError(
            "Argument `own_pd_sel` is too big".to_string(),
        ))
    } else if bound_ec_sel >= NUM_CAP_SEL {
        Err(SyscallError::ClientArgumentError(
            "Argument `bound_ec_sel` is too big".to_string(),
        ))
    } else {
        let mut arg1 = 0;
        arg1 |= SyscallNum::CreatePt.val();
        arg1 |= new_pt_cap_sel << 12;

        let arg2 = own_pd_sel;
        let arg3 = bound_ec_sel;
        let arg4 = mtd.bits();
        let arg5 = instruction_pointer as u64;

        unsafe {
            sys_generic_5(arg1, arg2, arg3, arg4, arg5)
                .map(|_x| ())
                .map_err(|e| SyscallError::HedronStatusError(e.0))
        }
    }
}
