#!/usr/bin/env bash
set -euo pipefail

redo-ifchange config.sh rgbdsver
. ./config.sh

# Assets
redo-ifchange "${1}.assets"

# Code
redo-ifchange "${1}.asm.sources"
mapfile -t ASM_SRCS < "${1}.asm.sources"

# Create target obj for each source
OBJS=()
for src in "${ASM_SRCS[@]}"; do
	stub="${src#${SRCPREFIX%/}/}"
	OBJS+=("${BUILDPREFIX%/}/${stub}.o")
done
redo-ifchange ${OBJS[@]}

# Link & finalise ROM
rgblink "${LDFLAGS[@]}" "${OBJS[@]}" -o - -m "${1}.map" -n "${1}.sym" |
	rgbfix "${FIXFLAGS[@]}" - >$3

# vim: ft=bash

