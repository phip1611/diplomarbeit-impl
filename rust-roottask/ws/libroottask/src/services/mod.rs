//! All services the roottask provides via portals.

use crate::process_mng::process::Process;
use crate::stack::StaticStack;
use alloc::collections::BTreeSet;
use alloc::rc::{
    Rc,
    Weak,
};
use libhrstd::cap_space::root::RootCapSpace;
use libhrstd::cap_space::root::RootCapSpace::RootPd;
use libhrstd::cap_space::user::UserAppCapSpace;
use libhrstd::kobjects::{
    LocalEcObject,
    PtObject,
};
use libhrstd::libhedron::capability::{
    CrdObjPT,
    PTCapPermissions,
};
use libhrstd::libhedron::hip::HIP;
use libhrstd::libhedron::syscall::pd_ctrl::{
    pd_ctrl_delegate,
    DelegateFlags,
};
use libhrstd::libhedron::utcb::Utcb;
use libhrstd::mem::PageAligned;
use libhrstd::service_ids::ServiceId;
use libhrstd::sync::mutex::SimpleMutex;
use libhrstd::sync::static_global_ptr::StaticGlobalPtr;

pub mod stderr;
pub mod stdout;

static mut LOCAL_EC_STACK: StaticStack<16> = StaticStack::new();

/// The stack top of the local EC that handles all exception calls.
pub static LOCAL_EC_STACK_TOP: StaticGlobalPtr<u8> =
    StaticGlobalPtr::new(unsafe { LOCAL_EC_STACK.get_stack_top_ptr() });

/// Page-aligned UTCB for the service handler portal.
static mut UTCB: Utcb = Utcb::new();

/// Holds a weak reference to the local EC object used for handling service calls the roottask.
static LOCAL_EC: SimpleMutex<Option<Weak<LocalEcObject>>> = SimpleMutex::new(None);

/// Initializes stdout and stderr writers.
/// See [`stdout::StdoutWriter`] and [`stderr::StderrWriter`].
pub fn init_writers(hip: &HIP) {
    stdout::init_writer(hip);
    stderr::init_writer(hip);
}

/// Inits the local EC used for the portals. Now [`create_and_delegate_service_pts`] can be called.
pub fn init_services(root: &Process) {
    unsafe { LOCAL_EC_STACK.activate_guard_page(RootCapSpace::RootPd.val()) };
    // adds itself to the root process
    let ec = LocalEcObject::create(
        RootCapSpace::RootServiceLocalEc.val(),
        &root.pd_obj(),
        LOCAL_EC_STACK_TOP.val(),
        unsafe { UTCB.self_ptr() } as u64,
    );
    log::trace!(
        "Created local EC for all service calls (UTCB={:016x})",
        ec.utcb_addr()
    );

    // TODO rausfinden warum im portal handler the UTCB der falsche ist
    dbg!(unsafe { UTCB.self_ptr() });
    LOCAL_EC.lock().replace(Rc::downgrade(&ec));
}

/// Entry for all services of the roottask.
pub fn handle_service_call(
    pt: &Rc<PtObject>,
    process: &Process,
    utcb: &mut Utcb,
    do_reply: &mut bool,
) {
    log::debug!(
        "got service call for service {:?} from Process({}, {})",
        pt.ctx().service_id(),
        process.pid(),
        process.name()
    );
    let cb = match pt.ctx().service_id() {
        ServiceId::StdoutService => stdout::stdout_service_handler,
        ServiceId::StderrService => stderr::stderr_service_handler,
        _ => panic!("service not supported yet"),
    };
    log::debug!("trying to get lock");
    cb(pt, process, utcb, do_reply);
}

/// Creates the service PTs for a process inside the roottask. Install the PTs in the
/// target PD.
///
/// Call [`init_services`] once first.
pub fn create_and_delegate_service_pts(process: &Process) {
    log::debug!(
        "creating service PTs for process {}, {}",
        process.pid(),
        process.name()
    );

    let cap_base_sel = RootCapSpace::calc_service_pt_sel_base(process.pid());

    // local EC for all service calls
    let ec = LOCAL_EC.lock().as_ref().unwrap().upgrade().unwrap();

    let stdout_pt = stdout::create_service_pt(cap_base_sel, &ec);
    log::trace!("created stdout pt");
    pd_ctrl_delegate(
        RootCapSpace::RootPd.val(),
        process.pd_obj().cap_sel(),
        CrdObjPT::new(stdout_pt.cap_sel(), 0, PTCapPermissions::CALL),
        CrdObjPT::new(
            UserAppCapSpace::StdoutServicePT.val(),
            0,
            PTCapPermissions::CALL,
        ),
        DelegateFlags::default(),
    )
    .unwrap();
    stdout_pt.attach_delegated_to_pd(&process.pd_obj());
    process.pd_obj().attach_delegated_pt(stdout_pt);
    log::trace!("delegated stdout pt");

    let stderr_pt = stderr::create_service_pt(cap_base_sel, &ec);
    log::trace!("created stderr pt");
    pd_ctrl_delegate(
        RootCapSpace::RootPd.val(),
        process.pd_obj().cap_sel(),
        CrdObjPT::new(stderr_pt.cap_sel(), 0, PTCapPermissions::CALL),
        CrdObjPT::new(
            UserAppCapSpace::StderrServicePT.val(),
            0,
            PTCapPermissions::CALL,
        ),
        DelegateFlags::default(),
    )
    .unwrap();
    log::trace!("delegated stderr pt");
    stderr_pt.attach_delegated_to_pd(&process.pd_obj());
    process.pd_obj().attach_delegated_pt(stderr_pt);
}
