//! Echo service. Replies to messages. Useful to do IPC benchmarking/measure IPC costs.

use crate::mem::VIRT_MEM_ALLOC;
use crate::process_mng::process::Process;
use crate::pt_multiplex::roottask_generic_portal_callback;
use crate::stack::StaticStack;
use alloc::rc::Rc;
use core::alloc::Layout;
use libhrstd::cap_space::root::RootCapSpace;
use libhrstd::kobjects::{
    LocalEcObject,
    PortalIdentifier,
    PtCtx,
    PtObject,
};
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::syscall::sys_reply;
use libhrstd::libhedron::Mtd;
use libhrstd::libhedron::{
    CapSel,
    Utcb,
};
use libhrstd::service_ids::ServiceId;
use libhrstd::sync::mutex::SimpleMutex;

static mut RAW_ECHO_SERVICE_STACK: StaticStack<4> = StaticStack::new();

static RAW_ECHO_SERVICE_LOCAL_EC: SimpleMutex<Option<Rc<LocalEcObject>>> = SimpleMutex::new(None);

/// Creates a local EC
pub fn init_echo_raw_service(root: &Process) {
    let mut lock = RAW_ECHO_SERVICE_LOCAL_EC.lock();
    assert!(lock.is_none(), "init only permitted once!");

    // make sure we reserve enough from virtual address space for the UTCB
    let utcb_addr = VIRT_MEM_ALLOC
        .lock()
        .next_addr(Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap());
    let echo_ec = LocalEcObject::create(
        RootCapSpace::RootRawEchoServiceEc.val(),
        &root.pd_obj(),
        unsafe { RAW_ECHO_SERVICE_STACK.get_stack_top_ptr() } as u64,
        utcb_addr,
    );

    lock.replace(echo_ec);
}

/// Creates the service PTs for the ECHO service and the RAW ECHO service for the roottask
/// itself.
pub(super) fn create_service_pts_fot_roottask(
    service_ec: &Rc<LocalEcObject>,
) -> (Rc<PtObject>, Rc<PtObject>) {
    // adds itself to the local EC
    let echo_service_pt = PtObject::create(
        RootCapSpace::RootEchoServicePt.val(),
        &service_ec,
        Mtd::empty(),
        roottask_generic_portal_callback,
        PtCtx::Service(ServiceId::EchoService),
    );

    let raw_echo_service_pt = PtObject::create(
        RootCapSpace::RootRawEchoServicePt.val(),
        &RAW_ECHO_SERVICE_LOCAL_EC.lock().as_ref().unwrap(),
        Mtd::empty(),
        raw_echo_pt_cb,
        PtCtx::Service(ServiceId::RawEchoService),
    );

    (echo_service_pt, raw_echo_service_pt)
}

/// Creates the service PTs for the ECHO service and the RAW ECHO service.
/// Only returns the service PT used by my PT multiplexing mechanism.
pub fn create_service_pts(
    base_cap_sel: CapSel,
    service_ec: &Rc<LocalEcObject>,
) -> (Rc<PtObject>, Rc<PtObject>) {
    // adds itself to the local EC
    let echo_service_pt = PtObject::create(
        base_cap_sel + ServiceId::EchoService.val(),
        &service_ec,
        Mtd::empty(),
        roottask_generic_portal_callback,
        PtCtx::Service(ServiceId::EchoService),
    );

    let raw_echo_service_pt = PtObject::create(
        base_cap_sel + ServiceId::RawEchoService.val(),
        &RAW_ECHO_SERVICE_LOCAL_EC.lock().as_ref().unwrap(),
        Mtd::empty(),
        raw_echo_pt_cb,
        PtCtx::Service(ServiceId::RawEchoService),
    );

    (echo_service_pt, raw_echo_service_pt)
}

/// Handler for the normal echo PT.
pub fn echo_service_handler(
    _pt: &Rc<PtObject>,
    _process: &Process,
    _utcb: &mut Utcb,
    do_reply: &mut bool,
) {
    *do_reply = true;
}

/// Cheap handler for the raw echo service PT.
fn raw_echo_pt_cb(_: PortalIdentifier) -> ! {
    log::trace!("raw echo pt called!");
    sys_reply(unsafe { RAW_ECHO_SERVICE_STACK.get_stack_top_ptr() } as u64)
}
