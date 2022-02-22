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
use alloc::string::String;
use alloc::vec::Vec;
use core::arch::global_asm;
use libfileserver::fs_read;
use libhrstd::cap_space::root::RootCapSpace;
use libhrstd::kobjects::SmObject;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::Utcb;
use libhrstd::libhedron::HIP;
use libhrstd::rt::services::fs::FsOpenFlags;
use libhrstd::util::BenchHelper;
use libroottask::process_mng::manager;
use libroottask::rt::userland;
use libroottask::services::init_roottask_echo_pts;
use libroottask::{
    roottask_exception,
    services,
};
use simple_chunk_allocator::DEFAULT_CHUNK_SIZE;

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
        log::debug!("stack top    (incl): 0x{:016x}", roottask_stack::STACK_TOP_PTR.val());
        log::debug!("stack bottom (incl): 0x{:016x}", roottask_stack::STACK_BOTTOM_PTR.val());
        log::debug!("stack size         : {:>18}", roottask_stack::STACK_SIZE);
        log::debug!("stack size (pages) : {:>18}", roottask_stack::STACK_SIZE / PAGE_SIZE);

        log::debug!("heap top    (excl) : 0x{:016x}", roottask_heap::HEAP_END_PTR.val());
        log::debug!("heap bottom (incl) : 0x{:016x}", roottask_heap::HEAP_BEGIN_PTR.val());
        log::debug!("heap size          : {:>18}", roottask_heap::HEAP_SIZE);
        log::debug!("heap size (pages)  : {:>18}", roottask_heap::HEAP_SIZE / PAGE_SIZE);
        log::debug!("heap size (chunks) : {:>18}", roottask_heap::HEAP_SIZE / DEFAULT_CHUNK_SIZE);

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

    let root_process = manager::PROCESS_MNG.lock().root().clone();
    let root_sm = SmObject::create(RootCapSpace::RootSmSleep.val(), &root_process.pd_obj());

    services::init_services(manager::PROCESS_MNG.lock().root());

    log::info!("Rust Roottask started successfully");
    do_bench();

    // NOW READY TO START PROCESSES
    let userland = userland::InitialUserland::load(hip, &root_process);
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
fn do_bench() {
    log::info!("benchmarking starts");
    let (echo_pt, raw_echo_pt) = init_roottask_echo_pts();
    // ############################################################################
    // MEASURE NATIVE SYSTEM CALL PERFORMANCE
    let native_syscall_costs = BenchHelper::bench(|i| unsafe {
        raw_echo_pt.ctrl(i).unwrap();
    });
    // ############################################################################
    // MEASURE ECHO SYSCALL PERFORMANCE (PD-internal IPC with my PT multiplexing mechanism)
    let echo_call_costs = BenchHelper::bench(|_| echo_pt.call().unwrap());
    // ############################################################################
    // MEASURE RAW ECHO SYSCALL PERFORMANCE (pure PD-internal IPC)
    let raw_echo_call_costs = BenchHelper::bench(|_| raw_echo_pt.call().unwrap());
    // ############################################################################
    // MEASURE ROOTTASK ALLOCATION COSTS (1 Byte)
    let alloc_1_byte_costs = BenchHelper::bench(|_| {
        let vec = Vec::<u8>::with_capacity(1);
        unsafe {
            let _x = core::ptr::read_volatile(vec.as_ptr());
        }
    });
    // ############################################################################
    // MEASURE ROOTTASK ALLOCATION COSTS (4096 Byte)
    let alloc_4096_byte_costs = BenchHelper::bench(|_| {
        let vec = Vec::<u8>::with_capacity(4096);
        unsafe {
            let _x = core::ptr::read_volatile(vec.as_ptr());
        }
    });
    // ############################################################################
    // MEASURE FILE SYSTEM PERFORMANCE WITHIN ROOTTASK: open, write &close
    let fs_open_write_close_costs = BenchHelper::bench(|_| {
        let fd = libfileserver::fs_open(
            0,
            String::from("/tmp/roottask_bench1"),
            FsOpenFlags::O_CREAT | FsOpenFlags::O_RDWR,
            0o777,
        );
        let data = [0xd_u8, 0xe, 0xa, 0xd, 0xb, 0xe, 0xe, 0xf];
        libfileserver::fs_write(0, fd, &[0xd, 0xe, 0xa, 0xd, 0xb, 0xe, 0xe, 0xf]).unwrap();
        libfileserver::fs_lseek(0, fd, 0).unwrap();
        let read_data = fs_read(0, fd, data.len()).unwrap();
        assert_eq!(
            &data,
            read_data.as_slice(),
            "written data must equal to read data"
        );
        libfileserver::fs_close(0, fd).unwrap();
    });
    // ############################################################################

    log::info!(
        "native pt_ctrl syscall costs costs: {} ticks / pt_ctrl syscall",
        native_syscall_costs
    );
    log::info!(
        "raw echo call costs               : {} ticks / call syscall (PD-internal IPC)",
        raw_echo_call_costs
    );
    log::info!(
        "echo call costs                   : {} ticks / call syscall (PD-internal IPC)",
        echo_call_costs
    );
    log::info!(
        "roottask 1 bytes mem alloc costs  : {} ticks / allocation (no IPC; pure internal)",
        alloc_1_byte_costs
    );
    log::info!(
        "roottask 4096 byte mem alloc costs: {} ticks / allocation (no IPC; pure internal)",
        alloc_4096_byte_costs
    );
    log::info!(
        "roottask fs open,w+r&close costs  : {} ticks / (open, write, read & close) (no IPC; pure internal)",
        fs_open_write_close_costs
    );

    log::info!("benchmarking done");
}
