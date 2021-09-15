use crate::hedron::capability::CrdObjPT;
use crate::syscall::syscall::SyscallNum::CreateEc;
use crate::syscall::syscall::{
    generic_syscall,
    SyscallStatus,
};

/// Kind of an EC. Bits 4-5 in ARG1 of syscall.
#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum EcKind {
    /// Local EC without scheduling context.
    /// Usually used as functionality available
    /// through a portal.
    Local = 0,
    Global = 1,
    vCpu = 2,
}

/// `create_ec` creates an EC kernel object and a capability pointing to
/// the newly created kernel object.
///
/// An EC can be either a normal host EC or a virtual CPU. It does not
/// come with scheduling time allocated to it. ECs need scheduling
/// contexts (SCs) to be scheduled and thus executed.
///
/// ECs can be either _global_ or _local_. A global EC can have a
/// dedicated scheduling context (SC) bound to it. When this SC is
/// scheduled the EC runs. Global ECs can be both, normal host ECs and
/// vCPUs. A normal EC bound to an SC builds what is commonly known as a
/// thread.
///
/// Local ECs can only be normal ECs and not vCPUs. They cannot have SCs
/// bound to them and are used for portal handlers. These handlers never
/// execute with their own SC, but borrow the scheduling context from the
/// caller.
///
/// Each EC has an _event base_. This event base is an offset into the
/// capability space of the PD the EC belongs to. Exceptions (for normal
/// ECs) and VM exits (for vCPUs) are sent as messages to the portal index
/// that results from adding the event reason to the event base. For vCPUs
/// the event reason are VM exit reasons, for normal ECs the reasons are
/// exception numbers.
///
/// # Parameters
/// - `kind` see [`EcKind`]
/// - `use_apic_access_page` Whether a vCPU should respect the APIC Access Page. Ignored for non-vCPUs or if no vLAPIC page is created.
/// - `use_page_destination`  If 0, the UTCB / vLAPIC page will be mapped in the parent PD, otherwise it's mapped in the current PD.
/// - `dest_crd` A capability selector in the current PD that will point to the newly created EC.
/// - `parent_pd` A capability selector to a PD domain in which the new EC will execute in.
/// - `utcb_vlapic_page_num` A page number where the UTCB / vLAPIC page will be created. Page 0 means no vLAPIC page or UTCB is created.
/// - `stack_ptr` The initial stack pointer for normal ECs. Ignored for vCPUs.
/// - `event_base` The Event Base of the newly created EC.
pub fn create_ec(
    kind: EcKind,
    use_apic_access_page: bool,
    use_page_destination: bool,
    dest_crd: CrdObjPT,
    parent_pd: CrdObjPT,
    utcb_vlapic_page_num: u64,
    stack_ptr: u64,
    event_base: u64,
) -> Result<(), SyscallStatus> {
    let mut arg1 = 0;
    arg1 |= CreateEc as u64;
    arg1 |= ((kind as u64) << 4) & 0x30;
    // Ignored for non-vCPUs or if no vLAPIC page is created.
    if use_apic_access_page {
        arg1 |= 1 << 6;
    }
    if use_page_destination {
        arg1 |= 1 << 7;
    }
    arg1 |= dest_crd.val() << 8;
    let arg2 = parent_pd.val();
    // bits 0-11 are unused; needs to be zero
    let mut arg3 = 0;
    arg3 |= utcb_vlapic_page_num << 12;
    let arg4 = stack_ptr;
    let arg5 = event_base;
    unsafe {
        generic_syscall(arg1, arg2, arg3, arg4, arg5)
            .map(|_x| ())
            .map_err(|e| e.0)
    }
}