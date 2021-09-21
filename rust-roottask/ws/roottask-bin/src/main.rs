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

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

use core::ptr;
use roottask_lib::hedron::hip::HIP;
use roottask_lib::hedron::utcb::UtcbData;

// TODO warum geht aktuell noch kein floating point?! nur softfloat..



#[no_mangle]
fn roottask_rust_entry(hip_ptr: u64, utcb_ptr: u64) -> ! {
    let hip = &unsafe { ptr::read(hip_ptr as *const HIP) };
    let _utcb = &unsafe { ptr::read(utcb_ptr as *const UtcbData) };

    logger::init(hip.root_pd());
    roottask_alloc::init();

    log::trace!("trace log");
    log::info!("info log");
    log::debug!("debug log");
    log::warn!("warn log");
    log::error!("error log");

    exception::init(hip);

    // TODO kriege ein divided by zero error sobald ich in
    /*let x = 5.1212 * 1414.2;
    log::debug!("{}", x);*/



    // trigger GPF
    {
        unsafe {
            x86::io::outb(0x0, 0);
        }
    }



    log::info!("Rust Roottask started");
    panic!("SHHIIIIIT");
    loop {}
}


