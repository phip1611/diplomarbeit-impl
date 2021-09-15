# Diplomarbeit: Rust Roottask Prototype

Prototype root task written in Rust, that can be loaded by Hedron.
It needs the following properties:
- bare metal x86_64 binary
- ELF-file where .bss is in .data, because otherwise the .bss section
  during runtime is expected to be larger than in the ELF file
  => Hedron doesn't suppor tthis yet.

### Current Problems with Rust Build
Currently, with Rust/Cargo 1.55-nightly, we are limited by two major Rust/Cargo issues, namely:
- https://github.com/rust-lang/cargo/issues/9710
- https://github.com/rust-lang/cargo/issues/9451 / https://github.com/rust-lang/cargo/pull/9030/

Therefore, we are forced to discard a cargo workspace for now and use a single cargo
project per crate. This has no big disadvantages, despite a larger project
setup and longer build times.

### Roottask Lib
- all generic functionality that should be unit-testable goes here

### Roottask Bin
- uses the library
- as minimum functionality as possible; move everything into lib
  --> see build error noted above
- the produced binary uses the SystemV ABI calling convention
  (I'm not sure why exactly and how I could change that in the target specification json)