use std::env::args;
use std::f64::consts::PI;

// This binary takes the first argument as radius and calculates
// area and circumference of a circle. If no argument is provided, it
// falls back to 42.
fn main() {
    println!("Hello World from a Rust App compiled for Linux");
    let mut args = args().skip(1);
    let radius = args.next().unwrap().parse::<f64>().unwrap_or(42.0);
    println!("Circle");
    println!("  Radius       ={:6.2}cm", radius);
    println!("  Area         ={:6.2}cm²", PI * radius.powi(2));
    println!("  Circumference={:6.2}cm", 2.0 * PI * radius);

    let args = std::env::args().collect::<Vec<_>>();
    println!("my args are: {:#?}", args);
    let envs = std::env::vars().collect::<Vec<_>>();
    println!("my envs are: {:#?}", envs);
}
