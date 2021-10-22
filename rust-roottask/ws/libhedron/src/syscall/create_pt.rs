use crate::capability::CapSel;
use crate::mtd::Mtd;
use crate::syscall::generic::{
    generic_syscall,
    SyscallNum,
    SyscallStatus,
};

/// Creates a new portal and attaches it to the owning local EC.
/// It is up to the caller to pass a new, yet unused capability selector.
/// If the call is successful, the kernel will install this kernel object
/// into the capability space of the PD.
pub fn create_pt(
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
) -> Result<(), SyscallStatus> {
    let mut arg1 = 0;
    arg1 |= SyscallNum::CreatePt.val();

    // according to spec, bits 63-8 are the new
    // pt_cap_sel but it is 0.. wtf?!
    arg1 |= new_pt_cap_sel << 8;

    let arg2 = own_pd_sel;
    let arg3 = bound_ec_sel;
    let arg4 = mtd.bits();
    let arg5 = instruction_pointer as u64;

    unsafe {
        generic_syscall(arg1, arg2, arg3, arg4, arg5)
            .map(|_x| ())
            .map_err(|e| e.0)
    }
}
