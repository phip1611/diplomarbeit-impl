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
    LIBS=$(find . -maxdepth 1 -type d ! -path . ! -name "libhedron" -name "lib*")
    BINS=$(find . -maxdepth 1 -type d ! -path . -name "*-bin")

    # I trigger this build here extra so that Cargo has time to install its custom
    # toolchain if it is not present. Otherwise, if someone doesn't have the toolchain
    # installed yet, the build will fail when all Rust builds start in parallel and
    # try to update the systems toolchain.
    fn_build_rust_lib libhedron

    set +e
    PIDS=()
    PROJECTS=()
    for LIB in $LIBS; do
        fn_build_rust_lib "$LIB" &
        # $! contains the PID of the just started process
        # if the process was put into background with "&"
        PID=$!
        PIDS+=("$PID")
        PROJECTS+=("$LIB")
    done

    for BIN in $BINS; do
        fn_build_rust_bin "$BIN" &
        # $! contains the PID of the just started process
        # if the process was put into background with "&"
        PID=$!
        PIDS+=("$PID")
        PROJECTS+=("$BIN")
    done

    # to optimize build time, I build everything
    # in parallel; "&" creates a bacground task
    # for every compilation!
    wait_and_check $PIDS $PROJECTS

    fn_build_extra_checks
}

function fn_build_rust_lib() {
    (
        cd "$1" || exit
        # For a lib "check" is enough.
        # It will be build anyway by the bins that uses it.
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

# Waits fo all
function wait_and_check() {
    PIDS=$1
    PROJECTS=$2

    # same len for arrays PIDS and PROJECTS
    LEN="${#PIDS[@]}"

    for ((i = 0; i < LEN; i++)); do
        if wait "${PIDS[i]}"; then
            echo "${PROJECTS[i]} success"
        else
            echo "${PROJECTS[i]} failure!"
            exit 1
        fi
    done
}

fn_main
