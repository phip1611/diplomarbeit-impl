//! General exception-handling for roottask. Registers a portal for each single possible
//! exception. Other parts of the roottask has the option to register themselves
//! as handler, without further interaction with the kernel
//! (i.e. dedicated syscalls to create new PTs).

use crate::capability_space::RootCapabilitySpace;
use crate::stack::StaticStack;
use arrayvec::ArrayString;
use core::convert::TryFrom;
use core::fmt::Write;
use libhrstd::libhedron::capability::CapSel;
use libhrstd::libhedron::event_offset::ExceptionEventOffset;
use libhrstd::libhedron::hip::HIP;
use libhrstd::libhedron::mtd::Mtd;
use libhrstd::libhedron::syscall::create_ec::create_local_ec;
use libhrstd::libhedron::syscall::create_pt::create_pt;
use libhrstd::libhedron::syscall::pt_ctrl::pt_ctrl;
use libhrstd::libhedron::utcb::Utcb;
use libhrstd::mem::PageAligned;
use libhrstd::sync::mutex::SimpleMutex;
use libhrstd::util::ansi::{
    AnsiStyle,
    Color,
    TextStyle,
};

/// The root task has 0 as event selector base. This means, initially
/// capability selectors 0..32 refer to a null capability, but can be used
/// for exception handling. To get the offset for the corresponding
/// event, see [`roottask_lib::hedron::event_offset::ExceptionEventOffset`].
///
/// The number of exceptions is also in [`roottask_lib::hedron::hip::HIP`] (field `num_exc_sel`).
pub const ROOT_EXC_EVENT_BASE: CapSel = 0;

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
static mut CALLBACK_STACK: StaticStack<4> = StaticStack::new();

/// UTCB for the exception handler portal.
static mut EXCEPTION_UTCB: PageAligned<Utcb> = PageAligned::new(Utcb::new());

/// Number of possible exceptions
const NUM_EXC: usize = (RootCapabilitySpace::ExceptionEnd.val() + 1) as usize;

/// Map that stores if specialized exception handlers are available.
static SPECIALIZES_EXCEPTION_HANDLER_MAP: SimpleMutex<
    [Option<fn(ExceptionEventOffset, &mut Utcb) -> !>; NUM_EXC],
> = SimpleMutex::new([None; NUM_EXC]);

/// Initializes a local EC and N portals to cover N exceptions.
/// All exceptions are considered as unrecoverable in this roottask.
/// Therefore, they panic. See [`roottask_lib::hedron::event_offset::ExceptionEventOffset`]
/// to see possible exceptions.
///
/// If it fails, the program aborts.
pub fn init(hip: &HIP) {
    create_local_ec(
        RootCapabilitySpace::RootExceptionLocalEc.val(),
        hip.root_pd(),
        unsafe { CALLBACK_STACK.get_stack_top_ptr() } as u64,
        ROOT_EXC_EVENT_BASE,
        0,
        unsafe { EXCEPTION_UTCB.page_num() } as u64,
    )
    .unwrap();

    unsafe {
        CALLBACK_STACK.activate_guard_page(hip.root_pd());
    }
    log::info!("created local ec for exception handling; guard page is active");

    // I iterate here over all available/reserved capability selectors for exceptionss.
    // This is relative to the event base selector. For the roottask/root protection domain,
    // it is 0 (See ROOT_EXC_EVENT_BASE).
    // We install an actual kernel object of type portal at the given indices.

    let from = RootCapabilitySpace::ExceptionEventBase.val();
    let to = hip.num_exc_sel() as u64;
    // iterate from 0 to 32 (exception capability selector space)
    for excp_offset in from..to {
        // equivalent to the enum variants from ExceptionEventOffset
        let portal_cap_sel = ROOT_EXC_EVENT_BASE + excp_offset as CapSel;

        // create portal for each exception
        create_pt(
            portal_cap_sel,
            hip.root_pd(),
            RootCapabilitySpace::RootExceptionLocalEc.val(),
            // Mtd::DEFAULT,
            Mtd::all(),
            general_exception_handler as *const u64,
        )
        .unwrap();
        // give each portal the proper callback argument / id.
        pt_ctrl(
            portal_cap_sel,
            // wert der in %rdi im evt handler ankommt (first arg)
            excp_offset as u64,
        )
        .unwrap();

        // logging, not important
        /*{
            let mut msg = ArrayString::<128>::from("created PT for exception=").unwrap();
            let exc = ExceptionEventOffset::try_from(excp_offset as u64);
            if let Ok(exc) = exc {
                write!(&mut msg, "{:?}({})", exc, excp_offset).unwrap();
            } else {
                write!(&mut msg, "Unknown({:?})", excp_offset).unwrap();
            }
            log::trace!("{}", msg);
        }*/
    }
}

/// Registers a special exception handler for a specific exception.
pub fn register_specialized_exc_handler(
    excp_id: ExceptionEventOffset,
    fnc: fn(ExceptionEventOffset, &mut Utcb) -> !,
) {
    let mut map = SPECIALIZES_EXCEPTION_HANDLER_MAP.lock();
    if map[excp_id.val() as usize].is_some() {
        panic!(
            "already registered a special exception handler for exception = {:?}",
            excp_id
        );
    }
    map[excp_id.val() as usize] = Some(fnc);
}

/// General exception handler for all x86 exceptions that can happen + Hedron specific exceptions.
/// The handler aborts the program if an exception occurs. Panics the Rust program.
fn general_exception_handler(exception_id: u64) -> ! {
    let id = ExceptionEventOffset::try_from(exception_id).expect("Unsupported exception variant");
    let mut map = SPECIALIZES_EXCEPTION_HANDLER_MAP.lock();

    log::debug!("cought exception (via portal): {:?}", id);

    if let Some(fnc) = map[id.val() as usize] {
        log::debug!("forwarding to specialized exception handler");
        fnc(id, unsafe { &mut EXCEPTION_UTCB })
    } else {
        log::debug!("no specialized exception handler available");

        let mut buf = ArrayString::<32>::new();
        write!(&mut buf, "{:#?}", id).unwrap();
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
