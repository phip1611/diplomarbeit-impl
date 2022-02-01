use crate::capability::CapSel;
use crate::consts::{
    NUM_CAP_SEL,
    NUM_CPUS,
    NUM_EXC,
};
use crate::syscall::hedron_syscall_5;
use crate::syscall::SyscallNum::CreateEc;
use crate::syscall::{
    SyscallError,
    SyscallResult,
};
use alloc::string::ToString;

/// Kind of an EC. Bits 9-7 in ARG1 of syscall.
#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types, dead_code)]
#[repr(u8)]
enum EcKind {
    /// Local EC without scheduling context.
    /// Used as functionality available through a portal.
    Local = 0b00,
    Global = 0b01,
    vCpu = 0b10,
}

impl EcKind {
    /// Bitmask for EcKind. Bits 9-8.
    const BITMASK: u64 = 0x300;
    const LEFT_SHIFT: u64 = 8;

    const fn val(self) -> u8 {
        self as u8
    }
}

/// Creates a local EC. Wrapper around [`sys_create_ec`].
///
/// # Safety
/// * This function may change the systems functionality in an unintended way,
///   if the arguments are illegal or wrong.
/// * This function is not allowed to panic.
/// * This function is strictly required to never produce any side effect system calls! Therefore,
///   also no log::trace()-stuff or similar. Otherwise, the current implementation of hybrid
///   foreign system calls will fail.
///
/// # Parameters
/// - `ec_cap_sel` Free [`CapSel`] where this EC is installed in the PD specified by `parent_pd_sel`
/// - `parent_pd_sel` [`CapSel`] of existing PD, where the EC belongs to
/// - `stack_ptr` Virtual address of stack. NOT PAGE NUMBER.
/// - `evt_base_sel` [`CapSel`] for the event base.
/// - `cpu_num` Number of the CPU. ECs are permanently bound to a CPU.
/// - `utcb_page_num` Page number of the UTCB. NOT A VIRTUAL ADDRESS.
#[inline]
pub fn sys_create_local_ec(
    ec_cap_sel: CapSel,
    parent_pd_sel: CapSel,
    stack_ptr: u64,
    evt_base_sel: CapSel,
    cpu_num: u64,
    utcb_page_num: u64,
) -> SyscallResult {
    if stack_ptr == 0 {
        Err(SyscallError::ClientArgumentError(
            "Argument `stack_ptr` is null".to_string(),
        ))
    } else if utcb_page_num == 0 {
        Err(SyscallError::ClientArgumentError(
            "Argument `utcb_page_num` is null".to_string(),
        ))
    } else {
        sys_create_ec(
            EcKind::Local,
            ec_cap_sel,
            parent_pd_sel,
            stack_ptr,
            evt_base_sel,
            cpu_num,
            utcb_page_num,
            false,
            // TODO do I ever need this?
            false,
        )
    }
}

/// Creates a global EC. . Wrapper around [`sys_create_ec`].
/// This will result in a [`crate::event_offset::ExceptionEventOffset::HedronGlobalEcStartup`]
/// exception in the PD, where the new global EC belongs to. Note that in comparison to
/// [`sys_create_local_ec`], this doesn't take a `stack_ptr` argument, because the stack
/// is set in the handler of the [`crate::event_offset::ExceptionEventOffset::HedronGlobalEcStartup`]
/// exception.
///
/// # Safety
/// * This function may change the systems functionality in an unintended way,
///   if the arguments are illegal or wrong.
/// * This function is not allowed to panic.
/// * This function is strictly required to never produce any side effect system calls! Therefore,
///   also no log::trace()-stuff or similar. Otherwise, the current implementation of hybrid
///   foreign system calls will fail.
#[inline]
pub fn sys_create_global_ec(
    ec_cap_sel: CapSel,
    parent_pd_sel: CapSel,
    evt_base_sel: CapSel,
    cpu_num: u64,
    utcb_page_num: u64,
) -> SyscallResult {
    if utcb_page_num == 0 {
        Err(SyscallError::ClientArgumentError(
            "Argument `utcb_page_num` is null".to_string(),
        ))
    } else {
        sys_create_ec(
            EcKind::Global,
            ec_cap_sel,
            parent_pd_sel,
            0,
            evt_base_sel,
            cpu_num,
            utcb_page_num,
            false,
            // TODO do I ever need this?
            false,
        )
    }
}

