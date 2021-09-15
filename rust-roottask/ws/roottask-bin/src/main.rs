#![no_std]
#![no_main]
// allow inline assembly
#![feature(asm)]
// allow global assembly
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
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

global_asm!(include_str!("assembly.S"));

mod logger;
mod panic;
mod rootask_alloc; // any global definitions required to be in assembly

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

use core::ptr;

use alloc::fmt::format;
use arrayvec::ArrayString;
use roottask_lib::hedron::capability::CrdPortIO;
use roottask_lib::hedron::hip::HIP;
use roottask_lib::hedron::utcb::UtcbData;
use roottask_lib::hrstd::io_port::request_io_ports;
use roottask_lib::hw::serial_port::snd_serial;

#[no_mangle]
fn roottask_rust_entry(hip_ptr: u64, utcb_ptr: u64) -> ! {
    let hip = &unsafe { ptr::read(hip_ptr as *const HIP) };
    let _utcb = &unsafe { ptr::read(utcb_ptr as *const UtcbData) };
    let root_pd_cap_sel = hip.root_pd();

    logger::init(root_pd_cap_sel);
    let mut buf = ArrayString::<64>::new();
    log::info!("Rust Roottask started");
    panic!("SHHIIIIIT");
    loop {}
}
