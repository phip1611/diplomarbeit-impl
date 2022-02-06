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
USERLAND_FILES=$(find . \
  `# make sure we don't search for files in './build/musl'` \
  -maxdepth 1 \
  -type f \
  `# exclude files that start with dot (hidden files)` \
  ! -path '*/.*' \
  `# exclude the userland.tar (otherwise the tar gets exponentially bigger :D)` \
  ! -path '*/userland.tar' \
  `# exclude Roottask` \
  | grep -v "roottask" \
  `# exclude Hedron` \
  | grep -v "hedron" \
  | \tr '\r\n' ' '
)

tar cfv "userland.tar" $USERLAND_FILES
