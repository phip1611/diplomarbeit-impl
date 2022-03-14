use crate::process::Process;
use crate::pt_multiplex::roottask_generic_portal_callback;
use alloc::rc::Rc;
use core::fmt::Write;
use libhrstd::kobjects::{
    LocalEcObject,
    PtCtx,
    PtObject,
};
use libhrstd::libhedron::CapSel;
use libhrstd::libhedron::Mtd;
use libhrstd::libhedron::Utcb;
use libhrstd::libhedron::HIP;
use libhrstd::service_ids::ServiceId;
use libhrstd::sync::mutex::{
    SimpleMutex,
    SimpleMutexGuard,
};

/// Global instance of the writer. Protects/synchronizes writers.
static STDERR_WRITER: SimpleMutex<StderrWriter> = SimpleMutex::new(StderrWriter::new());

/// Initializes the stderr writer struct. Afterwards [`writer`] can be called.
pub fn init_writer(_hip: &HIP) {
    let mut lock = STDERR_WRITER.lock();
    lock.init();
    // logger not initialized yet
    // log::debug!("stderr available");
}

/// Returns a mutable reference to [`StderrWriter`].
pub fn writer_mut<'a>() -> SimpleMutexGuard<'a, StderrWriter> {
    STDERR_WRITER.lock()
}

/// Creates a new STDERR service PT, which can be delegated to a new process.
pub fn create_service_pt(base_cap_sel: CapSel, ec: &Rc<LocalEcObject>) -> Rc<PtObject> {
    let service = ServiceId::StderrService;
    // adds itself to the local EC
    PtObject::create(
        base_cap_sel + service.val(),
        &ec,
        Mtd::empty(),
        roottask_generic_portal_callback,
        PtCtx::Service(service),
    )
}

/// Handles the functionality of the STDERR Portal.
pub fn stderr_service_handler(
    _pt: &Rc<PtObject>,
    process: &Process,
    utcb: &mut Utcb,
    do_reply: &mut bool,
) {
    // currently STDERR maps to STDOUT
    let msg = utcb.load_data::<&str>().unwrap();
    {
        let mut writer = STDERR_WRITER.lock();
        let res = write!(&mut writer, "[STDERR PID={}] {}\n", process.pid(), msg,);
        // drop before unwrap, because otherwise deadlock happens on panic
        // (panic needs lock to STDOUT_WRITER)
        core::mem::drop(writer);
        res.unwrap();
    }
    *do_reply = true;
}

/// In our use-case, stderr writes to the same final destination as stderr.
///
/// THERE SHOULD NEVER BE MORE THAN A SINGLE INSTANCE OF THIS.
/// [`STDERR_WRITER`] is the only instance allowed!
#[derive(Debug)]
pub struct StderrWriter {
    init: bool,
}

impl StderrWriter {
    const fn new() -> Self {
        Self { init: false }
    }

    pub fn init(&mut self) {
        if self.init {
            // note that Rust logger might not be initialized yet
            panic!("called init for stderr twice?!");
        }
        self.init = true;
    }
}

impl Write for StderrWriter {
    /// Forwards stderr to stdout.
    fn write_str(&mut self, msg: &str) -> core::fmt::Result {
        if !self.init {
            // note that Rust logger might not be initialized yet
            panic!("not initialized");
        }
        super::stdout::writer_mut().write_str(msg)
    }
}
