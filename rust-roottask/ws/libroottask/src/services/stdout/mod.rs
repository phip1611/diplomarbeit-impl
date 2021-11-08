use crate::process_mng::process::Process;
use crate::pt_multiplex::roottask_generic_portal_callback;
use crate::services::stdout::debugcon::DebugconWriter;
use crate::services::stdout::serial::SerialWriter;
use crate::stack::StaticStack;
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
use libhrstd::libhedron::capability::CapSel;
use libhrstd::libhedron::hip::HIP;
use libhrstd::libhedron::mtd::Mtd;
use libhrstd::libhedron::utcb::Utcb;
use libhrstd::mem::PageAligned;
use libhrstd::service_ids::ServiceId;
use libhrstd::sync::mutex::{
    SimpleMutex,
    SimpleMutexGuard,
};
use runs_inside_qemu::runs_inside_qemu;

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

/// Creates a new STDOUT service PT, which can be delegated to a new process.
pub fn create_service_pt(base_cap_sel: CapSel, ec: &Rc<LocalEcObject>) -> Rc<PtObject> {
    let service = ServiceId::StdoutService;
    // adds itself to the local EC
    PtObject::create(
        base_cap_sel + service.val(),
        &ec,
        Mtd::empty(),
        roottask_generic_portal_callback,
        PtCtx::Service(service),
    )
}

/// Handles the functionality of the STDOUT Portal.
pub fn stdout_service_handler(
    _pt: &Rc<PtObject>,
    process: &Process,
    utcb: &mut Utcb,
    do_reply: &mut bool,
) {
    log::debug!("WAH");
    log::debug!("WAH");
    let msg = utcb.load_data::<&str>().unwrap();
    log::debug!("WAH");
    log::info!("STDOUT service called by PID: {}", process.pid());
    log::debug!("WAH");
    {
        let mut writer = STDOUT_WRITER.lock();
        let res = write!(
            &mut writer,
            "[STDOUT PID={}] {}",
            process.pid(),
            utcb.load_data::<&str>().unwrap()
        );
        core::mem::drop(writer);
        res.unwrap();
    }
    log::debug!("WAH");
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
