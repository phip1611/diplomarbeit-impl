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
    fn_cp_runtime_binaries_to_build_dir
    fn_cp_foreign_linux_binaries
    fn_runtime_environment_tarball
}

# invokes the build of the whole Cargo workspace
function fn_build_ws() {
    cd "./ws" || exit
    ./build-cargo-fake-workspace.sh
    cd ..
}

# copy all artefacts (Roottask + Runtime Services) from the Rust workspace into the ./build directory
function fn_cp_runtime_binaries_to_build_dir() {
    rm -rf "./build"


    # there might be no release files or debug files (this errors after a "clean" otherwise sometimes)
    set +e
    SEARCH_DEBUG_FILES=$(find . -maxdepth 6 -type f ! -path . | grep "x86_64-unknown-hedron/debug")
    SEARCH_RELEASE_FILES=$(find . -maxdepth 6 -type f ! -path . | grep "x86_64-unknown-hedron/release")
    set -e

    mkdir -p "./build"

    for FILE in $SEARCH_DEBUG_FILES
    do
        if file "$FILE" | grep "executable" > /dev/null
        then
            NAME=$(basename "$FILE")
            echo "$NAME now in './build/${NAME}--debug.elf'"
            # ln -fs "$FILE" "./build/${NAME}_release.elf"
            # we have to copy; QEMUs "-initrd" doesn't work with links
            cp "$FILE" "./build/${NAME}--debug.elf"

            # also stripped binaries, because they are smaller
            # => less mem delegations => faster
            # cp "$FILE" "./build/${NAME}_debug_stripped.elf"
            # strip "./build/${NAME}_debug_stripped.elf"
        fi
    done

    for FILE in $SEARCH_RELEASE_FILES
    do
        if file "$FILE" | grep "executable" > /dev/null
        then
            NAME=$(basename "$FILE")
            echo "$NAME now in './build/${NAME}--release.elf'"
            # ln -fs "$FILE" "./build/${NAME}_release.elf"
            # we have to copy; QEMUs "-initrd" doesn't work with links
            cp "$FILE" "./build/${NAME}--release.elf"

            # also stripped binaries, because they are smaller
            # => less mem delegations => faster
            # cp "$FILE" "./build/${NAME}_release_stripped.elf"
            # strip "./build/${NAME}_release_stripped.elf"
        fi
    done
}

# copies the foreign Linux binaries into the build directory
function fn_cp_foreign_linux_binaries() {
    (
        cd "../static-hello-world" || exit
        make
        # copy everything into "./build"
        cp -r "build/" "../rust-roottask/"
    )
}

# Builds the whole userland into a tarball. This includes runtime services and user applications.
# It uses all executables in "./build" except for the roottask.
# Some binaries are included twice as "debug" and as "release" version.
function fn_runtime_environment_tarball() {
    (
        cd "./build" || exit

        # space separated string
        RUST_RT_AND_LINUX_BINS=$(find . -name "*.elf" ! -path . | grep -v "roottask" | tr '\r\n' ' ')
        tar cfv "hedron-userland_full.tar" $RUST_RT_AND_LINUX_BINS
    )
}

fn_main
