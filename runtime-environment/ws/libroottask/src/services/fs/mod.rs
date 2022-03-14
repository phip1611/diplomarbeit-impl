//! The in memory file system service currently lives inside the roottask.
//! This module connects the callable service portal with the actual functionality.

mod close;
mod lseek;
mod open;
mod read;
mod write;

use crate::process::Process;
use crate::pt_multiplex::roottask_generic_portal_callback;
use crate::services::fs::close::fs_service_impl_close;
use crate::services::fs::lseek::fs_service_impl_lseek;
use crate::services::fs::open::fs_service_impl_open;
use crate::services::fs::read::fs_service_impl_read;
use crate::services::fs::write::fs_service_impl_write;
use alloc::rc::Rc;
use libhrstd::kobjects::{
    LocalEcObject,
    PtCtx,
    PtObject,
};
use libhrstd::libhedron::CapSel;
use libhrstd::libhedron::Mtd;
use libhrstd::libhedron::Utcb;
use libhrstd::rt::services::fs::FsServiceRequest;
use libhrstd::service_ids::ServiceId;

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
    let file_server_request = utcb.load_data::<FsServiceRequest>().unwrap();
    match file_server_request {
        FsServiceRequest::Open(request) => fs_service_impl_open(&request, utcb, process),
        FsServiceRequest::Read(request) => fs_service_impl_read(&request, utcb, process),
        FsServiceRequest::Write(request) => fs_service_impl_write(&request, utcb, process),
        FsServiceRequest::Close(request) => fs_service_impl_close(&request, utcb, process),
        FsServiceRequest::LSeek(request) => fs_service_impl_lseek(&request, utcb, process),
    }

    *do_reply = true;
}
