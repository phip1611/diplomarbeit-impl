#![no_std]
#![no_main]
// allow inline assembly
#![feature(asm)]
// allow globa assembly
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
// TODO include this as soon as a first stable RC is there
#![deny(missing_debug_implementations)]
#![deny(clippy::all)]
#![deny(rustdoc::all)]

global_asm!(include_str!("assembly.S"));

mod panic;
mod rootask_alloc; // any global definitions required to be in assembly

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[no_mangle]
fn roottask_rust_entry(hip_ptr: u64, utsb_ptr: u64) -> ! {
    unsafe {
        asm!(
            "",
            in("rax") 0xdeadbeef_u32,
            in("r8") 0xdeadbeef_u32,
            in("r9") 0xdeadbeef_u32,
            in("r10") 0xdeadbeef_u32,
        );
    }
    loop {}
}
