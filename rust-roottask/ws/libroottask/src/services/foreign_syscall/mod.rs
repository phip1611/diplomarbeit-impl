//! Module is responsible for providing the service to handle foreign syscalls.
use crate::process_mng::process::Process;
use crate::process_mng::syscall_abi::SyscallAbi;
use crate::pt_multiplex::roottask_generic_portal_callback;
use crate::services::foreign_syscall::linux::GenericLinuxSyscall;
use crate::services::LOCAL_EC;
use alloc::rc::Rc;
use libhrstd::cap_space::root::RootCapSpace;
use libhrstd::cap_space::user::ForeignUserAppCapSpace;
use libhrstd::kobjects::{
    PtCtx,
    PtObject,
};
use libhrstd::libhedron::capability::{
    CrdObjPT,
    MemCapPermissions,
    PTCapPermissions,
};
use libhrstd::libhedron::consts::NUM_CPUS;
use libhrstd::libhedron::mtd::Mtd;
use libhrstd::libhedron::syscall::pd_ctrl::pd_ctrl_delegate;
use libhrstd::libhedron::utcb::{
    Utcb,
    UtcbDataException,
};
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

mod linux;

pub fn handle_foreign_syscall(
    _pt: &Rc<PtObject>,
    process: &Process,
    utcb: &mut Utcb,
    do_reply: &mut bool,
) {
    match process.syscall_abi() {
        SyscallAbi::Linux => {
            let syscall = GenericLinuxSyscall::try_from(utcb.exception_data()).unwrap();
            log::debug!(
                "Got {:?} syscall at RIP=0x{:x}",
                syscall,
                utcb.exception_data().rip
            );
            syscall.handle(utcb.exception_data_mut());
        }
        _ => panic!("not implemented syscall ABI {:?}", process.syscall_abi()),
    }
    *do_reply = true;
}

/// Creates the syscall handler PTs. The PD of a process gets `NUM_CPU` PTs.
pub fn create_and_delegate_syscall_handler_pts(process: &Process) {
    log::debug!(
        "creating syscall handler PTs for process {}, {}",
        process.pid(),
        process.name()
    );

    let base_sel = RootCapSpace::calc_foreign_syscall_pt_sel_base(process.pid(), 0);

    // local EC for all service calls
    let ec = LOCAL_EC.lock().as_ref().unwrap().upgrade().unwrap();

    for cpu in 0..NUM_CPUS as u64 {
        let cap_sel = base_sel + cpu;
        let pt = PtObject::create(
            cap_sel,
            &ec,
            Mtd::all(),
            roottask_generic_portal_callback,
            PtCtx::ForeignSyscall,
        );
        pt.attach_delegated_to_pd(&process.pd_obj());
        process.pd_obj().attach_delegated_pt(pt);
    }

    CrdDelegateOptimizer::new(
        base_sel,
        ForeignUserAppCapSpace::SyscallBasePt.val(),
        NUM_CPUS,
    )
    .pts(
        process.parent().unwrap().pd_obj().cap_sel(),
        process.pd_obj().cap_sel(),
    );
    log::trace!("delegated foreign syscall handler PTs");
}
