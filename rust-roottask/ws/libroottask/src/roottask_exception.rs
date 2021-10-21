//! General exception-handling for roottask. Registers a portal for each single possible
//! exception. Other parts of the roottask has the option to register themselves
//! as handler, without further interaction with the kernel
//! (i.e. dedicated syscalls to create new PTs).

use crate::capability_space::RootCapSpace;
use crate::stack::StaticStack;
use arrayvec::ArrayString;
use core::convert::TryFrom;
use core::fmt::Write;
use libhrstd::libhedron::capability::CapSel;
use libhrstd::libhedron::consts::NUM_EXC;
use libhrstd::libhedron::event_offset::ExceptionEventOffset;
use libhrstd::libhedron::hip::HIP;
use libhrstd::libhedron::mtd::Mtd;
use libhrstd::libhedron::syscall::create_ec::create_local_ec;
use libhrstd::libhedron::syscall::create_pt::create_pt;
use libhrstd::libhedron::syscall::pd_ctrl::pd_ctrl_delegate;
use libhrstd::libhedron::syscall::pt_ctrl::pt_ctrl;
use libhrstd::libhedron::utcb::Utcb;
use libhrstd::mem::PageAligned;
use libhrstd::portal_identifier::PortalIdentifier;
use libhrstd::process::ROOTTASK_PROCESS_PID;
use libhrstd::sync::fakelock::FakeLock;
use libhrstd::sync::mutex::SimpleMutex;
use libhrstd::sync::static_global_ptr::StaticGlobalPtr;
use libhrstd::util::ansi::{
    AnsiStyle,
    Color,
    TextStyle,
};

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

/// UTCB for the exception handler portal.
static mut EXCEPTION_UTCB: PageAligned<Utcb> = PageAligned::new(Utcb::new());

/// Payload for generic exceptions of the roottask itself.
/// Useful to better trace the origin of portal calls.
const PORTAL_ID_ROOTTASK_PAYLOAD: u64 = 0x001001001001;

/// Map that stores if specialized exception handlers are available.
/// Each handler callback must either panic or return with a `REPLY`
/// syscall.
// TODO proper locking!
static SPECIALIZES_EXCEPTION_HANDLER_MAP: FakeLock<
    [Option<fn(PortalIdentifier, &mut Utcb) -> !>; NUM_EXC],
> = FakeLock::new([None; NUM_EXC]);

/// Initializes a local EC and N portals to cover N exceptions.
/// All exceptions are considered as unrecoverable in this roottask.
/// Therefore, they panic. See [`roottask_lib::hedron::event_offset::ExceptionEventOffset`]
/// to see possible exceptions.
///
/// If it fails, the program aborts.
pub fn init(hip: &HIP) {
    create_local_ec(
        RootCapSpace::RootExceptionLocalEc.val(),
        hip.root_pd(),
        LOCAL_EXC_EC_STACK_TOP.val(),
        RootCapSpace::ExceptionEventBase.val(),
        0,
        unsafe { EXCEPTION_UTCB.page_num() } as u64,
    )
    .unwrap();

    unsafe {
        CALLBACK_STACK.activate_guard_page(hip.root_pd());
    }
    log::info!("created local ec for exception handling; guard page is active");
    log::trace!(
        "local exception handler ec stack top  (incl): {:016x?}",
        unsafe { CALLBACK_STACK.get_stack_top_ptr() as u64 }
    );

    // I iterate here over all available/reserved capability selectors for exceptionss.
    // This is relative to the event base selector. For the roottask/root protection domain,
    // it is 0 (See RootCapSpace::ExceptionEventBase.val()).
    // We install an actual kernel object of type portal at the given indices.

    // iterate from 0 to 32 (exception capability selector space)
    for excp_offset in 0..NUM_EXC {
        let portal_cap_sel = RootCapSpace::ExceptionEventBase.val() + excp_offset as CapSel;
        create_exc_handler_portal(
            RootCapSpace::ExceptionEventBase.val() + excp_offset as CapSel,
            PortalIdentifier::new(
                portal_cap_sel,
                ROOTTASK_PROCESS_PID,
                PORTAL_ID_ROOTTASK_PAYLOAD,
            ),
        );
    }
}

/// Registers a special exception handler for a specific exception.
pub fn register_specialized_exc_handler(
    excp_id: ExceptionEventOffset,
    fnc: fn(PortalIdentifier, &mut Utcb) -> !,
) {
    let mut map = SPECIALIZES_EXCEPTION_HANDLER_MAP.get_mut();
    if map[excp_id.val() as usize].is_some() {
        panic!(
            "already registered a special exception handler for exception = {:?}",
            excp_id
        );
    }
    map[excp_id.val() as usize] = Some(fnc);
}

/// Creates a new portal inside the root PD, that is associated with the generic callback handler
/// function [`portal_cb_exc_handler`]. Associates the portal with the local EC and its stack and
/// UTCB. Ensures the right [`PortalIdentifier`] gets passed to the portal.
///
/// # Parameters
/// * `portal_sel` Portal selector in the cap space of the roottask
pub fn create_exc_handler_portal(portal_sel: CapSel, pid: PortalIdentifier) {
    create_pt(
        portal_sel,
        RootCapSpace::RootPd.val(),
        RootCapSpace::RootExceptionLocalEc.val(),
        Mtd::all(),
        portal_cb_exc_handler as *const u64,
    )
    .unwrap();
    pt_ctrl(portal_sel, pid.val()).unwrap();
}

/// Handler for portal calls triggered by exceptions from Hedron. The handler itself is generic
/// for all applications, but specialized handlers inside the roottask can register themselves via
/// [`register_specialized_exc_handler`].
///
/// By default the handler aborts the program by throwing a Rust panic.
fn portal_cb_exc_handler(portal_id: PortalIdentifier) -> ! {
    log::debug!("caught via exception portal: {:?}", portal_id);
    // TODO use real lock!
    // ATTENTION; DEAD LOCK POTENTIAL!
    let mut map = SPECIALIZES_EXCEPTION_HANDLER_MAP.get_mut();

    if let Some(fnc) = map[portal_id.exc() as usize] {
        log::debug!("forwarding to specialized exception handler");
        fnc(portal_id, unsafe { &mut EXCEPTION_UTCB })
    } else {
        log::debug!("no specialized exception handler available");

        let mut buf = ArrayString::<128>::new();
        write!(&mut buf, "{:?}", portal_id).unwrap();
        panic!(
            "Mayday, caught exception id={} - aborting program\n{:#?}",
            AnsiStyle::new()
                .foreground_color(Color::Red)
                .text_style(TextStyle::Bold)
                .msg(buf.as_str()),
            unsafe { EXCEPTION_UTCB.exception_data() }
        );
        // entweder return mit reply syscall
        // oder panic => game over

        // Problem bei normalem return: keine rücksprugnadresse
    }
}
