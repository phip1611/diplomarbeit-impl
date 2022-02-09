#!/usr/bin/env bash

set -e

# See README.md -> Current Problems with Rust Build
# for details on this.
# This file builds all rust projects, formats all of them,
# executes clippy and tests.

#########################################################################
# nice "hack" which make the script work, even if not executed from "./"
DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit
#########################################################################

function fn_main() {
    # check ENV var exists
    if ! [[ $CARGO_TARGET_DIR ]]; then
        echo "ENV VAR CARGO_TARGET_DIR is missing"
        exit 1
    fi



    # find all direct directories without "."
    LIBS=$(find . -maxdepth 1 -type d ! -path . -name "lib*")
    BINS=$(find . -maxdepth 1 -type d ! -path . -name "*-bin")

    # I tried to parallelize this with "&" (put into background), but this
    # is error prone and breaks much developer convenience. The output
    # gets so polluted that it doesn't really stop there where it found
    # a compilation error.

    for LIB in $LIBS; do
        fn_build_rust_lib "$LIB"
    done

    for BIN in $BINS; do
        fn_build_rust_bin "$BIN"
    done

    fn_build_extra_checks
}

function fn_build_rust_lib() {
    (
        cd "$1" || exit
        # Check here is enough (no release build required) because even with a shared cargo target
        # dir, the build of the binaries compiles the library again.
        cargo check
        cargo test
        cargo fmt # automatically format everything
        # cargo fmt -- --check
        # cargo clippy
        # cargo doc
    )
}

function fn_build_rust_bin() {
    (
        cd "$1" || exit
        cargo build
        cargo build --release
        cargo fmt # automatically format everything
        # cargo fmt -- --check
        # cargo clippy
        # cargo doc
    )
}

# some extra checks that I can not cover with the stuff above..
function fn_build_extra_checks() {
    (
        cd "libhrstd" || exit
        cargo check --no-default-features --features foreign_rust_rt
    )
}

fn_main
