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

# find all direct directories without "."
#dirs=$(find . -maxdepth 1 -type d ! -path .)

#for dir in $dirs
#do
#  # echo $dir
#  (
#    cd "$dir" || exit
#    cargo build
#  )
#done


cd "roottask-lib" || exit
cargo build
cargo test
# cargo fmt -- --check
# cargo clippy
# cargo doc
cd ..


cd "roottask-bin" || exit
cargo build
# cargo fmt -- --check
# cargo clippy
# cargo doc
cd ..
