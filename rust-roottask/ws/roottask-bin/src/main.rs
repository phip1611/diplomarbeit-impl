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

mod logger;
mod panic;
mod roottask_alloc;

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
    let root_pd_cap_sel = hip.root_pd();

    logger::init(root_pd_cap_sel);
    roottask_alloc::init();

    // test: alloc works
    {
        let mut foo = vec![1, 2, 3, 4, 5, 6, 7, 8];
        log::info!("foo={:#?}", &foo);
        for i in 0..10 {
            log::info!("#{}", i);
            foo.push(i * i);
        }
        log::info!("foo2={:#?}", &foo);
    }

    log::info!("Rust Roottask started");
    panic!("SHHIIIIIT");
    loop {}
}
