//! General exception-handling for roottask. Registers a portal for each single possible
//! exception. Other parts of the roottask has the option to register themselves
//! as handler, without further interaction with the kernel.
//!
//! Example:
//! The code that creates all exception portals registers a specialized [`PTCallHandler`].
//! The code referenced from there has again the option to look into a data structure
//! to delegate the call to an even more specialized handler (e.g. startup exception).

use crate::mem::VIRT_MEM_ALLOC;
use crate::process::Process;
use crate::pt_multiplex::{
    roottask_generic_portal_callback,
    PTCallHandler,
};
use crate::stack::StaticStack;
use alloc::rc::{
    Rc,
    Weak,
};
use core::alloc::Layout;
use core::convert::TryFrom;
use libhrstd::cap_space::root::RootCapSpace;
use libhrstd::kobjects::PtCtx::Exception;
use libhrstd::kobjects::{
    LocalEcObject,
    PtObject,
};
use libhrstd::libhedron::consts::NUM_EXC;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::CapSel;
use libhrstd::libhedron::ExceptionEventOffset;
use libhrstd::libhedron::Mtd;
use libhrstd::libhedron::Utcb;
use libhrstd::process::consts::ROOTTASK_PROCESS_PID;
use libhrstd::sync::mutex::SimpleMutex;
use libhrstd::sync::static_global_ptr::StaticGlobalPtr;

/// Used as stack for the exception handler callback function. Must be either mutable
/// or manually placed in a writeable section in the file. Otherwise we get a page fault.
///
/// It is 4 pages long. This is enough probably. panic!() and log::*! needs 4096 bytes each.
///
/// **Note:** Out of the box, there is no guard page protection. Must be set up manually.
///
/// **Size:** Exception handler relies on panic and logging. Both require
///           1024 respectively 4096 bytes of stack for the formatting of the message.
///           Therefore, with 8KiB (2 pages) we are safe. Also note please: In Cargo.toml
///           I wrote that the opt-level is 1 for dev-builds. This significantly
///           reduces stack usage by Rust. Without it, even stacks that seem large
///           enough lead to memory corruptions.
///
// #[link_section = ".data"] (=rw) with "static VARNAME" or "static mut"
static mut CALLBACK_STACK: StaticStack<16> = StaticStack::new();

/// The stack top of the local EC that handles all exception calls.
pub static LOCAL_EXC_EC_STACK_TOP: StaticGlobalPtr<u8> =
    StaticGlobalPtr::new(unsafe { CALLBACK_STACK.get_stack_top_ptr() });

/// Holds a weak reference to the local EC object used for handling exceptions inside
/// the roottask.
static EXCEPTION_LOCAL_EC: SimpleMutex<Option<Weak<LocalEcObject>>> = SimpleMutex::new(None);

/// Map that helps to forward certain exceptions to specialized exception handlers, if are available.
/// The generic PT entry callback sends all exceptions to the callback of this module. This module
/// itself can further delegate the responsibility for handling the exception.
static SPECIALIZES_EXCEPTION_HANDLER_MAP: SimpleMutex<[Option<PTCallHandler>; NUM_EXC]> =
    SimpleMutex::new([None; NUM_EXC]);

