use crate::hedron::capability::CapSel;
use crate::hedron::mtd::Mtd;
use crate::syscall::generic::{
    generic_syscall,
    SyscallNum,
    SyscallStatus,
};

pub fn create_pt(
    // according to spec, this must be 0?! wtf?!
    // new_pt_cap_sel: CapSel,
    own_pd_sel: CapSel,
    bound_ec_sel: CapSel,
    instruction_pointer: *const u64,
    mtd: Mtd,
) -> Result<(), SyscallStatus> {
    let new_pt_cap_sel = 0;
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
