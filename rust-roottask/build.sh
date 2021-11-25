#!/usr/bin/env bash

# Builds and tests all crates. Copies all binaries into "./build".

set -e

#########################################################################
# nice "hack" which make the script work, even if not executed from "./"
DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit
#########################################################################

function fn_main() {
    fn_build_ws
    fn_cp_binaries_to_build_dir
    fn_cp_linux_userland_payload
    fn_runtime_environment_tarball
}

function fn_build_ws() {
    cd "./ws" || exit
    ./build-cargo-fake-workspace.sh
    cd ..
}

function fn_cp_binaries_to_build_dir() {
    rm -rf "./build"

    SEARCH_DEBUG_FILES=$(find . -maxdepth 6 -type f ! -path . | grep "x86_64-unknown-hedron/debug")
    SEARCH_RELEASE_FILES=$(find . -maxdepth 6 -type f ! -path . | grep "x86_64-unknown-hedron/release")

    mkdir -p "./build"

    for FILE in $SEARCH_DEBUG_FILES
    do
        if file "$FILE" | grep "executable" > /dev/null
        then
            NAME=$(basename "$FILE")
            echo "$NAME now in './build/${NAME}_debug.elf'"
            # ln -fs "$FILE" "./build/${NAME}_release.elf"
            # we have to copy; QEMUs "-initrd" doesn't work with links
            cp "$FILE" "./build/${NAME}_debug.elf"

            # also stripped binaries, because they are smaller
            # => less mem delegations => faster
            cp "$FILE" "./build/${NAME}_debug_stripped.elf"
            strip "./build/${NAME}_debug_stripped.elf"
        fi
    done

    for FILE in $SEARCH_RELEASE_FILES
    do
        if file "$FILE" | grep "executable" > /dev/null
        then
            NAME=$(basename "$FILE")
            echo "$NAME now in './build/${NAME}_release.elf'"
            # ln -fs "$FILE" "./build/${NAME}_release.elf"
            # we have to copy; QEMUs "-initrd" doesn't work with links
            cp "$FILE" "./build/${NAME}_release.elf"

            # also stripped binaries, because they are smaller
            # => less mem delegations => faster
            cp "$FILE" "./build/${NAME}_release_stripped.elf"
            strip "./build/${NAME}_release_stripped.elf"
        fi
    done
}

function fn_cp_linux_userland_payload() {
    (
        cd "../static-hello-world" || exit
        make
        # merge build-directories
        cp -r "build/" "../rust-roottask/"
    )
}

# Builds the whole runtime environment into a tarball.
# This means all executables in ./build except the roottask.
function fn_runtime_environment_tarball() {
    # Linux Files We
    LINUX_BINS=$(cd "./build" || exit && find . -name "linux_*" | tr '\r\n' ' ')

    # debug
    (
        cd "./build" || exit
        # space separated string
        HRSTD_RT_DEBUG_FILES=$(find . -name "*.elf" ! -path . | grep "_debug_stripped" | grep -v "roottask" | tr '\r\n' ' ')
        tar cfv "hedron-userland_debug.tar" $HRSTD_RT_DEBUG_FILES $LINUX_BINS
    )

    # release
    (
        cd "./build" || exit
        # space separated string
        HRSTD_RT_RELEASE_FILES=$(find . -name "*.elf" ! -path . | grep "_release_stripped" | grep -v "roottask" | tr '\r\n' ' ')
        tar cfv "hedron-userland_release.tar" $HRSTD_RT_RELEASE_FILES $LINUX_BINS
    )
}

fn_main
