//! Module is responsible for providing the service to handle foreign syscalls.
use crate::process::Process;
use crate::process::SyscallAbi;
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
use libhrstd::libhedron::consts::NUM_CPUS;
use libhrstd::libhedron::Mtd;
use libhrstd::libhedron::Utcb;

mod linux;

pub fn handle_foreign_syscall(
    _pt: &Rc<PtObject>,
    process: &Rc<Process>,
    utcb: &mut Utcb,
    do_reply: &mut bool,
) {
    // Make sure that we don't accidentally overwrite stuff!
    // For example that we don't overwrite fs_base when we don't want to do it at all!
    utcb.exception_data_mut().mtd = Mtd::RIP_LEN | Mtd::RSP;

    // ### GENERIC PART: same for all foreign syscalls ###
    // see x86 spec: rcx will contain original user RIP
    // utcb_exc.rip = utcb_exc.rcx;
    let next_rip = utcb.exception_data().rcx;
    // hedron saves original user SP in r11
    let original_rsp = utcb.exception_data().r11;
    // ####################################################

    match process.syscall_abi() {
        // syscall implementations may not change these values
        SyscallAbi::Linux => {
            // EMULATE COSTS OF AN ADDITIONAL CHEAP IPC CALL AS DISCUSSED WITH NILS
            // THIS IS SIMILAR TO A MEDIATOR LIBRARY LINKED NEXT TO FOREIGN APPLICATIONS
            // DURING RUNTIME.
            libhrstd::libhedron::syscall::sys_call(RootCapSpace::RootRawEchoServicePt.val())
                .unwrap();
            // EMULATE COSTS END.
            let syscall = GenericLinuxSyscall::try_from(utcb.exception_data()).unwrap();
            log::trace!("linux syscall: {:?}", syscall.syscall_num());
            syscall.handle(utcb.exception_data_mut(), process);
        }
        _ => panic!("not implemented syscall ABI {:?}", process.syscall_abi()),
    }

    // ### GENERIC PART: same for all foreign syscalls ###
    utcb.exception_data_mut().rip = next_rip;
    utcb.exception_data_mut().rsp = original_rsp;
    // ####################################################

    log::trace!("outgoing MTD: {:?}", utcb.exception_data().mtd);

    *do_reply = true;
}

/// Creates the syscall handler PTs. The PD of a process gets `NUM_CPU` PTs.
pub fn create_and_delegate_syscall_handler_pts(process: &Process) {
    log::debug!(
        "creating syscall handler PTs for process {}, {}",
        process.pid(),
        process.name()
    );

    let base_sel = RootCapSpace::calc_foreign_syscall_pt_sel_base(process.pid());

    // local EC for all service calls
    let ec_lock = LOCAL_EC.lock();
    let ec_lock = ec_lock.as_ref().unwrap();

    for cpu in 0..NUM_CPUS as u64 {
        let cap_sel = base_sel + cpu;
        let pt = PtObject::create(
            cap_sel,
            ec_lock,
            // Julian: Niemals FPU hier; viel schneller und das wird nur für vCPUs benötigt
            Mtd::DEFAULT,
            roottask_generic_portal_callback,
            PtCtx::ForeignSyscall,
        );

        // delegate the foreign system call PT to the PD object of the new process
        PtObject::delegate(
            &pt,
            &process.pd_obj(),
            ForeignUserAppCapSpace::SyscallBasePt.val() + cpu,
        )
    }

    log::trace!("delegated foreign syscall handler PTs");
}
