#!/usr/bin/env bash

# This script starts the Hedron Microhypervisor via Multiboot1 in QEMU
# and gives the binary of the roottask as first multiboot1 boot module
# along. Hedron will take the first boot module, extract it as ELF file
# and start it.
#
# The setup of this "run_qemu.sh" is tightly coupled to my personal setup..

set -e

# make sure that this copy is up-to-date!
BUILD_DIR="../build"
HEDRON="$BUILD_DIR/hedron.elf32"

# "debug" or "release"; only influences the roottask binary itself
RELEASE=release

ROOTTASK="$BUILD_DIR/roottask-bin"
# all the other Rust binaries that get loaded by the Roottask
USERLAND="$BUILD_DIR/userland.tar"

#########################################################################
# nice "hack" which make the script work, even if not executed from "./"
DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit
#########################################################################

# main allows us to move all function definitions to the end of the file
fn_main() {
    fn_check_qemu_version
    fn_run_qemu
}

# Checks if QEMU 6.2 or above is used. Otherwise, it outputs a warning.
# Before 6.2 there is no fast loading of multiboot modules into memory via the dma
# device. See https://gitlab.com/qemu-project/qemu/-/commit/48972f8cad24eb4462c97ea68003e2dd35be0444
fn_check_qemu_version() {
    # if this fails it's not that bad
    set +e

    REGEX="(([0-9]+)\.([0-9]+)\.([0-9]+))"
    MATCHES=$(qemu-system-x86_64 --version | grep -oE "$REGEX")
    # map multiple matches (multiline string) to array
    MATCHES=($(echo $MATCHES))
    # Version is now something like: "6.2.50"
    VERSION=${MATCHES[0]}

    REGEX="([0-9]+)"
    MATCHES=$(echo "$VERSION" | grep -oE "$REGEX")
    # map multiple matches (multiline string) to array
    MATCHES=($(echo $MATCHES))
    # major version as string
    MAJOR_VERSION=${MATCHES[0]}
    # feature version as string
    FEATURE_VERSION=${MATCHES[1]}
    # major version as integer
    MAJOR_VERSION=$(($MAJOR_VERSION + 0))
    # feature version as integer
    FEATURE_VERSION=$((FEATURE_VERSION + 0))
    echo "QEMU MAJOR_VER=${MAJOR_VERSION}, QEMU_FEATURE_VER=${FEATURE_VERSION}"

    if [[ $MAJOR_VERSION -lt 6 ]] || [[ $MAJOR_VERSION == 6 && $FEATURE_VERSION -lt 2 ]]; then
        ANSI_RED="\e[31m"
        ANSI_BOLD="\e[1m"
        ANSI_RESET="\e[0m"
        echo -e "${ANSI_BOLD}${ANSI_RED}"
        echo "===================================================================================="
        echo "QEMU: Please use Version 6.2 or above! Otherwise the loading of larger files as"
        echo "multiboot modules is really slow! Startup might take more than 10 seconds!"
        echo "===================================================================================="
        echo -e "${ANSI_RESET}"
    fi

    # restore fail on error
    set -e
}

fn_run_qemu() {
    QEMU_ARGS=(
        # Disable default devices
        # QEMU by default enables a ton of devices which slow down boot.
        "-nodefaults"

        # Use a standard VGA for graphics
        "-vga"
        "std"

        # Use a modern machine, with acceleration if possible.
        "-machine"
        "q35,accel=kvm:tcg"

        # Allocate some memory
        "-m"
        "2048M"

        # two cores
        "-smp"
        "2"

        # I also use this CPU micro arch to optimize all
        # Rust binaries for.
        "-cpu"
        "host"

        # Multiboot1 kernel
        "-kernel"
        "${HEDRON}"

        #"-append"
        # "" (additional Hedron args: "serial", "novga", ...)

        # QEMU passes this as Multiboot1 Modules to Hedron. Multiple modules are separated
        # by a comma. The text after the path is the "cmdline" string of the boot module.
        "-initrd"
        "${ROOTTASK} roottask,${USERLAND} userland"

        # Logging from the Roottask:
        # Same content as the serial log, but persists QEMU shutdowns (until the next run).
        "-debugcon"
        "file:../qemu_debugcon.txt"

        # Enable serial
        "-serial"
        "stdio"

        # Setup monitor
        "-monitor"
        "vc:1024x768"
    )

    # echo "Executing: qemu-system-x86_64 " "${QEMU_ARGS[@]}"
    qemu-system-x86_64 "${QEMU_ARGS[@]}"
}

# call main
fn_main
