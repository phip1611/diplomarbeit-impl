# Diplomarbeit: A Policy-free System Call Layer for Hedron
This repository contains a completely new roottask and a corresponding runtime environment written in Rust
for the Hedron microhypervisor. The system enables the co-existence of Hedron-native applications and unmodified
static Linux binaries under Hedron. It only supports a small sub-set of Linux syscalls and only "regular"
applications, that are no daemons. The technique added to Hedron and supported
by this userland enables all kind of foreign syscall ABIs and not just Linux. For simplicity
of this work, this work only focus on the most important Linux syscalls.

## Architecture
TODO insert figure

### libhedron
- raw system call wrappers
- type definitions for hardware
- UTCB and HIP
- Hedron constants

### libhrstd (Hedron Rust Standard Library)
- primitives for synchronization
- all kinds of utility functions (e.g. ANSI term colors)
- memory utilities (wrappers for alignment)
- enables user applications (not roottask) access to the functionality of the runtime system via UTCB IPC
  - write to stdout/stderr
  - contains a Rust logger (`log::info!()` that maps to stdout/stderr)
  - allocations (including a Global Allocator for Rust runtime)
  - file open, file write, file read, file close

### libfileserver
- only used by roottask (**so far no dedicated file system service, to save time)
- implements the internal data structures to manage files (manage FDs per PID, manage files in a in memory data structure)
- all (testable) functionality of the filesystem service

### libroottask
- only used by roottask
- all (testable) functionality of the roottask

### roottask-bin
- Rust-related binary stuff (linker script, panic handler) + libroottask functionality

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
