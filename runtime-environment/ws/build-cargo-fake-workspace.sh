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
    # find all direct directories without "."
    LIBS=$(find . -maxdepth 1 -type d ! -path . -name "lib*")
    BINS=$(find . -maxdepth 1 -type d ! -path . -name "*-bin")

    for LIB in $LIBS
    do
      fn_build_rust_lib "$LIB" &
    done

    for BIN in $BINS
    do
      fn_build_rust_bin "$BIN" &
    done

    # to optimize build time, I build everything
    # in parallel; "&" creates a bacground task
    # for every compilation!
    wait

    fn_build_extra_checks
}


function fn_build_rust_lib() {
    (
        cd "$1" || exit
        cargo build
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
