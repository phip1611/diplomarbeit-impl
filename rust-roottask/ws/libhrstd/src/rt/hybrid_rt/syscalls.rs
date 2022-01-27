//! Contains all hybrid syscall wrappers.

use crate::rt::user_load_utcb::user_load_utcb_mut;
use libhedron::syscall::sys_create_pd;
use libhedron::syscall::sys_create_pt;
use libhedron::syscall::sys_create_sc;
use libhedron::syscall::sys_pt_ctrl;
use libhedron::syscall::SyscallResult;
use libhedron::syscall::{
    sys_create_global_ec,
    sys_create_local_ec,
};
use libhedron::syscall::{
    sys_pd_ctrl_delegate,
    DelegateFlags,
};
use libhedron::Mtd;
use libhedron::Qpd;
use libhedron::{
    CapSel,
    Crd,
};

const HEDRON_NATIVE_SYSCALL_MAGIC: u64 = 1 << 63;

/// Wraps a single Hedron syscall. The code inside `actions`
/// should not do any more than the raw syscall. Things as
/// unwrapping on an `Err` will already break everything,
/// because the panic handler might write to stderr whereas
/// Hedron still things, the application tries to make Hedron-
/// native syscalls.
fn wrap_hybrid_hedron_syscall<T, R>(actions: T) -> R
where
    T: Fn() -> R,
{
    let utcb = user_load_utcb_mut();
    utcb.set_head_tls(HEDRON_NATIVE_SYSCALL_MAGIC);
    let res = actions();
    utcb.set_head_tls(0);
    res
}

/// Like [`libhedron::syscall::sys_create_pd`] but for usage in hybrid foreign applications.
pub fn sys_hybrid_create_pd(
    passthrough_access: bool,
    cap_sel: CapSel,
    parent_pd_sel: CapSel,
    foreign_syscall_base: Option<CapSel>,
) -> SyscallResult {
    log::trace!("Executing hybrid foreign syscall: sys_create_pd");
    wrap_hybrid_hedron_syscall(|| {
        sys_create_pd(
            passthrough_access,
            cap_sel,
            parent_pd_sel,
            foreign_syscall_base,
        )
    })
}

/// Like [`libhedron::syscall::sys_create_global_ec`] but for usage in hybrid foreign applications.
pub fn sys_hybrid_create_global_ec(
    ec_cap_sel: CapSel,
    parent_pd_sel: CapSel,
    evt_base_sel: CapSel,
    cpu_num: u64,
    utcb_page_num: u64,
) -> SyscallResult {
    log::trace!("Executing hybrid foreign syscall: sys_create_global_ec");
    wrap_hybrid_hedron_syscall(|| {
        sys_create_global_ec(
            ec_cap_sel,
            parent_pd_sel,
            evt_base_sel,
            cpu_num,
            utcb_page_num,
        )
    })
}

/// Like [`libhedron::syscall::sys_create_local_ec`] but for usage in hybrid foreign applications.
pub fn sys_hybrid_create_local_ec(
    ec_cap_sel: CapSel,
    parent_pd_sel: CapSel,
    stack_ptr: u64,
    evt_base_sel: CapSel,
    cpu_num: u64,
    utcb_page_num: u64,
) -> SyscallResult {
    log::trace!("Executing hybrid foreign syscall: sys_create_local_ec");
    wrap_hybrid_hedron_syscall(|| {
        sys_create_local_ec(
            ec_cap_sel,
            parent_pd_sel,
            stack_ptr,
            evt_base_sel,
            cpu_num,
            utcb_page_num,
        )
    })
}

/// Like [`libhedron::syscall::sys_create_pt`] but for usage in hybrid foreign applications.
pub fn sys_hybrid_create_pt(
    new_pt_cap_sel: CapSel,
    own_pd_sel: CapSel,
    bound_ec_sel: CapSel,
    mtd: Mtd,
    instruction_pointer: *const u64,
) -> SyscallResult {
    log::trace!("Executing hybrid foreign syscall: sys_create_pt");
    wrap_hybrid_hedron_syscall(|| {
        sys_create_pt(
            new_pt_cap_sel,
            own_pd_sel,
            bound_ec_sel,
            mtd,
            instruction_pointer,
        )
    })
}

/// Like [`libhedron::syscall::sys_pt_ctrl`] but for usage in hybrid foreign applications.
pub fn sys_hybrid_pt_ctrl(pt_sel: CapSel, callback_argument: u64) -> SyscallResult {
    log::trace!("Executing hybrid foreign syscall: sys_pt_ctrl");
    wrap_hybrid_hedron_syscall(|| sys_pt_ctrl(pt_sel, callback_argument))
}

/// Like [`libhedron::syscall::sys_pd_ctrl_delegate`] but for usage in hybrid foreign applications.
pub fn sys_hybrid_pd_ctrl_delegate<Perm, Spec, ObjSpec>(
    source_pd: CapSel,
    dest_pd: CapSel,
    source_crd: Crd<Perm, Spec, ObjSpec>,
    dest_crd: Crd<Perm, Spec, ObjSpec>,
    flags: DelegateFlags,
) -> SyscallResult {
    log::trace!("Executing hybrid foreign syscall: sys_pd_ctrl_delegate");
    wrap_hybrid_hedron_syscall(|| {
        sys_pd_ctrl_delegate(source_pd, dest_pd, source_crd, dest_crd, flags)
    })
}

/// Like [`libhedron::syscall::sys_create_sc`] but for usage in hybrid foreign applications.
pub fn sys_hybrid_create_sc(
    cap_sel: CapSel,
    owned_pd_sel: CapSel,
    bound_ec_sel: CapSel,
    scheduling_params: Qpd,
) -> SyscallResult {
    log::trace!("Executing hybrid foreign syscall: sys_create_sc");
    wrap_hybrid_hedron_syscall(|| {
        sys_create_sc(cap_sel, owned_pd_sel, bound_ec_sel, scheduling_params)
    })
}
