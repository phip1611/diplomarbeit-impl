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
// I see a benefit here: Even tho it might not be usable from the outside world,
// it may contain useful information about how the implementation works.
#![allow(rustdoc::private_intra_doc_links)]
#![allow(rustdoc::missing_doc_code_examples)]
#![feature(alloc_error_handler)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

mod panic;

#[no_mangle]
fn start() {}
