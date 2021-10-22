//! Roottask-lib.

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
#![feature(allocator_api)]
#![feature(const_btree_new)]

#[allow(unused)]
#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

#[allow(unused)]
#[macro_use]
extern crate alloc;

#[macro_use]
extern crate libhrstd;

pub mod hw;
pub mod io_port;
pub mod mem;
pub mod process_mng;
pub mod pt_multiplex;
pub mod roottask_exception;
pub mod rt;
pub mod stack;
pub mod static_alloc;
