#!/usr/bin/env bash

# This script starts the Hedron Microhypervisor via Multiboot1 in QEMU without a GUI
# and gives the binary of the roottask as first multiboot1 boot module
# along. Hedron will take the first boot module, extract it as ELF file
# and start it.

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
    ./_check_qemu_version.sh
    fn_run_qemu
}

fn_run_qemu() {
    QEMU_ARGS=(
        # Disable default devices
        # QEMU by default enables a ton of devices which slow down boot.
        "-nodefaults"

        "-nographic"

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

        "-append"
        # Hedron-specific args
        "serial novga"

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

    )

    # echo "Executing: qemu-system-x86_64 " "${QEMU_ARGS[@]}"
    qemu-system-x86_64 "${QEMU_ARGS[@]}"
}

# call main
fn_main
