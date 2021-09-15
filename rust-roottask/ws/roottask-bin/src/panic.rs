use core::panic::PanicInfo;
use core::sync::atomic::{Ordering, compiler_fence};

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    loop {
        compiler_fence(Ordering::SeqCst);
    }
}
