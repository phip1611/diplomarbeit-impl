use libhrstd::kobjects::PdObject;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::env::var;

fn main() {
    // to get log messages from libhrstd
    SimpleLogger::new()
        // prevent not supported "clock_gettime"
        // syscall on Hedron :)
        .without_timestamps()
        .with_level(LevelFilter::max())
        .init()
        .unwrap();

    println!("Hello world from hybrid Linux application written in Rust!");

    if var("LINUX_UNDER_HEDRON").is_ok() {
        println!("This Linux binary executes under Hedron");
        let self_pd = PdObject::self_in_user_cap_space(1);
        println!("Executing Hedron-native system call now:");
        let new_pd = PdObject::create(2, &self_pd, 1000, None);
        println!(
            "Executed Hedron-native system call successfully! new PD: {:#?}",
            new_pd
        );

        println!("Hedron service PTs can also be called:");
        libhrstd::rt::services::stderr::stderr_service("direct STDERR PT call from hybrid part in Linux App. YAY");
        println!("hybrid part done");
    } else {
        println!("This Linux binary executes under native Linux");
    }
}
