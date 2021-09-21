use crate::logger::debugcon::DebugconLogger;
use crate::logger::serial::SerialLogger;
use arrayvec::ArrayString;
use core::fmt::Write;
use log::{
    LevelFilter,
    Log,
    Metadata,
    Record,
};
use roottask_lib::hedron::capability::CapSel;
use roottask_lib::hrstd::sync::mutex::SimpleMutex;

mod debugcon;
mod serial;

static LOGGER: GenericLogger = GenericLogger::new();

/// Generic logger for the roottask which decides where things
/// should be logged to. Can use multiple/different loggers internally.
///
/// Synchronizes logging, therefore it can be used in a local EC to provide
/// a logging service for other components.
#[derive(Debug)]
struct GenericLogger {
    debugcon: SimpleMutex<Option<DebugconLogger>>,
    serial: SimpleMutex<Option<SerialLogger>>,
}

impl GenericLogger {
    const fn new() -> Self {
        GenericLogger {
            debugcon: SimpleMutex::new(None),
            serial: SimpleMutex::new(None),
        }
    }

    /// Builds the formatted error message in a stack-allocated array.
    /// Because we don't have nested logging, this is fine and cheap.
    ///
    /// Make sure that stack in `assembly.S` is big enough.
    fn fmt_msg(record: &Record) -> ArrayString<4096> {
        let mut buf = ArrayString::<4096>::new();

        let res = writeln!(
            &mut buf,
            "[{:>5}] {:>15}@{}: {}",
            record.level(),
            record
                .file()
                // remove full system path, only keep file path in project
                .map(|f| {
                    let index = f.find("/src");
                    if let Some(index) = index {
                        // skip slash
                        let index = index + 1;
                        &f[index..]
                    } else {
                        f
                    }
                })
                .unwrap_or("<unknown file>"),
            record.line().unwrap_or(0),
            record.args()
        );

        // TODO I think that we don't even come to this and some kernel
        //  error happens for too small bufs..dafuq?!
        // TODO fallback to allocated buffer?
        if res.is_err() {
            // this will work most probably, except
            // if we are out of stack memory for a recursive call
            log::warn!("fmt msg failed; not enough memory");
        }

        buf
    }

    /// Activates the logger backend.
    pub fn update_debugcon(&self, debugcon: DebugconLogger) {
        let mut lock = self.debugcon.lock();
        lock.replace(debugcon);
    }

    /// Activates the logger backend.
    pub fn update_serial(&self, serial: SerialLogger) {
        let mut lock = self.serial.lock();
        lock.replace(serial);
    }
}

impl Log for GenericLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // log everything
        true
    }

    fn log(&self, record: &Record) {
        let msg = Self::fmt_msg(record);

        if let Some(l) = self.debugcon.lock().as_ref() {
            l.write(msg.as_str()).unwrap();
        }
        if let Some(l) = self.serial.lock().as_ref() {
            l.write(msg.as_str()).unwrap();
        }
    }

    fn flush(&self) {
        // no buffering mechanism => no flushing
    }
}

/// Initializes the root task logger(s).
/// Needs the roottasks protection domain selector.
pub fn init(root_pd_sel: CapSel) {
    log::set_max_level(LevelFilter::max());
    log::set_logger(&LOGGER).unwrap();

    DebugconLogger::init(root_pd_sel);
    LOGGER.update_debugcon(DebugconLogger);
    log::debug!("Debugcon-Logger is available");

    let res = SerialLogger::init(root_pd_sel);
    if res.is_ok() {
        // only if serial device is available
        LOGGER.update_serial(SerialLogger);
        log::debug!("Serial-Logger is available");
    } else {
        log::debug!("Serial-Logger is not available");
    }
}
