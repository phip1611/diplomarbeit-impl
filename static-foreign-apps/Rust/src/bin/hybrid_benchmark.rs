use libhrstd::cap_space::user::UserAppCapSpace;
use libhrstd::kobjects::{LocalEcObject, PdObject, PortalIdentifier, PtCtx, PtObject};
use libhrstd::libhedron::Mtd;
use libhrstd::rt::services::echo::{call_echo_service, call_raw_echo_service};
use libhrstd::util::BenchHelper;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::env::var;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use libhrstd::util::ansi::{AnsiStyle, Color, TextStyle};

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
    linux_bench_expensive_fs_fstat();
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
    let pt = PtObject::create(
        1001,
        &local_ec,
        Mtd::DEFAULT,
        pt_entry,
        PtCtx::ForeignSyscall,
    );

    let duration_per_iteration = BenchHelper::bench(|i| unsafe {
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
    print!(
        "avg: {} ticks / open() syscall",
        duration_per_iteration
    );
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
    print!(
        "avg: {} ticks / fstat() syscall",
        duration_per_iteration
    );
    if var("LINUX_UNDER_HEDRON").is_ok() {
        print!(" (foreign syscall Cross-PD IPC)");
    }
    println!();
    fs::remove_file(path).unwrap();
}

/// Performs the file system microbenchmark that runs under Linux as well as Hedron.
/// Consists of multiple small sub benchmarks.
fn linux_bench_expensive_write_read_lseek_syscalls() {
    println!();
    println!("LINUX BENCH: File throughput performance");
    let bench_file_path = "/tmp/foobar";
    // remove file if it already exists (last iteration panic'ed or so)
    let _ = std::fs::remove_file(bench_file_path);

    // Prepares the file for the benchmark. Makes sure it is freshly created
    // and has a length of zero.
    let before_bench_fn = || {
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(bench_file_path)
            .unwrap()
    };

    // Removes the file after each sub benchmark ran.
    let after_bench_fn = || {
        let _ = std::fs::remove_file(bench_file_path);
    };

    // prints the result of the sub benchmarks in the same format to the screen
    let bench_print_result_fn = |desc: &str, bytes: &[u8], duration_per_iteration: u64| {
        println!(
            "  Bench {:>30} [{} bytes]: avg {:6} ticks / iteration",
            desc,
            AnsiStyle::new()
                .text_style(TextStyle::Bold)
                .msg(&format!("{:>8}", bytes.len())),
            duration_per_iteration
        );
        println!(
            "{:>55}: {} bytes / 1000 ticks",
            "",
            AnsiStyle::new()
                .text_style(TextStyle::Bold)
                .foreground_color(Color::Red)
                .msg(&format!("{:>7.2}", bytes.len() as f64 * 1000.0 / (duration_per_iteration as f64)))
        );
    };

    // performs the sub benchmark with the system calls:
    //  write,lseek,read,lseek
    let do_read_and_print_without_fstat_fn = |desc: &str, bytes: &[u8]| {
        let mut read_vec = Vec::with_capacity(bytes.len() + 1);
        let mut file = before_bench_fn();
        let duration_per_iteration = BenchHelper::bench(|_| {
            file.write_all(bytes).unwrap();
            file.seek(SeekFrom::Start(0)).unwrap();
            file.read(read_vec.as_mut_slice()).unwrap();
            file.seek(SeekFrom::Start(0)).unwrap();
        });
        after_bench_fn();
        bench_print_result_fn(desc, bytes, duration_per_iteration);
    };

    // performs the sub benchmark with the system calls:
    //  write,lseek,fsat,read,lseek
    let do_read_and_print_with_fstat_fn = |desc: &str, bytes: &[u8]| {
        let mut read_vec = Vec::with_capacity(bytes.len() + 1);
        let mut file = before_bench_fn();
        let duration_per_iteration = BenchHelper::bench(|_| {
            read_vec.clear();
            file.write_all(bytes).unwrap();
            file.seek(SeekFrom::Start(0)).unwrap();
            // will trigger an lstat syscall + read syscall
            // (to find out size of file)
            file.read_to_end(&mut read_vec).unwrap();
            file.seek(SeekFrom::Start(0)).unwrap();
        });
        after_bench_fn();
        bench_print_result_fn(desc, bytes, duration_per_iteration);
    };

    let bytes_4kib = [0_u8; 4096];
    let bytes_16kib = [0_u8; 16384];
    let bytes_128kib = [0_u8; 0x20000];
    let bytes_1mib = [0_u8; 1 * 1024 * 1024];

    let bench_sizes = [&bytes_4kib[..], &bytes_16kib[..], &bytes_128kib[..], &bytes_1mib[..]];
    bench_sizes.iter().for_each(|bytes| do_read_and_print_without_fstat_fn("write,lseek,read,lseek", bytes));
    println!("-----------------------------------------------------------------------");
    bench_sizes.iter().for_each(|bytes| do_read_and_print_with_fstat_fn("write,lseek,fstat,read,lseek", bytes));
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
