# Diplomarbeit: A Policy-free System Call Layer for Hedron
This repository contains a completely new roottask and a corresponding runtime environment written in Rust
for the Hedron microhypervisor. The system enables the co-existence of Hedron-native applications and unmodified
static Linux binaries under Hedron. It only supports a small sub-set of Linux syscalls and only "regular"
applications, that are no daemons.

## Architecture
TODO insert figure

### libhedron
- raw system call wrappers
- type definitions

### libhrstd (Hedron Rust Standard Library)
- primitives for synchronization
- all kinds of utility functions that are relevant
  - (e.g. ANSI term colors)
  - memory utilities (wrappers for alignment)
- enables access to the functionality of the runtime system (except for the roottask)
  - logging (`log` crate - typical Rust logging)
  - allocating (typical Rust allocations backed up by a custom allocator)

## Build
You need rustup. The build uses the Cargo and Rustc version defined in the `rust-toolchain.toml` file.

### Current Problems with Cargo
Currently, with Rust/Cargo 1.55-nightly, we are limited by two major Rust/Cargo issues, namely:
- https://github.com/rust-lang/cargo/issues/9710
- https://github.com/rust-lang/cargo/issues/9451 / https://github.com/rust-lang/cargo/pull/9030/

Therefore, we are forced to discard a cargo workspace for now and use a single cargo
project per crate. This has no big disadvantages, despite a larger project
setup and longer build times.

### `build.sh`
Builds the whole workspace and packs all binaries into the `./build` directory.
The runtime environment, i.e. all binaries except the roottask, gets bundled into
an archive file.


## Run
The roottask + the runtime environment can be started in QEMU via `$ ./run_qemu.sh`.
