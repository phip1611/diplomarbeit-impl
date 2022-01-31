//! **H**edron **R**ust **St**andar**d** Library.
//! High level utility functions around the low level syscalls.
//! Standard library for Rust apps under Hedron.

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
// I see a benefit here: Even tho it might not be usable from the outside world,
// it may contain useful information about how the implementation works.
#![allow(rustdoc::private_intra_doc_links)]
#![allow(rustdoc::missing_doc_code_examples)]
#![feature(const_ptr_offset)]
#![feature(const_fmt_arguments_new)]
#![feature(const_mut_refs)]
#![feature(const_fn_trait_bound)]
#![feature(allocator_api)]
#![feature(nonnull_slice_from_raw_parts)]
#![feature(alloc_error_handler)]
#![feature(const_btree_new)]
#![feature(panic_info_message)]

#[allow(unused)]
#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

#[allow(unused)]
#[macro_use]
extern crate alloc;

pub use libm;

pub use libhedron;

#[macro_use]
pub mod util;
pub mod cap_space;
pub mod cstr;
pub mod kobjects;
pub mod mem;
pub mod process;
pub mod rt;
pub mod service_ids;
pub mod sync;
pub mod time;
pub mod uaddress_space;
