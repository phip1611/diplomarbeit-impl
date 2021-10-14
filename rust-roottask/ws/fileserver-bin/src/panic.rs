use core::panic::PanicInfo;

#[panic_handler]
pub fn handle_panic(info: &PanicInfo) -> ! {
    libhrstd::rt::rust_rt::panic::handle_panic(info);
}
