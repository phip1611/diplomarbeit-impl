use crate::hedron::capability::CapSel;
use crate::syscall::generic::SyscallNum::CreateEc;
use crate::syscall::generic::{
    generic_syscall,
    SyscallStatus,
};
use crate::hedron::consts::NUM_CPUS;

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

impl EcKind {
    pub fn val(self) -> u8 {
        self as u8
    }
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
/// - `dest_cap_sel`   A capability selector in the current PD that will point to the newly created EC.
/// - `parent_pd_sel`  A capability selector to a PD domain in which the new EC will execute in.
/// - `stack_ptr`      The initial stack pointer for normal ECs (local & global). Ignored for vCPUs.
/// - `event_base_sel` The base selector for events. Base for event offsets like [`roottask_lib::hedron::event_offset::ExceptionEventOffset`]
///                    in the capability space of the corresponding PD.
/// - `cpu_num`        Number of CPU (ECs are CPU local). 0 to 63 (maximum supported CPU count by Hedron)
/// - `utcb_vlapic_page_num` A page number where the UTCB / vLAPIC page will be created. Page 0 means no vLAPIC page or UTCB is created.
/// - `use_apic_access_page` Whether a vCPU should respect the APIC Access Page. Ignored for non-vCPUs or if no vLAPIC page is created.
/// - `use_page_destination`  If 0, the UTCB / vLAPIC page will be mapped in the parent PD, otherwise it's mapped in the current PD.
pub fn create_ec(
    kind: EcKind,
    dest_cap_sel: CapSel,
    parent_pd_sel: CapSel,
    stack_ptr: u64,
    event_base_sel: CapSel,
    cpu_num: u64,
    // this is Hedron-specific
    utcb_vlapic_page_num: u64,
    use_apic_access_page: bool,
    use_page_destination: bool,
) -> Result<(), SyscallStatus> {
    let mut arg1 = 0;
    arg1 |= CreateEc.val();
    arg1 |= ((kind.val() as u64) << 4) & 0x30;

    // Ignored for non-vCPUs or if no vLAPIC page is created.
    if use_apic_access_page {
        arg1 |= 1 << 6;
    }
    if use_page_destination {
        arg1 |= 1 << 7;
    }
    arg1 |= dest_cap_sel << 8;

    let arg2 = parent_pd_sel;

    let mut arg3 = 0;
    // CPU 0 to 63 (the maximum supported CPU count)
    if arg3 > NUM_CPUS as u64 {
        log::warn!("Hedron supports CPUs 0..{}, you requested {}", NUM_CPUS - 1, cpu_num);
    }
    arg3 |= cpu_num & 0xfff;
    arg3 |= utcb_vlapic_page_num << 12;

    let arg4 = stack_ptr;

    let arg5 = event_base_sel;

    unsafe {
        generic_syscall(arg1, arg2, arg3, arg4, arg5)
            .map(|_x| ())
            .map_err(|e| e.0)
    }
}
