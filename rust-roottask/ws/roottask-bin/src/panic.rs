use arrayvec::ArrayString;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::sync::atomic::{
    compiler_fence,
    Ordering,
};
use roottask_lib::util::ansi::{
    AnsiStyle,
    Color,
    TextStyle,
};

/// Formats a nice panic message.
fn generate_panic_msg(info: &PanicInfo) -> ArrayString<1024> {
    let mut buf = arrayvec::ArrayString::<1024>::new();
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
    let _ = writeln!(
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
    buf
}

/// Writes 0x2EEDCOFFEE into r8 to r15, writes a nice panic message to the logger,
/// and aborts the program in an endless loop.
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    unsafe {
        asm!(
            "",
            in("r8") 0x2EEDC0FFEE_u64,
            in("r9") 0x2EEDC0FFEE_u64,
            in("r10") 0x2EEDC0FFEE_u64,
            in("r11") 0x2EEDC0FFEE_u64,
            in("r12") 0x2EEDC0FFEE_u64,
            in("r13") 0x2EEDC0FFEE_u64,
            in("r14") 0x2EEDC0FFEE_u64,
            in("r15") 0x2EEDC0FFEE_u64,
        )
    }

    log::error!("{}", generate_panic_msg(info));

    loop {
        compiler_fence(Ordering::SeqCst);
    }
}
