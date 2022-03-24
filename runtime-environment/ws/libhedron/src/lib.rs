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
// I can not influence this; this is the problem of some dependencies
#![allow(clippy::multiple_crate_versions)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::all)]
// I see a benefit here: Even tho it might not be usable from the outside world,
// it may contain useful information about how the implementation works.
#![allow(rustdoc::private_intra_doc_links)]
#![allow(rustdoc::missing_doc_code_examples)]
#![feature(const_ptr_offset)]
#![feature(const_fmt_arguments_new)]
#![feature(const_mut_refs)]

#[allow(unused)]
#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

mod acpi_gas;
pub use acpi_gas::*;
mod capability;
pub use capability::*;
pub mod consts;
mod cpu;
pub use cpu::*;
mod event_offset;
pub use event_offset::*;
mod hip;
pub use hip::*;
pub mod mem;
mod mtd;
pub use mtd::Mtd;
mod qpd;
pub use qpd::Qpd;
mod utcb;
pub use utcb::*;
pub mod syscall;

/// Re-export the `postcard`-version required for serialization of arbitrary UTCB data.
pub use postcard as ipc_postcard;
/// Re-export the `no_std` serde-version required for serialization of arbitrary UTCB data.
pub use serde as ipc_serde;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
