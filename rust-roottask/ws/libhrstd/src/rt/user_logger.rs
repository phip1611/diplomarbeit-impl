use crate::rt::services::stdout::stdout_service;
use crate::util::ansi::{
    AnsiStyle,
    Color,
    TextStyle,
};
use arrayvec::ArrayString;
use core::fmt::Write;
use libhedron::mem::PAGE_SIZE;
use log::{
    Level,
    LevelFilter,
    Log,
    Metadata,
    Record,
};

static LOGGER: UserRustLogger = UserRustLogger;

#[derive(Debug)]
pub struct UserRustLogger;

impl UserRustLogger {
    pub fn init() {
        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(LevelFilter::Debug);
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

impl Log for UserRustLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let msg = Self::fmt_msg(record);
        stdout_service(msg.as_str());
    }

    fn flush(&self) {}
}
