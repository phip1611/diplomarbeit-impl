#!/usr/bin/env bash

# Helper script for "run_qemu.sh".
# Checks if QEMU 6.2 or above is used. Otherwise, it outputs a warning.
# Before 6.2 there is no fast loading of multiboot modules into memory via the dma
# device. See https://gitlab.com/qemu-project/qemu/-/commit/48972f8cad24eb4462c97ea68003e2dd35be0444

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
    echo "You notice this if 'Booting from ROM...' stands in QEMUs GUI for a longer time."
    echo "===================================================================================="
    echo -e "${ANSI_RESET}"
fi
