use std::env::args;
use std::f64::consts::PI;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

// This binary takes the first argument as radius and calculates
// area and circumference of a circle. If no argument is provided, it
// falls back to 42.
fn main() {
    println!("Hello World from a Rust App compiled for Linux");
    let mut args = args().skip(1);
    let radius = args
        .next()
        .map(|x| x.parse::<f64>().ok())
        .flatten()
        .unwrap_or(42.0);
    println!("Circle");
    println!("  Radius       ={:6.2}cm", radius);
    println!("  Area         ={:6.2}cm²", PI * radius.powi(2));
    println!("  Circumference={:6.2}cm", 2.0 * PI * radius);

    let args = std::env::args().collect::<Vec<_>>();
    println!("my args are: {:#?}", args);
    let envs = std::env::vars().collect::<Vec<_>>();
    println!("my envs are: {:#?}", envs);

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open("/tmp/foobar")
        .unwrap();
    let write_msg = "Hello World; it works!!";
    file.write_all(write_msg.as_bytes()).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    let mut read_msg = String::new();
    file.read_to_string(&mut read_msg).unwrap();
    assert_eq!(write_msg, read_msg);
    println!("File content is: '{}'", read_msg);
}
