#!/usr/bin/env bash

# This script checks if the machine is ready to execute this project. It gets called by the Makefile.

set -e

ANSI_RED="\e[31m"
ANSI_GREEN="\e[32m"
ANSI_BOLD="\e[1m"
ANSI_RESET="\e[0m"

# some file of the musl git submodule to check, if the submodule was really checked out
MUSL_SUBMODULE="libc-musl/README"
# some file of the hedron git submodule to check, if the submodule was really checked out
HEDRON_SUBMODULE="thesis-hedron-fork/README.md"

EXIT_FAILURE=0

fn_main() {
    echo "Checking system:"
    fn_check_x86_64
    fn_broadwell_notice
    fn_check_kvm

    if [ $EXIT_FAILURE -ne 0 ]; then
        echo "If KVM is not available: Perhaps you have to enable virtualization in BIOS/UEFI."
        exit 1
    fi
}

fn_check_x86_64() {
    set +e
    lscpu | grep x86_64 > /dev/null
    set -e
    if [[ $? -eq 0 ]] ; then
        echo -e "  ✅  ${ANSI_GREEN}CPU is x86_64.${ANSI_RESET}"
    else
        echo -e "  ❌  ${ANSI_RED}${ANSI_BOLD}CPU is NOT x86_64!${ANSI_RESET}"
        EXIT_FAILURE=1
    fi
}

fn_check_kvm() {
    set +e
    lsmod | grep kkvm > /dev/null
    if [[ $? -eq 0 ]] ; then
        echo -e "  ✅  ${ANSI_GREEN}KVM available.${ANSI_RESET}"
    else
        echo -e "  ❌  ${ANSI_RED}${ANSI_BOLD}KVM NOT available.${ANSI_RESET}"
        EXIT_FAILURE=1
    fi
    set -e
}

fn_broadwell_notice() {
    set +e
    echo -n "     "
    lscpu | grep -i "model name"
    echo -e "     ${ANSI_BOLD}Please make sure you are running a 'broadwell' processor or newer. Otherwise,"
    echo -e "     please look into the README. Can be easily fixed.${ANSI_RESET}"
    set -e
}

# call main
fn_main
