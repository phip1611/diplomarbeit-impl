use libhrstd::cap_space::user::UserAppCapSpace;
use libhrstd::kobjects::{LocalEcObject, PdObject, PortalIdentifier, PtCtx, PtObject};
use libhrstd::libhedron::Mtd;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::env::var;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use libhrstd::rt::services::echo::{call_echo_service, call_raw_echo_service};
use libhrstd::util::BenchHelper;


// This executable is what I use for the evaluation of
// my diplom thesis. It measures all relevant properties:
// - raw syscall performance
// - file system micro benchmark
// - foreign system call performance
fn main() {
    // to get log messages from libhrstd
    SimpleLogger::new()
        // prevent not supported "clock_gettime"
        // syscall on Hedron :)
        .without_timestamps()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();
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
    linux_bench_expensive_fs_open();
    linux_bench_expensive_write_read_lseek_syscalls();
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
    let pt = PtObject::create(1001, &local_ec, Mtd::DEFAULT, pt_entry, PtCtx::ForeignSyscall);

    let duration_per_iteration = BenchHelper::bench(|i|  unsafe {
        pt.ctrl(i).expect("pt_ctrl must be executed");
    });
    println!("avg: {} ticks / syscall (Native Syscall from Hybrid App)", duration_per_iteration);
}

/// Executes a cheap Linux system call from the Linux App multiple
/// times and calculates the average clock ticks per call.
///
/// This is a Cross-PD IPC.
fn linux_bench_cheap_foreign_set_tid_address_syscall() {
    println!();
    println!("BENCH: CHEAP FOREIGN SYSCALL FROM FOREIGN APP (set_tid_address)");
    let duration_per_iteration = BenchHelper::bench(|_|  unsafe {
        // this is a super cheap syscall and can be used to measure raw
        // foreign syscall path performance
        libc::syscall(libc::SYS_set_tid_address, 0);
    });
    println!("avg: {} ticks / set_tid_address() syscall (foreign syscall Cross-PD IPC)", duration_per_iteration);
}

/// Executes a cheap Linux system call from the Linux App multiple
/// times and calculates the average clock ticks per call.
///
/// This is a Cross-PD IPC.
fn linux_bench_expensive_fs_open() {
    println!();
    println!("BENCH: EXPENSIVE FOREIGN SYSCALL FROM FOREIGN APP (open)");
    let duration_per_iteration = BenchHelper::bench(|_|  unsafe {
        // this is a super cheap syscall and can be used to measure raw
        // foreign syscall path performance
        let _ = OpenOptions::new()
            .create(true)
            .write(true)
            .open("/tmp/diplom_evaluation_test_rwos8uf9sg").unwrap();
    });
    println!("avg: {} ticks / open() syscall (foreign syscall Cross-PD IPC)", duration_per_iteration);
}

/// Executes a cheap Hedron system call from the Linux App multiple
/// times and calculates the average clock ticks per call.
fn linux_bench_expensive_write_read_lseek_syscalls() {
    println!();
    println!("LINUX BENCH: File throughput performance");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open("/tmp/foobar")
        .unwrap();

    let bytes_4096 = [0_u8; 4096];
    let bytes_16384 = [0_u8; 16384];
    let bytes_128kb = [0_u8; 0x20000];
    let bytes_1mb = [0_u8; 1 * 1024 * 1024];

    let mut read_vec = Vec::with_capacity(bytes_1mb.len() + 1);
    let mut write_read = |bytes| {
        read_vec.clear();
        file.seek(SeekFrom::Start(0)).unwrap();
        file.write_all(bytes).unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
        file.read_to_end(&mut read_vec).unwrap();
    };

    let duration_per_iteration_read_write_4096  = BenchHelper::bench(|_|
        write_read(&bytes_4096)
    );
    println!("avg write + read 4096 bytes : {:6} ticks / (write and read)", duration_per_iteration_read_write_4096);
    println!("                            : {:>7.2} bytes / 1000 ticks", 4096.0 * 1000.0 / (duration_per_iteration_read_write_4096 as f64));

    let duration_per_iteration_read_write_16384  = BenchHelper::bench(|_|
        write_read(&bytes_16384)
    );
    println!("avg write + read 16384 bytes: {:6} ticks / (write and read)", duration_per_iteration_read_write_16384);
    println!("                            : {:>7.2} bytes / 1000 ticks", 16384.0 * 1000.0 / (duration_per_iteration_read_write_16384 as f64));

    let duration_per_iteration_read_write_128kib  = BenchHelper::bench(|_|
        write_read(&bytes_128kb)
    );
    println!("avg write + read 128kib     : {:6} ticks / (write and read)", duration_per_iteration_read_write_128kib);
    println!("                            : {:>7.2} bytes / 1000 ticks", bytes_128kb.len() as f64 * 1000.0 / (duration_per_iteration_read_write_128kib as f64));

    let duration_per_iteration_read_write_1mb  = BenchHelper::bench(|_|
        write_read(&bytes_1mb)
    );
    println!("avg write + read 1 mb       : {:6} ticks / (write and read)", duration_per_iteration_read_write_1mb);
    println!("                            : {:>7.2} bytes / 1000 ticks", bytes_1mb.len() as f64 * 1000.0 / (duration_per_iteration_read_write_1mb as f64));
}

/// Calculates the average time to call the RAW ECHO SERVICE PT. This is the raw cost of
/// cross-PD IPC.
fn hedron_bench_raw_echo_pt_call() {
    println!();
    println!("BENCH: RAW ECHO SERVICE PT");
    let duration_per_iteration = BenchHelper::bench(|_| call_raw_echo_service());
    println!("avg: {} ticks / syscall (raw Cross-PD IPC)", duration_per_iteration);
}

/// Calculates the average time to call the REGULAR ECHO SERVICE PT. This is the cost of
/// cross-PD IPC including my PT multiplexing mechanism.
fn hedron_bench_echo_pt_call() {
    println!();
    println!("BENCH: ECHO SERVICE PT");
    let duration_per_iteration = BenchHelper::bench(|_| call_echo_service());
    println!("avg: {} ticks / syscall (Cross-PD IPC)", duration_per_iteration);
}
