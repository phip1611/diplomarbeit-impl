//! Code related to exception handling. All exception handling is done inside the
//! root task.

use crate::roottask_dispatch::ROOT_EXC_EVENT_BASE;
use arrayvec::ArrayString;
use core::convert::TryFrom;
use core::fmt::Write;
use roottask_lib::hedron::capability::CapSel;
use roottask_lib::hedron::event_offset::ExceptionEventOffset;
use roottask_lib::hedron::hip::HIP;
use roottask_lib::hedron::mtd::Mtd;
use roottask_lib::stack::StaticStack;
use roottask_lib::syscall::create_ec::{
    create_ec,
    EcKind,
};
use roottask_lib::syscall::create_pt::create_pt;
use roottask_lib::syscall::pt_ctrl::pt_ctrl;
use roottask_lib::util::ansi::{
    AnsiStyle,
    Color,
    TextStyle,
};

/// Used as stack for the exception handler callback function. Must be either mutable
/// or manually placed in a writeable section in the file. Otherwise we get a page fault.
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
static mut CALLBACK_STACK: StaticStack<2> = StaticStack::new();

/// Initializes a local EC and N portals to cover N exceptions.
/// All exceptions are considered as unrecoverable in this roottask.
/// Therefore, they panic. See [`roottask_lib::hedron::event_offset::ExceptionEventOffset`]
/// to see possible exceptions.
///
/// If it fails, the program aborts.
pub fn init(hip: &HIP) {
    // todo make dynamic cap sel
    let ec_cap_sel = 64;

    create_ec(
        EcKind::Local,
        ec_cap_sel,
        hip.root_pd(),
        unsafe { CALLBACK_STACK.get_stack_top_ptr() } as u64,
        ROOT_EXC_EVENT_BASE,
        0,
        0,
        false,
        false,
    )
    .unwrap();

    log::info!("created local ec for exception handling");

    // I iterate here over all available/reserved capability selectors for exceptionss.
    // This is relative to the event base selector. For the roottask/root protection domain,
    // it is 0 (See ROOT_EXC_EVENT_BASE).
    // We install an actual kernel object of type portal at the given indices.

    let from = ROOT_EXC_EVENT_BASE;
    let to = hip.num_exc_sel() as u64;
    // iterate from 0 to 32 (exception capability selector space)
    for excp_offset in from..to {
        // equivalent to the enum variants from ExceptionEventOffset
        let portal_cap_sel = ROOT_EXC_EVENT_BASE + excp_offset as CapSel;

        // create portal for each exception
        create_pt(
            portal_cap_sel,
            hip.root_pd(),
            ec_cap_sel,
            general_exception_handler as *const u64,
            Mtd::DEFAULT,
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
        {
            let mut msg = ArrayString::<128>::from("created PT for exception=").unwrap();
            let exc = ExceptionEventOffset::try_from(excp_offset as u64);
            if let Ok(exc) = exc {
                write!(&mut msg, "{:?}({})", exc, excp_offset).unwrap();
            } else {
                write!(&mut msg, "Unknown({:?})", excp_offset).unwrap();
            }
            log::info!("{}", msg);
        }
    }
}

/// General exception handler for all x86 exceptions that can happen.
/// The handler aborts the program if an exception occurs. Panics the Rust program.
fn general_exception_handler(id: u64) -> ! {
    let id = ExceptionEventOffset::try_from(id).unwrap();
    let mut buf = ArrayString::<32>::new();
    write!(&mut buf, "{:#?}", id).unwrap();
    panic!(
        "Mayday, caught exception id={} - aborting program",
        AnsiStyle::new()
            .foreground_color(Color::Red)
            .text_style(TextStyle::Bold)
            .msg(buf.as_str())
    );

    // entweder return mit reply syscall
    // oder panic => game over

    // Problem bei normalem return: keine rücksprugnadresse
}
