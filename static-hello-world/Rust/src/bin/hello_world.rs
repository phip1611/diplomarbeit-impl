fn main() {
    println!("Hello world from Rust!");
    let args = std::env::args().collect::<Vec<_>>();
    println!("my args are: {:#?}", args);
    let envs = std::env::vars().collect::<Vec<_>>();
    println!("my envs are: {:#?}", envs);
}
