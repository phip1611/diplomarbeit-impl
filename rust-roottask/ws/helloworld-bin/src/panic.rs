use core::panic::PanicInfo;

#[panic_handler]
pub fn handle_panic(info: &PanicInfo) -> ! {
    libhrstd::rt::rust_rt::user_panic_handler::handle_panic(info);
}
