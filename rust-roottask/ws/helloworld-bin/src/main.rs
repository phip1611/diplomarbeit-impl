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

use alloc::string::String;
use alloc::vec::Vec;
use libhrstd::cap_space::user::UserAppCapSpace;
use libhrstd::kobjects::{
    LocalEcObject,
    PdObject,
    PortalIdentifier,
    PtCtx,
    PtObject,
};
use libhrstd::libhedron::syscall::sys_pt_ctrl;
use libhrstd::libhedron::Mtd;
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
use libhrstd::time::Instant;

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

    hedron_bench_native_syscall();

    loop {}
}

fn fs_test() {
    let fd = fs_open(FsOpenRequest::new(
        String::from("/foo/bar"),
        FsOpenFlags::O_CREAT | FsOpenFlags::O_RDWR,
        0o777,
    ));

    fs_write(FsWriteRequest::new(
        fd,
        UserPtrOrEmbedded::new_slice(b"Hallo Welt!"),
        b"Hallo Welt!".len(),
    ));

    fs_lseek(FsLseekRequest::new(fd, "Hallo ".len() as u64));
    let mut read_buf = Vec::with_capacity(100);

    let read_bytes = fs_read(FsReadRequest::new(
        fd,
        read_buf.as_mut_ptr() as usize,
        read_buf.capacity(),
    ));

    unsafe {
        read_buf.set_len(read_bytes);
    };
    let read = String::from_utf8(read_buf).unwrap();
    assert_eq!(read, "Welt!");

    fs_lseek(FsLseekRequest::new(fd, 0));
    let mut read_buf = Vec::with_capacity(100);

    let read_bytes = fs_read(FsReadRequest::new(
        fd,
        read_buf.as_mut_ptr() as usize,
        read.capacity(),
    ));
    unsafe {
        read_buf.set_len(read_bytes);
    };

    let read = String::from_utf8(read_buf).unwrap();
    assert_eq!(read, "Hallo Welt!")
}

/// Executes a Hedron syscall from a foreign app multiple
/// times and calculates the average clock ticks per call.
fn hedron_bench_native_syscall() {
    log::info!("BENCH: NATIVE SYSCALL FROM HEDRON NATIVE APP");
    let self_pd = PdObject::self_in_user_cap_space(UserAppCapSpace::Pd.val());
    // I never use the local ec; i.e. call a PT on it; I just need it to attach a PT to it for the
    // benchmark.
    let local_ec = LocalEcObject::create(1000, &self_pd, 0xf00ba1, 0xdeadb000);
    // some PT I never use; I just need it to be created
    let pt = PtObject::create(
        1001,
        &local_ec,
        Mtd::DEFAULT,
        pt_entry,
        PtCtx::ForeignSyscall,
    );

    let start = Instant::now();
    let iterations = 100_000;
    for i in 0..iterations {
        sys_pt_ctrl(pt.cap_sel(), i).expect("pt_ctrl must be executed");
    }
    let dur = Instant::now() - start;

    log::info!("{}x pt_ctrl took {} ticks", iterations, dur);
    log::info!("avg: {} ticks / sys call", dur / iterations);
}

fn pt_entry(_id: PortalIdentifier) -> ! {
    panic!()
}
