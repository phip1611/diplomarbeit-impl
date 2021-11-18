//! Module for [`roottask_generic_portal_callback`].

use crate::process_mng::manager::PROCESS_MNG;
use crate::process_mng::process::Process;
use alloc::rc::Rc;
use libhrstd::kobjects::{
    PortalIdentifier,
    PtObject,
};
use libhrstd::libhedron::syscall::ipc::reply;
use libhrstd::libhedron::utcb::Utcb;

/// Describes a function, that handles a specific portal call.
/// # Parameters
/// * `pt` The [`PtObject`] that triggered the portal call
/// * `process` The [`Process`] where the call comes from
/// * `utcb` The [`Utcb`] of the portal
/// * `do_reply` If a `reply` should be made when the handler finishes, otherwise the code panics.
pub type PTCallHandler =
    fn(pt: &Rc<PtObject>, process: &Process, utcb: &mut Utcb, do_reply: &mut bool);

/// Common entry for all portals of the roottask. Multiplexes all portal calls through this function.
/// A call can either be a service all or an exception call.
pub fn roottask_generic_portal_callback(id: PortalIdentifier) -> ! {
    // log::trace!("generic portal callback called with argument: {}", id);

    let stack_top;
    let mut do_reply = false;

    // drop lock before reply()!
    {
        // log::debug!("trying to get lock for PROCESS_MNG");
        let mng = PROCESS_MNG.lock();
        // log::debug!("got lock");

        // find what portal triggered the request
        let pt = mng.lookup_portal(id).expect("there is no valid portal?!");
        // find what PdObject used the portal
        let calling_pd = if let Some(pd) = pt.delegated_to_pd().as_ref() {
            pd.clone()
        } else {
            pt.local_ec().pd()
        };
        let calling_process = mng
            .lookup_process(calling_pd.pid())
            .expect("unknown process!");
        // stack_top of the local EC that handles the call. Important for reply() syscall
        stack_top = pt.stack_top();
        // +++++++++++++++++++++++++++++++++++
        // here goes portal-specific handling

        let cb: PTCallHandler = if pt.ctx().is_exception_pt() {
            crate::roottask_exception::generic_error_exception_handler
        } else if pt.ctx().is_service_pt() {
            crate::services::handle_service_call
        } else {
            panic!("no portal callback handler known for given PT ctx");
        };

        cb(
            &pt,
            calling_process,
            pt.local_ec().utcb_mut(),
            &mut do_reply,
        );

        // log::debug!("specialized PT handler done");
        // +++++++++++++++++++++++++++++++++++
    }

    // important that all locks are dropped now!

    // not a convenient method in the PtObj itself, because the lock needs to be relased first!
    if do_reply {
        // log::debug!("reply now!");
        reply(stack_top);
    } else {
        // furthermore, the stack of the local EC would be corrupted afterwards
        panic!("panic without reply, end of game");
    }
}
