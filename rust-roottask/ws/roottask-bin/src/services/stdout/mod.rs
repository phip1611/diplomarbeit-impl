use alloc::rc::Rc;
use core::fmt::{
    Debug,
    Write,
};

use libhrstd::cap_space::root::RootCapSpace;
use libhrstd::kobjects::{
    LocalEcObject,
    PtCtx,
    PtObject,
};
use runs_inside_qemu::runs_inside_qemu;

use libhrstd::libhedron::hip::HIP;
use libhrstd::libhedron::mtd::Mtd;
use libhrstd::libhedron::syscall::create_ec::create_local_ec;
use libhrstd::libhedron::syscall::create_pt::create_pt;
use libhrstd::libhedron::syscall::ipc::reply;
use libhrstd::libhedron::utcb::Utcb;
use libhrstd::mem::PageAligned;
use libhrstd::sync::mutex::{
    SimpleMutex,
    SimpleMutexGuard,
};
use libroottask::process_mng::manager::ProcessManager;
use libroottask::process_mng::process::Process;
use libroottask::pt_multiplex::roottask_generic_portal_callback;
use libroottask::stack::StaticStack;

use crate::services::stdout::debugcon::DebugconWriter;
use crate::services::stdout::serial::SerialWriter;

mod debugcon;
mod serial;

/// Global instance of the writer. Protects/synchronizes writers.
static STDOUT_WRITER: SimpleMutex<StdoutWriter> = SimpleMutex::new(StdoutWriter::new());

static mut STDOUT_SERVICE_STACK: StaticStack<4> = StaticStack::new();

/// UTCB for the exception handler portal.
static mut STDOUT_SERVICE_UTCB: PageAligned<Utcb> = PageAligned::new(Utcb::new());

/// Initializes the stdout writer struct. Afterwards [`writer`] can be called.
pub fn init_writer(hip: &HIP) {
    let mut writer = STDOUT_WRITER.lock();
    writer.init(hip);
    // logger not initialized yet
    // log::debug!("stdout available");
}

fn utcb() -> &'static Utcb {
    unsafe { &STDOUT_SERVICE_UTCB }
}

/// Returns a mutable reference to [`StdoutWriter`].
pub fn writer_mut<'a>() -> SimpleMutexGuard<'a, StdoutWriter> {
    STDOUT_WRITER.lock()
}

/// Initializes the service portals for the functionality of this module.
/// Must be called after [`init_writer`].
pub fn init_service(roottask: &Process) {
    let ec = LocalEcObject::create(
        RootCapSpace::RoottaskStdoutServiceLocalEc.val(),
        &roottask.pd_obj(),
        unsafe { STDOUT_SERVICE_STACK.get_stack_top_ptr() } as u64,
        unsafe { STDOUT_SERVICE_UTCB.page_addr() } as u64,
    );
    let pt = PtObject::create(
        RootCapSpace::RoottaskStdoutServicePortal.val(),
        &ec,
        Mtd::empty(),
        roottask_generic_portal_callback,
        None,
    );
    libroottask::pt_multiplex::add_callback_hook(pt.portal_id(), stdout_service_handler);
}

fn stdout_service_handler(
    pt: &Rc<PtObject>,
    process: &Process,
    utcb: &mut Utcb,
    do_reply: &mut bool,
) {
    log::info!("got via IPC: {}", utcb.load_data::<&str>().unwrap());
    *do_reply = true;
}

/// Handles the locations where Stdout-Output goes to.
/// In our case, only Serial- and Debugcon, since we don't have any Display-driver.
///
/// THERE SHOULD NEVER BE MORE THAN A SINGLE INSTANCE OF THIS.
/// [`STDOUT_WRITER`] is the only instance allowed!
#[derive(Debug)]
pub struct StdoutWriter {
    inner: Option<StdoutWriterInner>,
}

impl StdoutWriter {
    const fn new() -> Self {
        Self { inner: None }
    }

    /// Initializes serial and debugcon.
    fn init(&mut self, hip: &HIP) {
        if self.inner.is_some() {
            // note that Rust logger might not be initialized yet
            panic!("already initialized?!");
        }

        let inner = StdoutWriterInner::new(hip);
        self.inner.replace(inner);
    }
}

impl Write for StdoutWriter {
    /// Forwards the write to all available destinations.
    fn write_str(&mut self, msg: &str) -> core::fmt::Result {
        if let Some(ref mut inner) = self.inner {
            inner.serial_writer.write_str(msg)?;
            if let Some(ref mut writer) = inner.debugcon_writer {
                writer.write_str(msg)?;
            }
            Ok(())
        } else {
            // note that Rust logger might not be initialized yet
            panic!("call init_writer() first");
        }
    }
}

#[derive(Debug)]
struct StdoutWriterInner {
    debugcon_writer: Option<DebugconWriter>,
    serial_writer: SerialWriter,
}

impl StdoutWriterInner {
    fn new(hip: &HIP) -> Self {
        let mut debugcon_writer = None;

        if runs_inside_qemu().is_maybe_or_very_likely() {
            let mut writer = DebugconWriter::new();
            writer.init(hip.root_pd());
            writer
                .write_str("+++ STDOUT via DebugconWriter ready +++ \n")
                .unwrap();
            debugcon_writer.replace(writer);
        }

        let mut serial_writer = SerialWriter;
        serial_writer.init(hip.root_pd()).unwrap();
        serial_writer
            .write_str("+++ STDOUT via SerialWriter ready +++ \n")
            .unwrap();

        Self {
            debugcon_writer,
            serial_writer,
        }
    }
}
