use crate::PAGE_SIZE;
use core::arch::asm;
use core::panic::PanicInfo;
use core::sync::atomic::{
    compiler_fence,
    Ordering,
};
use libhrstd::util::panic_msg::generate_panic_msg;

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

    log::error!("{}", generate_panic_msg::<PAGE_SIZE>(info));

    loop {
        compiler_fence(Ordering::SeqCst);
    }
}
