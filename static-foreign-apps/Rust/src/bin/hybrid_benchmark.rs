use libhrstd::cap_space::user::UserAppCapSpace;
use libhrstd::kobjects::{LocalEcObject, PdObject, PortalIdentifier, PtCtx, PtObject};
use libhrstd::libhedron::Mtd;
use libhrstd::rt::services::echo::{call_echo_service, call_raw_echo_service};
use libhrstd::time::Instant;
use libhrstd::util::BenchHelper;
use log::{Metadata, Record};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::env::var;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::rc::Rc;

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        println!(
            "[{level:>5}@{line}]: {msg}",
            level = record.level(),
            line = record.line().unwrap_or(0),
            msg = record.args()
        );
    }

    fn flush(&self) {}
}

// This executable is what I use for the evaluation of
// my diplom thesis. It measures all relevant properties:
// - raw syscall performance
// - file system micro benchmark
// - foreign system call performance
fn main() {
    log::set_max_level(log::LevelFilter::Debug);
    log::set_logger(&Logger).unwrap();
    println!("Hello world from Hybrid Foreign Benchmark!");

    if var("LINUX_UNDER_HEDRON").is_ok() {
        println!("This Linux binary runs as a hybrid foreign application under Hedron");
        hedron_hybrid_bench_native_pt_ctrl_syscall();
        hedron_bench_raw_echo_pt_call();
        hedron_bench_echo_pt_call();
    } else {
        println!("This Linux binary executes under native Linux");
    }

    linux_bench_cheap_foreign_set_tid_address_syscall();
    linux_bench_expensive_fs_fstat();
    linux_bench_expensive_fs_open();
    linux_bench_file_system_microbenchmark();
}

fn pt_entry(_id: PortalIdentifier) -> ! {
    panic!()
}

/// Executes a Hedron syscall from a foreign app multiple
/// times and calculates the average clock ticks per call.
fn hedron_hybrid_bench_native_pt_ctrl_syscall() {
    println!();
    println!("BENCH: NATIVE SYSCALL FROM HYBRID FOREIGN APP");
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

    let duration_per_iteration = BenchHelper::<_>::bench_direct(|i| unsafe {
        pt.ctrl(i).expect("pt_ctrl must be executed");
    });
    println!(
        "avg: {} ticks / syscall (Native Syscall from Hybrid App)",
        duration_per_iteration
    );
}

/// Executes a cheap Linux system call from the Linux App multiple
/// times and calculates the average clock ticks per call.
///
/// This is a Cross-PD IPC.
fn linux_bench_cheap_foreign_set_tid_address_syscall() {
    println!();
    println!("BENCH: CHEAP FOREIGN set_tid_address SYSCALL");
    let duration_per_iteration = BenchHelper::<_>::bench_direct(|_| unsafe {
        // this is a super cheap syscall and can be used to measure raw
        // foreign syscall path performance
        libc::syscall(libc::SYS_set_tid_address, 0);
    });
    print!(
        "avg: {} ticks / set_tid_address() syscall",
        duration_per_iteration
    );
    if var("LINUX_UNDER_HEDRON").is_ok() {
        print!(" (foreign syscall Cross-PD IPC)");
    }
    println!();
}

/// Executes a cheap Linux system call from the Linux App multiple
/// times and calculates the average clock ticks per call.
///
/// This is a Cross-PD IPC.
fn linux_bench_expensive_fs_open() {
    println!();
    println!("BENCH: EXPENSIVE FOREIGN open SYSCALL");
    let path = "/tmp/diplom_evaluation_test_rwos8uf9sg";
    // remove in case it exists
    //let _ = fs::remove_file(path);
    let duration_per_iteration = BenchHelper::<_>::bench_direct(|_| {
        // this is a super cheap syscall and can be used to measure raw
        // foreign syscall path performance
        let _ = OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)
            .unwrap();
    });
    print!("avg: {} ticks / open() syscall", duration_per_iteration);
    if var("LINUX_UNDER_HEDRON").is_ok() {
        print!(" (foreign syscall Cross-PD IPC)");
    }
    println!();
    //fs::remove_file(path).unwrap();
}

/// Executes a cheap Linux system call from the Linux App multiple
/// times and calculates the average clock ticks per call.
///
/// This is a Cross-PD IPC.
fn linux_bench_expensive_fs_fstat() {
    println!();
    println!("BENCH: EXPENSIVE FOREIGN fstat SYSCALL)");
    let path = "/tmp/diplom_evaluation_test_r15156sg";
    // this is a super cheap syscall and can be used to measure raw
    // foreign syscall path performance
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .unwrap();
    let duration_per_iteration = BenchHelper::<_>::bench_direct(|_| {
        // performs a fstat system call
        // Observation: Under GNU/Linux this uses "statx" syscall instead of
        // fstat but the result/overhead is similar.
        let metadata = file.metadata().unwrap();
        unsafe {
            // prevent compiler optimizations
            core::ptr::read_volatile(core::ptr::addr_of!(metadata));
        }
    });
    print!("avg: {} ticks / fstat() syscall", duration_per_iteration);
    if var("LINUX_UNDER_HEDRON").is_ok() {
        print!(" (foreign syscall Cross-PD IPC)");
    }
    println!();
    fs::remove_file(path).unwrap();
}