/// Initializes a local EC and N portals to cover N exceptions for the roottask.
pub fn init(root_process: &Process) {
    // make sure we reserve enough from virtual address space for the UTCB
    let utcb_addr = VIRT_MEM_ALLOC
        .lock()
        .next_addr(Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap());

    // adds itself to the root process
    let exception_local_ec = LocalEcObject::create(
        RootCapSpace::RootExceptionLocalEc.val(),
        &root_process.pd_obj(),
        LOCAL_EXC_EC_STACK_TOP.val(),
        utcb_addr,
    );
    EXCEPTION_LOCAL_EC
        .lock()
        .replace(Rc::downgrade(&exception_local_ec));
    unsafe {
        CALLBACK_STACK.activate_guard_page(RootCapSpace::RootPd.val());
    }

    log::debug!("created local ec for exception handling; guard page is active");
    log::trace!(
        "local exception handler ec stack top  (incl): {:016x?}",
        unsafe { CALLBACK_STACK.get_stack_top_ptr() as u64 }
    );

    // I iterate here over all available/reserved capability selectors for exceptionss.
    // This is relative to the event base selector. For the roottask/root protection domain,
    // it is 0 (See RootCapSpace::ExceptionEventBase.val()).
    // We install an actual kernel object of type portal at the given indices.

    // iterate from 0 to 32 (exception capability selector space)
    for exc_offset in 0..NUM_EXC {
        // TODO maybe this should not register the startup exception?!
        //  or the roottask_exception module offers to register custom hooks too.. maybe the nicer way!
        let portal_cap_sel = RootCapSpace::ExceptionEventBase.val() + exc_offset as CapSel;
        create_exc_pt_for_process(exc_offset as u64, portal_cap_sel);
    }
}

/// Registers a special exception handler for a specific exception.
/// See [`SPECIALIZES_EXCEPTION_HANDLER_MAP`].
pub fn register_specialized_exc_handler(excp_id: ExceptionEventOffset, fnc: PTCallHandler) {
    let mut map = SPECIALIZES_EXCEPTION_HANDLER_MAP.lock();
    if map[excp_id.val() as usize].is_some() {
        panic!(
            "already registered a special exception handler for exception = {:?}",
            excp_id
        );
    }
    map[excp_id.val() as usize] = Some(fnc);
}

/// Creates a new exception portal, that is bound to the local EC defined in this module.
/// It needs to know the target process/PID, so that the roottask exception handler knows
/// what process triggered a specific exception.
///
/// Makes sure that the correct callback hook gets called for this portal too.
///
/// # Parameters
/// * `portal_cap_sel` Capability selector for portal in root PD
/// * `process_id` Process ID, where this exception portal gets installed/delegated.
pub fn create_exc_pt_for_process(exc_offset: u64, portal_cap_sel: CapSel) -> Rc<PtObject> {
    let ec = EXCEPTION_LOCAL_EC
        .lock()
        .as_ref()
        .expect("call init first")
        .upgrade()
        .unwrap();
    let pt = PtObject::create(
        portal_cap_sel,
        &ec,
        Mtd::DEFAULT,
        roottask_generic_portal_callback,
        Exception(exc_offset),
    );
    pt
}

/// Handler that handles all error exceptions that Hedron can trigger, both from the roottask or
/// other processes.
///
/// Doesn't reply, because this is done a layer above.
pub fn generic_error_exception_handler(
    pt: &Rc<PtObject>,
    process: &Rc<Process>,
    utcb: &mut Utcb,
    do_reply: &mut bool,
) {
    // All exception portals live in the roottask, therefore their parent is the roottask.
    // Therefore we need to get the target PID (the process that triggered an exception) from the context.
    let is_roottask = process.pid() == ROOTTASK_PROCESS_PID;
    let exc = ExceptionEventOffset::try_from(pt.ctx().exc()).unwrap();
    if is_roottask {
        log::debug!(
            "caught exception {:?} from roottask via pt={}",
            exc,
            pt.portal_id()
        );
    } else {
        log::debug!(
            "caught exception {:?} from pid={} via pt={}",
            exc,
            process.pid(),
            pt.portal_id()
        );
    }

    let map = SPECIALIZES_EXCEPTION_HANDLER_MAP.lock();
    if let Some(handler) = map[exc.val() as usize] {
        log::debug!("use specialized exception handler");
        handler(pt, process, utcb, do_reply);
    } else {
        log::debug!("use generic (=panic) exception handler");
        *do_reply = false;
        panic!(
            "can't handle exception {:?} at rip={:?} from process {} ({}) currently - game over\n{:#?}",
            exc,
            utcb.exception_data().rip as *const u8,
            process.pid(),
            process.name(),
            utcb.exception_data(),
        );
    }
}
