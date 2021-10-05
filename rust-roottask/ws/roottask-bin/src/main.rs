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
#![feature(const_ptr_offset)]
#![feature(stmt_expr_attributes)]
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
mod roottask_dispatch;
mod roottask_heap;
mod roottask_stack;

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use libhrstd::hip::HIP;
use libhrstd::utcb::UtcbData;

#[no_mangle]
fn roottask_rust_entry(hip_ptr: u64, utcb_ptr: u64) -> ! {
    let hip = unsafe { (hip_ptr as *const HIP).as_ref().unwrap() };
    let _utcb = unsafe { (utcb_ptr as *const UtcbData).as_ref().unwrap() };

    logger::init(hip.root_pd());
    // unsafe {ROOTTASK_STACK.test_rw_guard_page()};
    // log::info!("guard-page inactive");
    roottask_stack::init(hip);
    // unsafe {ROOTTASK_STACK.test_rw_guard_page()};
    roottask_heap::init();
    exception::init(hip);

    #[rustfmt::skip]
    {
        log::trace!("stack top (inc)   : 0x{:016x?}", roottask_stack::STACK_TOP_PTR.val());
        log::trace!("stack bottom (inc): 0x{:016x}", roottask_stack::STACK_BOTTOM_PTR.val());
        log::trace!("stack size        : {:>18}", roottask_stack::STACK_SIZE_128KIB);

        log::trace!("heap top   (excl) : 0x{:016x}", roottask_heap::HEAP_END_PTR.val());
        log::trace!("heap bottom (inc) : 0x{:016x}", roottask_heap::HEAP_BEGIN_PTR.val());
        log::trace!("heap size         : {:>18}", roottask_heap::HEAP_SIZE);

        log::trace!("utcb ptr          : 0x{:016x}", utcb_ptr);
        log::trace!("hip ptr           : 0x{:016x}", hip_ptr);
    }

    log::debug!("===========================================================");
    let mem_descs = hip.mem_desc_iterator().collect::<Vec<_>>();
    log::debug!("Mem_descs: #{}", mem_descs.len());
    log::debug!("{:#?}", mem_descs);

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
