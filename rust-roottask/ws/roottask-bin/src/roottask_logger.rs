//! Module to initialize typical Rust logging for the Roottask itself.

use arrayvec::ArrayString;
use core::fmt::Write;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::sync::mutex::SimpleMutex;
use libhrstd::util::ansi::{
    AnsiStyle,
    Color,
    TextStyle,
};
use log::{
    Level,
    LevelFilter,
    Log,
    Metadata,
    Record,
};

/// Logger instance that gets passed to the [`log`]-crate.
/// Synchronizes all logs.
static LOGGER: GenericLogger = GenericLogger::new();

/// Generic logger for the roottask which decides where things
/// should be logged to. Can use multiple/different loggers internally.
///
/// Synchronizes logging, therefore it can be used in a local EC to provide
/// a logging service for other components.
#[derive(Debug)]
struct GenericLogger {
    // Advisory lock for logging.
    //
    // I'm not 100% sure if I need synchronization at this point, but because other threads
    // (global ECs) can invoke portals, which may log, it's better to synchronize at the
    // the logger level too and not just at the level of the serial writer!
    lock: SimpleMutex<()>,
}

impl GenericLogger {
    /// Creates a new [`GenericLogger`].
    const fn new() -> Self {
        Self {
            lock: SimpleMutex::new(()),
        }
    }

    /// Builds the formatted error message in a stack-allocated array.
    /// Because we don't have nested logging, this is fine and cheap.
    ///
    /// Make sure that stack of roottask is big enough.
    fn fmt_msg(record: &Record) -> ArrayString<PAGE_SIZE> {
        let mut buf = ArrayString::new();

        // "TRACE", " INFO", "ERROR"...
        let mut level = ArrayString::<5>::new();
        write!(&mut level, "{:>5}", record.level().as_str()).unwrap();

        let crate_name = record
            .module_path()
            .map(|module| module.split_once("::").map(|x| x.0).unwrap_or(module))
            .unwrap_or("<unknown mod>");

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

        let mut line = ArrayString::<5>::new();
        write!(&mut line, "{}", record.line().unwrap_or(0)).unwrap();

        let res = writeln!(
            &mut buf,
            "[{level:>5}] {crate_name}:{file:>15}{at_sign}{line}{double_point} {msg}",
            // level is padded to 5 chars and right-aligned
            // style around
            level = Self::style_for_level(record.level()).msg(level.as_str()),
            crate_name = AnsiStyle::new()
                .foreground_color(Color::Magenta)
                .msg(crate_name),
            file = AnsiStyle::new().msg(file).text_style(TextStyle::Dimmed),
            at_sign = AnsiStyle::new().text_style(TextStyle::Dimmed).msg("@"),
            line = AnsiStyle::new()
                .msg(line.as_str())
                .text_style(TextStyle::Dimmed),
            double_point = AnsiStyle::new().text_style(TextStyle::Bold).msg(":"),
            msg = record.args(),
        );

        if res.is_err() {
            let msg_too_long = "<LOG MSG TOO LONG; TRUNCATED>\n";
            unsafe { buf.set_len(buf.len() - msg_too_long.len()) };
            let _ = buf.write_str(msg_too_long);
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
}

impl Log for GenericLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // log everything
        true
    }

    fn log(&self, record: &Record) {
        // this is synchronized, because this may be invoked by multiple portals
        // (which are called from other PDs/global ECs).
        self.lock.lock().execute_while_locked(|| {
            let msg = Self::fmt_msg(record);
            crate::services::stderr::writer_mut()
                .write_str(msg.as_str())
                .unwrap();
        });
    }

    fn flush(&self) {
        // no buffering mechanism => no flushing
    }
}

/// Initializes the Rust logger for the root task. Forwards to the default STDERR location.
pub fn init() {
    // log::set_max_level(LevelFilter::max());
    log::set_max_level(LevelFilter::Info);
    log::set_logger(&LOGGER).expect("call this only once!");

    // Q&D: execute this once, so catch the logging-messages, which gives us nice
    //  info about the environment (hypervisor or not, ...)
    let _ = runs_inside_qemu::runs_inside_qemu();
}
