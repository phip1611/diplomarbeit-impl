#![no_std]
#![no_main]
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
#![allow(rustdoc::missing_doc_code_examples)]
#![feature(alloc_error_handler)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

use libhrstd::rt::services::stderr::stderr_write;
use libhrstd::rt::services::stdout::stdout_write;
use libhrstd::rt::user_logger::UserRustLogger;

mod panic;

#[no_mangle]
fn start() {
    UserRustLogger::init();
    let msg = "Hallo Welt Lorem Ipsum Dolor sit Damet.";
    stdout_write(msg);
    stderr_write(msg);
    log::info!("log info msg");
    log::debug!("log debug msg");
    log::warn!("log warn msg");
    log::error!("log error msg");
    log::trace!("log trace msg");

    let mut nums = vec![1, 2, 3, 4, 5];
    nums.push(7);
    log::info!("nums: {:#?}", nums);

    loop {}
}
