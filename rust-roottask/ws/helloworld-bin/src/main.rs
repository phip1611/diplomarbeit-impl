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

use alloc::string::String;
use alloc::vec::Vec;
use libhrstd::mem::UserPtrOrEmbedded;
use libhrstd::rt::services::fs::fs_lseek::{
    fs_lseek,
    FsLseekRequest,
};
use libhrstd::rt::services::fs::fs_open::{
    fs_open,
    FsOpenFlags,
    FsOpenRequest,
};
use libhrstd::rt::services::fs::fs_read::{
    fs_read,
    FsReadRequest,
};
use libhrstd::rt::services::fs::fs_write::{
    fs_write,
    FsWriteRequest,
};
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

    fs_test();

    loop {}
}

fn fs_test() {
    let fd = fs_open(FsOpenRequest::new(
        String::from("/foo/bar"),
        FsOpenFlags::O_CREAT | FsOpenFlags::O_RDWR,
        0o777,
    ));
    /*let bytes_written = */
    fs_write(FsWriteRequest::new(
        fd,
        UserPtrOrEmbedded::new_slice(b"Hallo Welt!"),
        b"Hallo Welt!".len(),
    ));
    fs_lseek(FsLseekRequest::new(fd, "Hallo ".len() as u64));
    let mut read_buf = Vec::with_capacity(100);
    /*let read_bytes = */
    fs_read(FsReadRequest::new(
        fd,
        read_buf.as_mut_ptr() as usize,
        read_buf.capacity(),
    ));
    let read = String::from_utf8(read_buf).unwrap();
    assert_eq!(read, "Welt!");

    fs_lseek(FsLseekRequest::new(fd, 0));
    let mut read_buf = Vec::with_capacity(100);
    /*let read = */
    fs_read(FsReadRequest::new(
        fd,
        read_buf.as_mut_ptr() as usize,
        read.capacity(),
    ));
    let read = String::from_utf8(read_buf).unwrap();
    assert_eq!(read, "Hallo Welt!")
}
