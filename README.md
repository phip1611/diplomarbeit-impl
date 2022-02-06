# Git Submodules
- musl
    - required for the C programs builds with musl
    - Rust musl targets have the lib already bundled inside the rust toolchain

# Build
- git submodule update --init --recursive
## Required Packages and Tooling
- sudo apt install build-essential make cmake
- rustup: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
## QEMU (`qemu-system-x86_64`) 6.2 or above
Older QEMU versions are really slow when larger payloads are used as multiboot modules.
This patch is only avaialble with 6.2 or above (https://gitlab.com/qemu-project/qemu/-/commit/48972f8cad24eb4462c97ea68003e2dd35be0444)


This repository contains all binaries, tools, and programs of my Diplomarbeit.

- static hello world binaries for testing
- the in-memory file system apps (app + server) for evaluation
- the hedron root task + other relevant crates
