use crate::process_mng::process::Process;
use crate::pt_multiplex::roottask_generic_portal_callback;
use alloc::alloc::Layout;
use alloc::rc::Rc;
use core::alloc::Allocator;
use core::cmp::max;
use core::ptr::NonNull;
use core::sync::atomic::Ordering;
use libhrstd::kobjects::{
    LocalEcObject,
    PtCtx,
    PtObject,
};
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::Mtd;
use libhrstd::libhedron::{
    CapSel,
    MemCapPermissions,
};

use libhrstd::libhedron::Utcb;
use libhrstd::rt::services::allocate::AllocRequest;
use libhrstd::service_ids::ServiceId;
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

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
    // TODO ALL OF THIS IS Q&D and needs a nicer refactoring!
    //  currently allocates only in pages; very inefficient and hacky

    let layout = utcb.load_data::<AllocRequest>().unwrap();
    // ensure that we only map even pages; Q&D
    let layout = Layout::from_size_align(layout.size(), max(PAGE_SIZE, layout.align())).unwrap();
    let ptr: NonNull<[u8]> = alloc::alloc::Global.allocate_zeroed(layout).unwrap();
    // allocate in Roottask (i.e. physical memory) and delegete mem capability to new process
    utcb.store_data(&process.heap_ptr().load(Ordering::SeqCst))
        .unwrap();

    // Q&D: map the page directly,
    let page_addr = ptr.as_ptr() as *const u8 as usize & !0xfff;
    let page_num = page_addr / PAGE_SIZE;

    let page_count = if layout.size() % PAGE_SIZE == 0 {
        layout.size() / PAGE_SIZE
    } else {
        (layout.size() / PAGE_SIZE) + 1
    };

    CrdDelegateOptimizer::new(
        page_num as u64,
        process.heap_ptr().load(Ordering::SeqCst) / PAGE_SIZE as u64,
        page_count,
    )
    .mmap(
        process.parent().unwrap().pd_obj().cap_sel(),
        process.pd_obj().cap_sel(),
        MemCapPermissions::READ | MemCapPermissions::WRITE,
    );

    // update heap pointer
    process.heap_ptr().store(
        process.heap_ptr().load(Ordering::SeqCst) + page_count as u64 * PAGE_SIZE as u64,
        Ordering::SeqCst,
    );

    *do_reply = true;
}
