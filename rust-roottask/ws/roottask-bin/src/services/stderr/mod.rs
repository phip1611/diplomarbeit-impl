use core::fmt::Write;
use libhrstd::libhedron::hip::HIP;
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

/// Initializes the service portals for the functionality of this module.
/// Must be called after [`init_writer`].
pub fn init_service() {
    todo!("must implement portals");
}

/// In our use-case, stderr writes to the same final destination as stderr.
///
/// THERE SHOULD NEVER BE MORE THAN A SINGLE INSTANCE OF THIS.
/// [`STDERR_WRITER`] is the only instance allowed!
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
