#!/usr/bin/env bash

set -e

#########################################################################
# nice "hack" which make the script work, even if not executed from "./"
DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit
#########################################################################


cd "./ws" || exit
./build-cargo-fake-workspace.sh
cd ..

ln -sf "./ws/roottask-bin/target/x86_64-unknown-hedron/debug/roottask-bin" . || exit
