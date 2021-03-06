//! See [`generate_panic_msg`].

use crate::util::ansi::{
    AnsiStyle,
    Color,
    TextStyle,
};
use arrayvec::ArrayString;
use core::fmt::Write;
use core::panic::PanicInfo;

// Formats a nice panic message (for roottask and user apps)
pub fn generate_panic_msg<const STATIC_BUF_SIZE: usize>(
    info: &PanicInfo,
) -> ArrayString<STATIC_BUF_SIZE> {
    let mut buf = arrayvec::ArrayString::new();
    // if this is an error, we ignore it. It would most probably mean, that the message
    // was only created partly, i.e. the buffer was too small.

    // formats "src/exception.rs"
    let mut file_fmt = ArrayString::<128>::new();
    let _ = write!(
        &mut file_fmt,
        "{}",
        info.location()
            .map(|l| l.file())
            .unwrap_or("<Unknown File>")
    );

    // formats "@14:4:"
    let mut file_location_fmt = ArrayString::<16>::new();
    let _ = write!(
        &mut file_location_fmt,
        "@{}:{}:",
        info.location().map(|l| l.line()).unwrap_or(0),
        info.location().map(|l| l.column()).unwrap_or(0)
    );

    // format the panic message
    let res = writeln!(
        &mut buf,
        "{panic} in {filename}{filelocation} {msg:#?}",
        panic = AnsiStyle::new()
            .msg("PANIC")
            .foreground_color(Color::Red)
            .text_style(TextStyle::Bold),
        filename = AnsiStyle::new()
            .msg(file_fmt.as_str())
            .foreground_color(Color::Blue),
        filelocation = AnsiStyle::new()
            .msg(file_location_fmt.as_str())
            .text_style(TextStyle::Dimmed),
        msg = info.message().unwrap_or(&format_args!("<none>")),
    );

    if res.is_err() {
        let msg_too_long = "<PANIC MSG TOO LONG; TRUNCATED>\n";
        unsafe { buf.set_len(buf.len() - msg_too_long.len()) };
        let _ = buf.write_str(msg_too_long);
    }

    buf
}
