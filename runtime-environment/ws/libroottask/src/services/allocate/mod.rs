use crate::process::Process;
use crate::pt_multiplex::roottask_generic_portal_callback;
use alloc::rc::Rc;
use libhrstd::kobjects::{
    LocalEcObject,
    PtCtx,
    PtObject,
};
use libhrstd::libhedron::CapSel;
use libhrstd::libhedron::Mtd;
use libhrstd::libhedron::Utcb;
use libhrstd::rt::services::allocate::AllocRequest;
use libhrstd::service_ids::ServiceId;

/// Creates a new ALLOCATOR service PT, which can be delegated to a new process.
pub fn create_service_pt(base_cap_sel: CapSel, ec: &Rc<LocalEcObject>) -> Rc<PtObject> {
    let service = ServiceId::AllocateService;
    // adds itself to the local EC for services
    PtObject::create(
        base_cap_sel + service.val(),
        &ec,
        Mtd::empty(),
        roottask_generic_portal_callback,
        PtCtx::Service(service),
    )
}

/// Handles the functionality of the ALLOCATOR Portal.
pub fn allocate_service_handler(
    _pt: &Rc<PtObject>,
    process: &Process,
    utcb: &mut Utcb,
    do_reply: &mut bool,
) {
    let alloc_request = utcb.load_data::<AllocRequest>().unwrap();

    log::trace!("alloc_request: {alloc_request:?}");

    if alloc_request.is_allocation() {
        let addr = process
            .memory_manager_mut()
            .mmap(alloc_request.to_layout(), process);
        utcb.store_data(&addr).unwrap();
    } else {
        let addr = alloc_request.ptr().unwrap();
        process.memory_manager_mut().munmap(addr, process);
    }

    /*let brk = process
        .memory_manager_mut()
        .increase_break_by(layout.size(), process);

    utcb.store_data(&brk).unwrap();*/

    *do_reply = true;
}
