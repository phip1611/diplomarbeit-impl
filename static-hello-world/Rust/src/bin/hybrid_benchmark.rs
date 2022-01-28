use libhrstd::cap_space::user::UserAppCapSpace;
use libhrstd::kobjects::{LocalEcObject, PdObject, PortalIdentifier, PtCtx, PtObject};
use libhrstd::libhedron::Mtd;
use libhrstd::rt::hybrid_rt::syscalls::{sys_hybrid_create_pt, sys_hybrid_pt_ctrl};
use libhrstd::time::Instant as HedronInstant;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::env::var;

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
    println!("Hello, world from Hybrid Foreign Benchmark!");

    if var("LINUX_UNDER_HEDRON").is_ok() {
        let self_pd = PdObject::self_in_user_cap_space(UserAppCapSpace::Pd.val());
        // I never use the local ec; i.e. call a PT on it; I just need it to attach a PT to it for the
        // benchmark.
        let local_ec = LocalEcObject::create(1000, &self_pd, 0xf00ba1, 0xdeadb000);
        // some PT I never use; I just need it to be created
        let pt = PtObject::create(1001, &local_ec, Mtd::DEFAULT, pt_entry, PtCtx::ForeignSyscall);

        let start = HedronInstant::now();
        let iterations = 2000000;
        for i in 0..iterations {
            sys_hybrid_pt_ctrl(pt.cap_sel(), i).expect("pt_ctrl must be executed");
        }
        let dur = HedronInstant::now() - start;
        println!("{}x pt_ctrl took {} ticks", iterations, dur);
        println!("avg: {} ticks / sys call", dur / iterations);
    } else {
        println!("This Linux binary executes under native Linux");
    }
}

fn pt_entry(id: PortalIdentifier) -> ! {
    panic!()
}
