#![no_std]
#![no_main]
// allow inline assembly
#![feature(asm)]
// allow global assembly
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(const_mut_refs)]
#![deny(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    // clippy::restriction,
    // clippy::pedantic
)]
// now allow a few rules which are denied by the above statement
// --> they are ridiculous and not necessary
#![allow(
    clippy::suboptimal_flops,
    clippy::redundant_pub_crate,
    clippy::fallible_impl_from
)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::all)]

// any global definitions required to be in assembly
global_asm!(include_str!("assembly.S"));

mod exception;
mod logger;
mod panic;
mod roottask_alloc;
mod roottask_dispatch;
mod stack;

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

use core::ptr;
use libhrstd::hip::HIP;
use libhrstd::utcb::UtcbData;
use stack::ROOTTASK_STACK_TOP_PTR;

#[no_mangle]
fn roottask_rust_entry(hip_ptr: u64, utcb_ptr: u64) -> ! {
    let hip = &unsafe { ptr::read(hip_ptr as *const HIP) };
    let _utcb = &unsafe { ptr::read(utcb_ptr as *const UtcbData) };

    logger::init(hip.root_pd());
    // unsafe {ROOTTASK_STACK.test_rw_guard_page()};
    // log::info!("guard-page inactive");
    stack::init(hip);
    // unsafe {ROOTTASK_STACK.test_rw_guard_page()};
    roottask_alloc::init();
    exception::init(hip);

    log::info!("stack_ptr: 0x{:x}", ROOTTASK_STACK_TOP_PTR.val());

    /*// TODO kriege ein divided by zero error sobald ich softfloat deaktiviere
    let x = 5.1212 * 1414.2;
    log::debug!("{}", x);*/

    /* test: floating point + SSE registers work
    let x = 2.0;
    let y = core::f32::consts::PI;
    let _z = x * y;
    */

    /* test: trigger devided by zero exception
    {
        unsafe { asm!("mov rax, 5", "mov rdi, 0", "div rax, rdi") }
    }*/

    // test: trigger GPF
    /*{
        unsafe {
            x86::io::outb(0x0, 0);
        }
    }*/

    log::info!("Rust Roottask started");
    panic!("SHHIIIIIT");
    loop {}
}
