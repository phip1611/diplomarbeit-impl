//! Type definitions and low-level system call wrappers for Hedron.

#![cfg_attr(not(test), no_std)]
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
#![feature(asm)]
#![feature(const_panic)]
#![feature(const_ptr_offset)]
#![feature(const_fmt_arguments_new)]
#![feature(const_mut_refs)]

#[allow(unused)]
#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

#[allow(unused)]
#[macro_use]
extern crate alloc;

pub mod acpi_gas;
pub mod capability;
pub mod consts;
pub mod cpu;
pub mod event_offset;
pub mod hip;
pub mod mem;
pub mod mtd;
pub mod qpd;
pub mod syscall;
pub mod utcb;

// /// Re-export the `postcard`-version required for serialization of arbitrary UTCB data.
// pub use postcard as ipc_postcard;
/// Re-export the `no_std` serde-version required for serialization of arbitrary UTCB data.
pub use serde as ipc_serde;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
