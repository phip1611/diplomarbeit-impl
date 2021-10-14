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
#![allow(rustdoc::missing_doc_code_examples)]
#![feature(asm)]
#![feature(const_panic)]
#![feature(const_ptr_offset)]
#![feature(const_fmt_arguments_new)]
#![feature(const_mut_refs)]
#![feature(const_fn_trait_bound)]
#![feature(allocator_api)]
#![feature(nonnull_slice_from_raw_parts)]
#![feature(alloc_error_handler)]

#[allow(unused)]
#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

#[allow(unused)]
#[macro_use]
extern crate alloc;

pub use libhedron;
pub use libm;

pub mod cap_mngmt;
pub mod cstr;
pub mod mem;
#[cfg(feature = "rt")]
pub mod rt;
pub mod sync;
pub mod util;
