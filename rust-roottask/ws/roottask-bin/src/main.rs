#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(const_mut_refs)]
#![feature(const_ptr_offset)]
#![feature(stmt_expr_attributes)]
#![feature(const_btree_new)]
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

use core::arch::global_asm;

// any global definitions required to be in assembly
global_asm!(include_str!("assembly.S"));

mod panic;
mod roottask_heap;
mod roottask_logger;
mod roottask_stack;

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[allow(unused)]
#[macro_use]
extern crate libhrstd;

use crate::roottask_stack::{
    STACK_SIZE,
    STACK_TOP_PTR,
};
use libhrstd::libhedron::hip::HIP;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::utcb::Utcb;
use libroottask::process_mng::manager;
use libroottask::rt::userland;
use libroottask::static_alloc::GlobalStaticChunkAllocator;
use libroottask::{
    roottask_exception,
    services,
};

#[no_mangle]
fn roottask_rust_entry(hip_addr: u64, utcb_addr: u64) -> ! {
    let hip = unsafe { (hip_addr as *const HIP).as_ref().unwrap() };
    let _utcb = unsafe { (utcb_addr as *mut Utcb).as_mut().unwrap() };

    services::init_writers(hip);
    roottask_logger::init();

    // unsafe {ROOTTASK_STACK.test_rw_guard_page()};
    // log::info!("guard-page inactive");
    roottask_stack::init(hip);
    // unsafe {ROOTTASK_STACK.test_rw_guard_page()};
    roottask_heap::init();

    #[rustfmt::skip]
    {
        log::trace!("stack top    (incl): 0x{:016x?}", roottask_stack::STACK_TOP_PTR.val());
        log::trace!("stack bottom (incl): 0x{:016x}", roottask_stack::STACK_BOTTOM_PTR.val());
        log::trace!("stack size         : {:>18}", roottask_stack::STACK_SIZE);
        log::trace!("stack size (pages) : {:>18}", roottask_stack::STACK_SIZE / PAGE_SIZE);

        log::trace!("heap top    (excl) : 0x{:016x}", roottask_heap::HEAP_END_PTR.val());
        log::trace!("heap bottom (incl) : 0x{:016x}", roottask_heap::HEAP_BEGIN_PTR.val());
        log::trace!("heap size          : {:>18}", roottask_heap::HEAP_SIZE);
        log::trace!("heap size (pages)  : {:>18}", roottask_heap::HEAP_SIZE / PAGE_SIZE);
        log::trace!("heap size (chunks) : {:>18}", roottask_heap::HEAP_SIZE / GlobalStaticChunkAllocator::CHUNK_SIZE);

        log::trace!("utcb ptr           : 0x{:016x}", utcb_addr);
        log::trace!("hip ptr            : 0x{:016x}", hip_addr);
        log::debug!("===========================================================");
    }

    manager::PROCESS_MNG.lock().init(
        hip_addr,
        utcb_addr,
        (STACK_SIZE / PAGE_SIZE) as u64,
        STACK_TOP_PTR.val(),
    );
    roottask_exception::init(manager::PROCESS_MNG.lock().root());
    manager::PROCESS_MNG.lock().register_startup_exc_callback();

    services::init_services(manager::PROCESS_MNG.lock().root());

    // NOW READY TO START PROCESSES

    /* TEST IPC
    let msg = "hallo welt 123 fooa\n";
    utcb.store_data(&msg).unwrap();
    call(RootCapSpace::RoottaskStdoutServicePortal.val()).unwrap();
    log::info!("done");*/

    log::debug!(
        "{:#?}",
        hip.mem_desc_iterator().collect::<alloc::vec::Vec<_>>()
    );

    let userland = userland::Userland::load(hip);
    userland.bootstrap();

    /* test: floating point + SSE registers work
    let x = 2.0;
    let y = core::f32::consts::PI;
    let _z = x * y;
    */

    /* test: trigger devided by zero exception */
    /*{
        unsafe { asm!("mov rax, 5", "mov rdi, 0", "div rax, rdi") };
    }*/

    // test: trigger GPF
    /*{
        unsafe {
            x86::io::outb(0x0, 0);
        }
    }*/

    log::info!("Rust Roottask started");

    // panic!("panic works");
    loop {}
}
