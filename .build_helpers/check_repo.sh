#!/usr/bin/env bash

# This script checks if the repo is ready to be built. It gets called by the Makefile.

set -e

ANSI_GREEN="\e[32m"
ANSI_RED="\e[31m"
ANSI_BOLD="\e[1m"
ANSI_RESET="\e[0m"

# some file of the musl git submodule to check, if the submodule was really checked out
MUSL_SUBMODULE="libc-musl/README"
# some file of the hedron git submodule to check, if the submodule was really checked out
HEDRON_SUBMODULE="thesis-hedron-fork/README.md"

EXIT_FAILURE=0

fn_main() {
    echo "Checking repo:"
    fn_check_submodule_exists "libc-musl" "libc-musl/README"
    fn_check_submodule_exists "hedron" "thesis-hedron-fork/README.md"

    if [ $EXIT_FAILURE -ne 0 ]; then
        echo "⚠ Please initialize the git submodules!"
        exit 1
    fi
}

fn_check_submodule_exists() {
    NAME=$1
    MODULE_PATH=$2

    if test -f "$MODULE_PATH"; then
        echo -e "  ✅  ${ANSI_GREEN}submodule '$NAME' is available.${ANSI_RESET}"
    else
        echo -e "  ❌  ${ANSI_RED}${ANSI_BOLD}submodule '$NAME' is not available!${ANSI_RESET}"
        EXIT_FAILURE=1
    fi
}

# call main
fn_main
