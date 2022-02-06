#!/usr/bin/env sh

# This script checks if relevant tooling is available. It gets called by the Makefile.

set -e

ANSI_GREEN="\e[32m"
ANSI_RED="\e[31m"
ANSI_RESET="\e[0m"

EXIT_FAILURE=0

fn_main() {
    echo "Checking relevant tooling:"
    # make sure this script works with "sh" only :D
    fn_check_tool bash
    fn_check_tool gcc
    # actually ridiculous, because this gets invoked by make :)
    fn_check_tool make
    fn_check_tool cmake
    fn_check_tool rustc
    fn_check_tool cargo
    fn_check_tool rustup
    fn_check_tool qemu-system-x86_64

    if [ $EXIT_FAILURE -ne 0 ]; then
        echo "In case cargo is missing and you've just installed it: It isn't immediately"
        echo " in PATH but only after a re-login."
        exit 1
    fi
}

fn_check_tool() {
    TOOL=$1
    set +e
    which $1 >/dev/null
    # $? contains the exit code
    if [ $? -eq 0 ]; then
        echo "  ✅  ${ANSI_GREEN}Tool '$TOOL' is available.${ANSI_RESET}"
    else
        echo "  ❌  ${ANSI_RED}Tool '$TOOL' is not available.${ANSI_RESET}"
        EXIT_FAILURE=1
    fi
    set -e
}

# call main
fn_main
