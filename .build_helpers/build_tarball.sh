#!/usr/bin/env bash

# Invoked by make.
# Builds the userland tarball.

set -e

#########################################################################
# nice "hack" which make the script work, even if not executed from "./"
DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit
#########################################################################

cd "../build" || exit

# oneline string with all files for the tarball
USERLAND_FILES=$(find . -type f\
 | grep -v "roottask" \
 | grep -v "hedron" \
 | \tr '\r\n' ' '
 )

tar cfv "userland.tar" $USERLAND_FILES
