use core::panic::PanicInfo;
use core::sync::atomic::{
    compiler_fence,
    Ordering,
};

#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    loop {
        compiler_fence(Ordering::SeqCst)
    }
}
