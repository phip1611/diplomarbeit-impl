#!/usr/bin/env bash

# This script starts the Hedron Microhypervisor via Multiboot1 in QEMU
# and gives the binary of the roottask as first multiboot1 boot module
# along. Hedron will take the first boot module, extract it as ELF file
# and start it.
#
# The setup of this "run_qemu.sh" is tightly coupled to my personal setup..

set -e

# make sure that this copy is up-to-date!
HEDRON=/tftpboot/hypervisor.elf32

# "debug" or "release"
RELEASE=debug

ROOTTASK="./build/roottask-bin_${RELEASE}.elf"
# all the other Rust binaries that get loaded by the Roottask
HEDRON_USERLAND="./build/hedron-userland_${RELEASE}.tar"

#########################################################################
# nice "hack" which make the script work, even if not executed from "./"
DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit
#########################################################################

# main allows us to move all function definitions to the end of the file
main() {

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

        "-cpu"
        "IvyBridge"

        # Multiboot1 kernel
        "-kernel"
        "${HEDRON}"

        # QEMU passes this as Multiboot1 Modules to Hedron. Multiple modules are separated
        # by a comma. The text after the path is the "cmdline" string of the boot module.
        "-initrd"
        "${ROOTTASK} roottask,${HEDRON_USERLAND} userland"

        # I use this for logging files that survive
        # a QEMU shutdown or crash. Log keeps
        # persistent until the next run.
        "-debugcon"
        "file:qemu_debugcon.txt"

        # Enable serial
        #
        # Connect the serial port to the host. OVMF is kind enough to connect
        # the UEFI stdout and stdin to that port too.
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
main
