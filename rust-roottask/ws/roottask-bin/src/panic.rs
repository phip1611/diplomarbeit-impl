use arrayvec::ArrayString;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::sync::atomic::{
    compiler_fence,
    Ordering,
};

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

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("{}", generate_panic_msg(info));
    loop {
        compiler_fence(Ordering::SeqCst);
    }
}
