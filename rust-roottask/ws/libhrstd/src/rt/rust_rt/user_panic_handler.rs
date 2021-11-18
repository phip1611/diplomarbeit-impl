use crate::libhedron::mem::PAGE_SIZE;
use crate::util::panic_msg::generate_panic_msg;
use core::panic::PanicInfo;
use core::sync::atomic::{
    compiler_fence,
    Ordering,
};

pub fn handle_panic(info: &PanicInfo) -> ! {
    log::error!("{}", generate_panic_msg::<PAGE_SIZE>(info));
    loop {
        compiler_fence(Ordering::SeqCst)
    }
}
