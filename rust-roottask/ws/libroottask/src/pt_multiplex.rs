//! Module for [`roottask_generic_portal_callback`].

use crate::process_mng::manager::{
    ProcessManager,
    PROCESS_MNG,
};
use crate::process_mng::process::Process;
use crate::roottask_exception::generic_error_exception_handler;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use libhrstd::kobjects::{
    PdObject,
    PortalIdentifier,
    PtCtx,
    PtObject,
};
use libhrstd::libhedron::event_offset::ExceptionEventOffset;
use libhrstd::libhedron::syscall::ipc::reply;
use libhrstd::libhedron::utcb::Utcb;
use libhrstd::sync::mutex::SimpleMutex;

/// Describes a function, that handles a specific portal call.
/// # Parameters
/// * `pt` The [`PtObject`] that triggered the portal call
/// * `process` The [`Process`] where the call comes from
/// * `utcb` The [`Utcb`] of the portal
/// * `do_reply` If a `reply` should be made when the handler finishes, otherwise the code panics.
pub type CallbackHook =
    fn(pt: &Rc<PtObject>, process: &Process, utcb: &mut Utcb, do_reply: &mut bool);

/// Map that enables code from the roottask to hook into the generic roottask portal
/// callback handler.
static PID_CALLBACK_MAP: SimpleMutex<BTreeMap<PortalIdentifier, CallbackHook>> =
    SimpleMutex::new(BTreeMap::new());

/// Adds an entry into [`PID_CALLBACK_MAP`].
pub fn add_callback_hook(pid: PortalIdentifier, fnc: CallbackHook) {
    PID_CALLBACK_MAP.lock().insert(pid, fnc);
}

/// Common entry for all portals of the roottask. Multiplexes all portal calls through this function.
pub fn roottask_generic_portal_callback(id: PortalIdentifier) -> ! {
    log::info!("generic portal callback called with argument: {}", id);

    let stack_top;
    let mut do_reply = false;

    // drop lock before reply()!
    {
        log::debug!("trying to get lock for PROCESS_MNG");
        let mng = PROCESS_MNG.lock();
        log::debug!("got lock");

        // find what portal triggered the request
        let pt = mng.lookup_portal(id).expect("there is no a valid portal?!");
        // find what PdObject used the portal
        let calling_pd = if let Some(pd) = pt.delegated_to_pd().as_ref() {
            pd.clone()
        } else {
            pt.local_ec().pd()
        };
        if let Some(ctx) = pt.ctx() {
            assert_eq!(
                calling_pd.pid(),
                ctx.exc_pid().1,
                "portal ctx pid doesn't match process"
            );
        }
        let calling_process = mng
            .lookup_process(calling_pd.pid())
            .expect("unknown process!");
        stack_top = pt.stack_top();
        // +++++++++++++++++++++++++++++++++++
        // here goes portal-specific handling
        let map = PID_CALLBACK_MAP.lock();
        if let Some(hook) = map.get(&id) {
            hook(
                &pt,
                calling_process,
                pt.local_ec().utcb_mut(),
                &mut do_reply,
            )
        } else {
            log::debug!("no specific portal callback handler registered");
        }
        // +++++++++++++++++++++++++++++++++++
    }

    // not a convenient method in the PtObj itself, because the lock needs to be relased first!
    if do_reply {
        log::debug!("reply now!");
        reply(stack_top);
    } else {
        // furthermore, the stack of the local EC would be corrupted afterwards
        panic!("panic without reply, end of game");
    }
}
