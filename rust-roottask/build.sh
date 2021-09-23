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

# BUILD_TYPE=release
BUILD_TYPE=debug
ln -sf "./ws/roottask-bin/target/x86_64-unknown-hedron/${BUILD_TYPE}/roottask-bin" . || exit
