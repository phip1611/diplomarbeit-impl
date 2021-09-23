use crate::logger::debugcon::DebugconLogger;
use crate::logger::serial::SerialLogger;
use arrayvec::ArrayString;
use core::fmt::Write;
use log::{
    Level,
    LevelFilter,
    Log,
    Metadata,
    Record,
};
use roottask_lib::hedron::capability::CapSel;
use roottask_lib::hrstd::sync::mutex::SimpleMutex;
use roottask_lib::util::ansi::{
    AnsiStyle,
    Color,
    TextStyle,
};

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

        // "TRACE", " INFO", "ERROR"...
        let mut level = ArrayString::<5>::new();
        write!(&mut level, "{:>5}", record.level().as_str()).unwrap();

        // file name: origin of logging msg
        let file = record
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
            .unwrap_or("<unknown file>");
        let file_style = AnsiStyle::new().msg(file).text_style(TextStyle::Dimmed);

        let mut line = ArrayString::<5>::new();
        write!(&mut line, "{}", record.line().unwrap_or(0)).unwrap();
        let line_style = AnsiStyle::new()
            .msg(line.as_str())
            .text_style(TextStyle::Dimmed);

        let res = writeln!(
            &mut buf,
            "[{level:>5}] {file:>15}{at_sign}{line}{double_point} {msg}",
            // level is padded to 5 chars and right-aligned
            // style around
            level = Self::style_for_level(record.level()).msg(level.as_str()),
            file = file_style,
            at_sign = AnsiStyle::new().text_style(TextStyle::Dimmed).msg("@"),
            line = line_style,
            double_point = AnsiStyle::new().text_style(TextStyle::Bold).msg(":"),
            msg = record.args(),
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

    /// Gets the style for "DEBUG", "ERROR" etc.
    fn style_for_level<'a>(level: Level) -> AnsiStyle<'a> {
        match level {
            Level::Error => AnsiStyle::new()
                .text_style(TextStyle::Bold)
                .foreground_color(Color::Red),
            Level::Warn => AnsiStyle::new()
                .text_style(TextStyle::Bold)
                .foreground_color(Color::Yellow),
            Level::Info => AnsiStyle::new().foreground_color(Color::Green),
            Level::Debug => AnsiStyle::new().foreground_color(Color::Yellow),
            Level::Trace => AnsiStyle::new().foreground_color(Color::Green),
        }
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
