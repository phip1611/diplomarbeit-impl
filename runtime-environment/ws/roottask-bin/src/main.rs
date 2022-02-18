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
use core::arch::global_asm;
use libhrstd::cap_space::root::RootCapSpace;
use libhrstd::kobjects::SmObject;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::HIP;
use libhrstd::libhedron::{
    Mtd,
    Utcb,
};
use libhrstd::time::Instant;
use libroottask::process_mng::manager;
use libroottask::rt::userland;
use libroottask::services::init_roottask_echo_pts;
use libroottask::static_alloc::GlobalStaticChunkAllocator;
use libroottask::{
    roottask_exception,
    services,
};

#[no_mangle]
fn roottask_rust_entry(hip_addr: u64, utcb_addr: u64) -> ! {
    let hip = unsafe { (hip_addr as *const HIP).as_ref().unwrap() };
    let utcb = unsafe { (utcb_addr as *mut Utcb).as_mut().unwrap() };

    services::init_writers(hip);
    roottask_logger::init();

    // unsafe {ROOTTASK_STACK.test_rw_guard_page()};
    // log::info!("guard-page inactive");
    roottask_stack::init(hip);
    // unsafe {ROOTTASK_STACK.test_rw_guard_page()};
    roottask_heap::init();

    #[rustfmt::skip]
    {
        log::debug!("stack top    (incl): 0x{:016x}", roottask_stack::STACK_TOP_PTR.val());
        log::debug!("stack bottom (incl): 0x{:016x}", roottask_stack::STACK_BOTTOM_PTR.val());
        log::debug!("stack size         : {:>18}", roottask_stack::STACK_SIZE);
        log::debug!("stack size (pages) : {:>18}", roottask_stack::STACK_SIZE / PAGE_SIZE);

        log::debug!("heap top    (excl) : 0x{:016x}", roottask_heap::HEAP_END_PTR.val());
        log::debug!("heap bottom (incl) : 0x{:016x}", roottask_heap::HEAP_BEGIN_PTR.val());
        log::debug!("heap size          : {:>18}", roottask_heap::HEAP_SIZE);
        log::debug!("heap size (pages)  : {:>18}", roottask_heap::HEAP_SIZE / PAGE_SIZE);
        log::debug!("heap size (chunks) : {:>18}", roottask_heap::HEAP_SIZE / GlobalStaticChunkAllocator::CHUNK_SIZE);

        log::debug!("utcb ptr           : 0x{:016x}", utcb_addr);
        log::debug!("hip ptr            : 0x{:016x}", hip_addr);
        log::debug!("hip: serial port   : 0x{:04x}", hip.serial_port());
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

    let root_pd = manager::PROCESS_MNG.lock().root().clone();
    let root_sm = SmObject::create(RootCapSpace::RootSmSleep.val(), &root_pd.pd_obj());

    services::init_services(manager::PROCESS_MNG.lock().root());

    log::info!("Rust Roottask started successfully");
    do_bench(utcb);

    // NOW READY TO START PROCESSES
    let userland = userland::InitialUserland::load(hip);
    userland.bootstrap();
    log::info!("Userland bootstrapped");

    /* test: floating point + SSE registers work
    let x = 2.0;
    let y = core::f32::consts::PI;
    let _z = x * y;
    */

    /* test: trigger divided by zero exception */
    /*{
        unsafe { asm!("mov rax, 5", "mov rdi, 0", "div rax, rdi") };
    }*/

    // test: trigger GPF
    /*{
        unsafe {
            x86::io::outb(0x0, 0);
        }
    }*/

    // Puts the main thread to sleep nicely; there is no need for a busy loop
    root_sm.sem_down();
    unreachable!();
}

/// Performs several PD-internal IPC benchmarks and measures native system call
/// performance from a Native Hedron App (i.e. the roottask).
fn do_bench(utcb: &mut Utcb) {
    log::info!("benchmarking starts");
    let (echo_pt, raw_echo_pt) = init_roottask_echo_pts();
    const ITERATIONS: u64 = 100_000;
    // ############################################################################
    // MEASURE NATIVE SYSTEM CALL PERFORMANCE
    let begin = Instant::now();
    for i in 0..ITERATIONS {
        unsafe {
            raw_echo_pt.ctrl(i).unwrap();
        }
    }
    let dur_pt_ctrl = Instant::now() - begin;
    // ############################################################################
    // MEASURE ECHO SYSCALL PERFORMANCE (PD-internal IPC with my PT multiplexing mechanism)
    let begin = Instant::now();
    for _ in 0..ITERATIONS {
        echo_pt.call().unwrap();
    }
    let dur_echo = Instant::now() - begin;
    // ############################################################################
    // MEASURE RAW EXCEPTION IPC PERFORMANCE (pure PD-internal IPC)
    // To influence this value you may change the Mtd assigned to the echo raw PT
    // where it gets created. For example, if FPU is present or not will influence the
    // performance.

    //utcb.enable_store_ipc_exc();
    let begin = Instant::now();
    for _ in 0..ITERATIONS {
        // for each iteration the kernel should:
        // 1) load the exception data (CPU state) of the caller into the UTCB of the
        //    receiver after call()
        // 2) copy the exception data (CPU state) in the UTCB of the callee into the
        //    CPU state data structure of the original caller (i.e. the roottask's global EC)
        raw_echo_pt.call().unwrap();
    }
    let dur_exception_ipc = Instant::now() - begin;
    //utcb.disable_store_ipc_exc();
    // ############################################################################
    // MEASURE RAW ECHO SYSCALL PERFORMANCE (pure PD-internal IPC)
    let begin = Instant::now();
    for _ in 0..ITERATIONS {
        raw_echo_pt.call().unwrap();
    }
    let dur_raw_echo = Instant::now() - begin;
    // ############################################################################

    let native_syscall_costs = dur_pt_ctrl / ITERATIONS;
    let echo_call_costs = dur_echo / ITERATIONS;
    let raw_echo_call_costs = dur_raw_echo / ITERATIONS;
    let raw_exception_ipc_costs = dur_exception_ipc / ITERATIONS;

    log::info!(
        "native pt_ctrl syscall costs costs   : {} ticks / pt_ctrl syscall",
        native_syscall_costs
    );
    log::info!(
        "raw echo call costs                  : {} ticks / call syscall (PD-internal IPC)",
        raw_echo_call_costs
    );
    assert!(
        !raw_echo_pt.mtd().is_empty(),
        "for the benchmark, Hedron must handle the MTD as expected! Make sure the PT has an appropriate MTD"
    );
    log::info!(
        "raw exception ipc costs ({:<11}): {} ticks / call syscall (PD-internal exception IPC)",
        if utcb.exception_data().mtd.contains(Mtd::FPU) {
            &"with FPU"
        } else {
            &"without FPU"
        },
        raw_exception_ipc_costs
    );
    log::info!(
        "echo call costs                      : {} ticks / call syscall (PD-internal IPC)",
        echo_call_costs
    );

    log::info!("benchmarking done");
}
