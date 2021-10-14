use core::fmt::{
    Debug,
    Write,
};

use runs_inside_qemu::runs_inside_qemu;

use libhrstd::libhedron::hip::HIP;
use libhrstd::sync::mutex::{
    SimpleMutex,
    SimpleMutexGuard,
};
use libroottask::capability_space::RootCapabilitySpace;

use crate::services::stdout::debugcon::DebugconWriter;
use crate::services::stdout::serial::SerialWriter;

mod debugcon;
mod serial;

/// Global instance of the writer. Protects/synchronizes writers.
static STDOUT_WRITER: SimpleMutex<StdoutWriter> = SimpleMutex::new(StdoutWriter::new());

/// Initializes the stdout writer struct. Afterwards [`writer`] can be called.
pub fn init_writer(hip: &HIP) {
    let mut writer = STDOUT_WRITER.lock();
    writer.init(hip);
    // logger not initialized yet
    // log::debug!("stdout available");
}

/// Returns a mutable reference to [`StdoutWriter`].
pub fn writer_mut<'a>() -> SimpleMutexGuard<'a, StdoutWriter> {
    STDOUT_WRITER.lock()
}

/// Initializes the service portals for the functionality of this module.
/// Must be called after [`init_writer`].
pub fn init_service() {
    todo!("must implement portals");
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