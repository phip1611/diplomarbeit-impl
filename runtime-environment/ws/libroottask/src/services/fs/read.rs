use crate::mem::VIRT_MEM_ALLOC;
use crate::process_mng::process::Process;
use core::alloc::Layout;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::{
    MemCapPermissions,
    Utcb,
};
use libhrstd::mem::calc_page_count;
use libhrstd::rt::services::fs::FsReadRequest;
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

/// Implements the fs read service functionality that is accessible via the FS portal.
pub(super) fn fs_service_impl_read(request: &FsReadRequest, utcb: &mut Utcb, process: &Process) {
    // data from the file system
    let read_bytes = libfileserver::fs_read(process.pid(), request.fd(), request.count()).unwrap();

    // early return if EOF reached
    if read_bytes.len() == 0 {
        utcb.store_data(&read_bytes.len()).unwrap();
        return;
    }

    // now map the data to a user destination
    let u_addr = request.user_ptr();
    let u_addr_page_offset = u_addr & 0xfff;
    let u_page_num = u_addr / PAGE_SIZE;
    let required_bytes = u_addr_page_offset + request.count();
    let page_count = calc_page_count(required_bytes);

    // get virt address to map the user memory into the roottask
    let r_mapping_addr = VIRT_MEM_ALLOC
        .lock()
        .next_addr(Layout::from_size_align(required_bytes, PAGE_SIZE).unwrap());
    let r_mapping_page_num = r_mapping_addr / PAGE_SIZE as u64;

    // map memory from user app into root task
    CrdDelegateOptimizer::new(u_page_num as u64, r_mapping_page_num, page_count).mmap(
        process.pd_obj().cap_sel(),
        process.parent().unwrap().pd_obj().cap_sel(),
        MemCapPermissions::READ | MemCapPermissions::WRITE,
    );
    // memory in roottask where I mapped the user memory
    let r_dest_ptr = (r_mapping_addr + u_addr_page_offset as u64) as *mut u8;
    unsafe {
        core::ptr::copy_nonoverlapping(read_bytes.as_ptr(), r_dest_ptr, request.count());
    }

    // read bytes
    utcb.store_data(&read_bytes.len()).unwrap();
}