/// Performs the file system microbenchmark that runs under Linux as well as Hedron.
/// Consists of multiple small sub benchmarks.
fn linux_bench_file_system_microbenchmark() {
    println!();
    println!("LINUX BENCH: File System Microbenchmark");
    let bench_file_path = "/tmp/foobar";
    // remove file if it already exists (last iteration panic'ed or so)
    let _ = std::fs::remove_file(bench_file_path);

    let mut bench_results = BTreeMap::new();

    let buffer_sizes = [
        0x4000,  // 16KiB
        0x8000,  // 32 KiB
        0x10000, // 64 KiB
        0x20000, // 128 KiB
        0x40000, // 256 KiB
    ];
    let file_sizes = [
        0x10000,  // 64 KiB
        0x100000, // 1 MiB
    ];

    for file_size in file_sizes {
        for buffer_size in buffer_sizes {
            if buffer_size > file_size {
                continue;
            }

            let file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .read(true)
                .write(true)
                .append(false)
                .open(bench_file_path)
                .unwrap();

            // fast workaround: I need a mutable reference in multiple callbacks.
            let file = Rc::new(RefCell::new(file));

            // allocate a buffer with "random" data. I use this to ensure that the read and the write
            // operation operate on the same data (to detect possible bugs in my in-mem file system)
            let mut data_to_write = Vec::with_capacity(file_size);
            (0..file_size)
                .map(|_| get_random_u64())
                .flat_map(|val| val.to_ne_bytes())
                .take(file_size)
                .for_each(|val| data_to_write.push(val));

            let write_bench_result = {
                let mut write_bench_after_each_fnc = || {
                    file.borrow_mut().seek(SeekFrom::Start(0)).unwrap();
                };
                let mut write_bench = BenchHelper::<_>::new(|_| {
                    let mut file = file.borrow_mut();
                    let total_bytes_written = data_to_write
                        .as_slice()
                        .chunks(buffer_size)
                        .map(|bytes| file.write(bytes))
                        .map(|res| res.expect("must work in the benchmark"))
                        .sum::<usize>();
                    assert_eq!(
                        total_bytes_written, file_size,
                        "written bytes must match the file size! all bytes must be written!"
                    );
                });
                write_bench.with_after_each(&mut write_bench_after_each_fnc);
                write_bench.bench()
            };

            // ################################################################

            let mut read_buffer = Vec::with_capacity(file_size);
            (0..file_size).for_each(|_| read_buffer.push(0));

            let read_bench_result = {
                let mut read_bench_after_each_fnc = || {
                    file.borrow_mut().seek(SeekFrom::Start(0)).unwrap();
                };
                let mut read_bench = BenchHelper::<_>::new(|_| {
                    let mut file = file.borrow_mut();
                    let total_bytes_read = read_buffer
                        .as_mut_slice()
                        .chunks_mut(buffer_size)
                        .map(|bytes| file.read(bytes))
                        .map(|res| res.expect("must work inside the benchmark"))
                        .sum::<usize>();
                    assert_eq!(
                        total_bytes_read, file_size,
                        "read bytes must match the file size! all bytes must be read!"
                    );
                });
                read_bench.with_after_each(&mut read_bench_after_each_fnc);
                read_bench.bench()
            };

            assert_eq!(
                data_to_write, read_buffer,
                "written data must match read data! file_size={file_size}, buffer_size={buffer_size}"
            );

            // insert result into result map
            let _ = bench_results.insert(
                (file_size, buffer_size),
                (write_bench_result, read_bench_result),
            );

            let _ = std::fs::remove_file(bench_file_path);
        }
    }

    // OUTPUT in a CSV-like format so that I can easily copy it to a google sheets

    {
        println!("heading column:");
        for ((file_size, buffer_size), _) in &bench_results {
            println!(
                "write [file_size={file_size:8}, buf_size={buffer_size:8}]"
            );
        }
        for ((file_size, buffer_size), _) in &bench_results {
            println!(
                "read  [file_size={file_size:8}, buf_size={buffer_size:8}]"
            );
        }

        println!("data column:");
        for (_, (write_res, _read_res)) in &bench_results {
            println!(
                "{write_res}"
            );
        }
        for (_, (_write_res, read_res)) in &bench_results {
            println!(
                "{read_res}"
            );
        }


    }



    let _ = std::fs::remove_file(bench_file_path);
}

/// Calculates the average time to call the RAW ECHO SERVICE PT. This is the raw cost of
/// cross-PD IPC.
fn hedron_bench_raw_echo_pt_call() {
    println!();
    println!("BENCH: RAW ECHO SERVICE PT");
    let duration_per_iteration = BenchHelper::<_>::bench_direct(|_| call_raw_echo_service());
    println!(
        "avg: {} ticks / syscall (raw Cross-PD IPC)",
        duration_per_iteration
    );
}

/// Calculates the average time to call the REGULAR ECHO SERVICE PT. This is the cost of
/// cross-PD IPC including my PT multiplexing mechanism.
fn hedron_bench_echo_pt_call() {
    println!();
    println!("BENCH: ECHO SERVICE PT");
    let duration_per_iteration = BenchHelper::<_>::bench_direct(|_| call_echo_service());
    println!(
        "avg: {} ticks / syscall (Cross-PD IPC)",
        duration_per_iteration
    );
}

// when this runs under Hedron I can't use cool and fancy features of the "rand" library.
// I use the timestamp counter instead.
fn get_random_u64() -> u64 {
    Instant::now().val()
}
