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
    # find all direct directories without "." and without libhedron (see below)
    LIBS=$(find . -maxdepth 1 -type d ! -path . -name "lib*")
    BINS=$(find . -maxdepth 1 -type d ! -path . -name "*-bin")

    # I tried to parallelize this via background tasks, but this leads to build
    # instabilities when the static binaries get build. It may happen that
    # two cargo processes uses rustup to download further compiler targets or
    # new compiler versions. This will fail.

    for LIB in $LIBS
    do
      fn_build_rust_lib "$LIB"
    done

    for BIN in $BINS
    do
      fn_build_rust_bin "$BIN"
    done

    fn_build_extra_checks
}


function fn_build_rust_lib() {
    (
        cd "$1" || exit
        # For libraries it is okay to just "check".
        # The libraries will be used by the binaries anyway,
        # so "build" gets tested.
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
