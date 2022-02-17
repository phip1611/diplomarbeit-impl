use libhrstd::cap_space::user::UserAppCapSpace;
use libhrstd::kobjects::{LocalEcObject, PdObject, PortalIdentifier, PtCtx, PtObject};
use libhrstd::libhedron::Mtd;
use libhrstd::time::Instant as HedronInstant;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::env::var;
use libhrstd::rt::services::echo::{call_echo_service, call_raw_echo_service};


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
        hedron_bench_foreign_set_tid_address_syscall();
        hedron_bench_raw_echo_pt_call();
        hedron_bench_echo_pt_call();
    } else {
        println!("This Linux binary executes under native Linux");
        linux_bench_read_syscall();
    }
}

fn pt_entry(_id: PortalIdentifier) -> ! {
    panic!()
}

/// Executes a Hedron syscall from a foreign app multiple
/// times and calculates the average clock ticks per call.
fn hedron_hybrid_bench_native_pt_ctrl_syscall() {
    println!("BENCH: NATIVE SYSCALL FROM HYBRID FOREIGN APP");
    let self_pd = PdObject::self_in_user_cap_space(UserAppCapSpace::Pd.val());
    // I never use the local ec; i.e. call a PT on it; I just need it to attach a PT to it for the
    // benchmark.
    let local_ec = LocalEcObject::create(1000, &self_pd, 0xf00ba1, 0xdeadb000);
    // some PT I never use; I just need it to be created
    let pt = PtObject::create(1001, &local_ec, Mtd::DEFAULT, pt_entry, PtCtx::ForeignSyscall);

    let start = HedronInstant::now();
    let iterations = 1_000_000;
    for i in 0..iterations {
        unsafe {
            pt.ctrl(i).expect("pt_ctrl must be executed");
        }
    }
    let dur = HedronInstant::now() - start;
    println!("{}x pt_ctrl took {} ticks", iterations, dur);
    println!("avg: {} ticks / syscall (Native Syscall from Hybrid App)", dur / iterations);
}

/// Executes a cheap Linux system call from the Linux App multiple
/// times and calculates the average clock ticks per call.
///
/// This is a Cross-PD IPC.
fn hedron_bench_foreign_set_tid_address_syscall() {
    println!("BENCH: FOREIGN SYSCALL FROM FOREIGN APP");
    let iterations = 1_000_000;
    let begin = HedronInstant::now();
    for _ in 0..iterations {
        unsafe {
            // this is a super cheap syscall and can be used to measure raw
            // foreign syscall performance
            libc::syscall(libc::SYS_set_tid_address);
        }
    }
    let duration_ticks = HedronInstant::now() - begin;
    println!("{}x set_tid_address took {} ticks", iterations, duration_ticks);
    println!("avg: {} ticks / syscall (Cross-PD IPC)", duration_ticks / iterations);
}

/// Executes a cheap Hedron system call from the Linux App multiple
/// times and calculates the average clock ticks per call.
fn linux_bench_read_syscall() {
    // TODO rethink bench
    println!("LINUX BENCH: Raw system call performance");
    let fd = unsafe {
        libc::open("/dev/zero".as_ptr().cast(), libc::O_RDONLY)
    };
    let iterations = 1_000_000;
    let begin = HedronInstant::now();
    let mut buf = [0_u8];
    for _ in 0..iterations {
        unsafe {
            libc::read(fd, buf.as_mut_ptr().cast(), 1);
        }
    }
    let duration_ticks = HedronInstant::now() - begin;
    println!("{}x read took {} ticks", iterations, duration_ticks);
    println!("avg: {} ticks / syscall", duration_ticks / iterations);
}

/// Calculates the average time to call the RAW ECHO SERVICE PT. This is the raw cost of
/// cross-PD IPC.
fn hedron_bench_raw_echo_pt_call() {
    println!("BENCH: RAW ECHO SERVICE PT");

    let iterations = 10_000_000;
    let begin = HedronInstant::now();
    for _ in 0..iterations {
        call_raw_echo_service();
    }
    let duration_ticks = HedronInstant::now() - begin;
    println!("{}x calling raw echo service took {} ticks", iterations, duration_ticks);
    println!("avg: {} ticks / syscall (Cross-PD IPC)", duration_ticks / iterations);
}

/// Calculates the average time to call the REGULAR ECHO SERVICE PT. This is the cost of
/// cross-PD IPC including my PT multiplexing mechanism.
fn hedron_bench_echo_pt_call() {
    println!("BENCH: ECHO SERVICE PT");

    let iterations = 10_000_000;
    let begin = HedronInstant::now();
    for _ in 0..iterations {
        call_echo_service();
    }
    let duration_ticks = HedronInstant::now() - begin;
    println!("{}x calling regular echo service took {} ticks", iterations, duration_ticks);
    println!("avg: {} ticks / syscall (Cross-PD IPC)", duration_ticks / iterations);
}
