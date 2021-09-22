use arrayvec::ArrayString;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::sync::atomic::{
    compiler_fence,
    Ordering,
};

/// Formats a nice panic message.
fn generate_panic_msg(info: &PanicInfo) -> ArrayString<1024> {
    let mut buf = arrayvec::ArrayString::<1024>::new();
    // if this is an error, we ignore it. It would most probably mean, that the message
    // was only created partly.
    let _ = writeln!(
        &mut buf,
        "PANIC in {}@{}:{}: {:#?}",
        info.location()
            .map(|l| l.file())
            .unwrap_or("<Unknown File>"),
        info.location().map(|l| l.line()).unwrap_or(0),
        info.location().map(|l| l.column()).unwrap_or(0),
        info.message().unwrap_or(&format_args!("")),
        // info.payload(),
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