const USE_APIC_ACCESS_PAGE_LEFT_SHIFT: u64 = 10;
const USE_PAGE_DESTINATION_LEFT_SHIFT: u64 = 11;
const DEST_CAP_SEL_LEFT_SHIFT: u64 = 12;
const PAGE_NUM_LEFT_SHIFT: u64 = 12;

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
/// # Safety
/// * This function may change the systems functionality in an unintended way,
///   if the arguments are illegal or wrong.
/// * This function is not allowed to panic.
/// * This function is strictly required to never produce any side effect system calls! Therefore,
///   also no log::trace()-stuff or similar. Otherwise, the current implementation of hybrid
///   foreign system calls will fail.
///
/// # Parameters
/// - `kind` see [`EcKind`]
/// - `dest_cap_sel`   A capability selector in the current PD that will point to the newly created EC.
/// - `parent_pd_sel`  A capability selector to a PD domain in which the new EC will execute in.
/// - `stack_ptr`      The initial stack pointer for normal ECs (local & global). Ignored for vCPUs.
/// - `event_base_sel` The base selector for events. Base for event offsets like [`crate::event_offset::ExceptionEventOffset`]
///                    in the capability space of the corresponding PD.
/// - `cpu_num`        Number of CPU (ECs are CPU local). 0 to 63 (maximum supported CPU count by Hedron)
/// - `utcb_vlapic_page_num` A page number where the UTCB / vLAPIC page will be created. Page 0 means no vLAPIC page or UTCB is created.
/// - `use_apic_access_page` Whether a vCPU should respect the APIC Access Page. Ignored for non-vCPUs or if no vLAPIC page is created.
///                          Important for interrupt handling.
/// - `use_page_destination`  If 0, the UTCB / vLAPIC page will be mapped in the parent PD, otherwise it's mapped in the current PD.
#[allow(clippy::too_many_arguments)]
#[inline]
fn sys_create_ec(
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
) -> SyscallResult {
    if dest_cap_sel >= NUM_CAP_SEL {
        Err(SyscallError::ClientArgumentError(
            "Argument `dest_cap_sel` is bigger than NUM_CAP_SEL".to_string(),
        ))
    } else if parent_pd_sel >= NUM_CAP_SEL {
        Err(SyscallError::ClientArgumentError(
            "Argument `parent_pd_sel` is bigger than NUM_CAP_SEL".to_string(),
        ))
    } else if cpu_num >= NUM_CPUS as u64 {
        Err(SyscallError::ClientArgumentError(
            "Argument `cpu_num` is too big".to_string(),
        ))
    } else if event_base_sel + NUM_EXC as u64 >= NUM_CAP_SEL as u64 {
        Err(SyscallError::ClientArgumentError(
            "Argument `event_base_sel` is too big".to_string(),
        ))
    } else {
        #[cfg(not(feature = "foreign_rust_rt"))]
        log::trace!(
            "syscall create_ec: kind={:?}, sel={}, pd={}, evt_base={}, cpu_num={}, utcb_lapic_page_num={}, utcb_lapic_page_num_addr={:016x}",
            kind,
            dest_cap_sel,
            parent_pd_sel,
            event_base_sel,
            cpu_num,
            utcb_vlapic_page_num,
            utcb_vlapic_page_num * crate::mem::PAGE_SIZE as u64,
        );

        let mut arg1 = 0;
        arg1 |= CreateEc.val();
        arg1 |= ((kind.val() as u64) << EcKind::LEFT_SHIFT) & EcKind::BITMASK;

        // Ignored for non-vCPUs or if no vLAPIC page is created.
        if use_apic_access_page {
            arg1 |= 1 << USE_APIC_ACCESS_PAGE_LEFT_SHIFT;
        }
        if use_page_destination {
            arg1 |= 1 << USE_PAGE_DESTINATION_LEFT_SHIFT;
        }
        arg1 |= dest_cap_sel << DEST_CAP_SEL_LEFT_SHIFT;

        let arg2 = parent_pd_sel;

        let mut arg3 = 0;
        arg3 |= cpu_num & 0xfff;
        arg3 |= utcb_vlapic_page_num << PAGE_NUM_LEFT_SHIFT;

        let arg4 = stack_ptr;

        let arg5 = event_base_sel;

        unsafe {
            hedron_syscall_5(arg1, arg2, arg3, arg4, arg5)
                .map(|_x| ())
                .map_err(|e| SyscallError::HedronStatusError(e.0))
        }
    }
}
