//! The in memory file system service currently lives inside the roottask.

use crate::mem::VIRT_MEM_ALLOC;
use crate::process_mng::process::Process;
use crate::pt_multiplex::roottask_generic_portal_callback;
use alloc::rc::Rc;
use alloc::string::ToString;
use core::alloc::Layout;
use libhrstd::kobjects::{
    LocalEcObject,
    PtCtx,
    PtObject,
};
use libhrstd::libhedron::capability::{
    CapSel,
    MemCapPermissions,
};
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::mtd::Mtd;
use libhrstd::libhedron::utcb::Utcb;
use libhrstd::rt::services::fs::service::FsServiceRequest;
use libhrstd::service_ids::ServiceId;
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

/// Creates a new FILE SYSTEM service PT, which can be delegated to a new process.
pub fn create_service_pt(base_cap_sel: CapSel, ec: &Rc<LocalEcObject>) -> Rc<PtObject> {
    let service = ServiceId::FileSystemService;
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
pub fn fs_service_handler(
    _pt: &Rc<PtObject>,
    process: &Process,
    utcb: &mut Utcb,
    do_reply: &mut bool,
) {
    dbg!(utcb.untyped_items_count());
    let file_server_request = utcb.load_data::<FsServiceRequest>().unwrap();
    dbg!(&file_server_request);
    match file_server_request {
        FsServiceRequest::Open(request) => {
            let fd = libfileserver::fs_open(
                process.pid(),
                request.path().to_string(),
                request.flags(),
                request.umode(),
            );
            utcb.store_data(&fd).unwrap();
        }
        FsServiceRequest::Read(request) => {
            let read_bytes =
                libfileserver::fs_read(process.pid(), request.fd(), request.count()).unwrap();
            let user_ptr = request.user_ptr();
            let user_ptr_page_offset = user_ptr & 0xfff;
            let user_page = user_ptr / PAGE_SIZE;
            let mapping_dest = VIRT_MEM_ALLOC
                .lock()
                .next_addr(Layout::from_size_align(request.count(), PAGE_SIZE).unwrap());
            let page_count = if request.count() % PAGE_SIZE == 0 {
                request.count() / PAGE_SIZE
            } else {
                request.count() / PAGE_SIZE + 1
            };
            CrdDelegateOptimizer::new(user_page as u64, mapping_dest, page_count).mmap(
                process.pd_obj().cap_sel(),
                process.parent().unwrap().pd_obj().cap_sel(),
                MemCapPermissions::READ | MemCapPermissions::WRITE,
            );
            let dest_ptr = (mapping_dest + user_ptr_page_offset as u64) as *mut u8;
            unsafe {
                core::ptr::copy_nonoverlapping(read_bytes.as_ptr(), dest_ptr, request.count());
            }

            // read bytes
            utcb.store_data(&read_bytes.len()).unwrap();
        }
        FsServiceRequest::Write(request) => {
            libfileserver::fs_write(
                process.pid(),
                request.fd(),
                // currently don't support user ptr read
                request.data().embedded_slice(),
            )
            .unwrap();

            utcb.store_data(&request.data().embedded_slice().len())
                .unwrap();
        }
        FsServiceRequest::Close(request) => {
            libfileserver::fs_close(process.pid(), request.fd()).unwrap();
        }
        FsServiceRequest::LSeek(request) => {
            libfileserver::fs_lseek(process.pid(), request.fd(), request.offset() as usize)
                .unwrap();
        }
    }

    *do_reply = true;
}
