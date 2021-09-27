fn main() {
    println!("cargo:rerun-if-changed=src/link.ld");
}
