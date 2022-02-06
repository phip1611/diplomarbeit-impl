#!/bin/sh

RUST_HW_LIBC=./target/x86_64-unknown-linux-gnu/debug/hello_world
RUST_HW_MUSL=./target/x86_64-unknown-linux-musl/debug/hello_world



make
echo "####################################"
echo "Rust Hello World: Syscalls with libc"
echo "-----"
strace "${RUST_HW_LIBC}"
echo "####################################"
echo "Rust Hello World: Syscalls with musl"
echo "-----"
strace "${RUST_HW_MUSL}"
