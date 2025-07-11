#!/usr/bin/env bash
set -euo pipefail

redo-ifchange config.sh
. ./config.sh

redo-ifchange rgbdsver

# Ensure assets build first
redo-ifchange assets

redo-ifchange "${1}.sources"
mapfile -t SOURCES < "${1}.sources"

# Create target obj for each source
OBJDIR="${2}"
OBJS=()
for src in "${SOURCES[@]}"; do
	OBJS+=("${2}/${src#*/}.o")
done
redo-ifchange ${OBJS[@]}

rgblink "${LDFLAGS[@]}" "${OBJS[@]}" -o - -m "${1}.map" -n "${1}.sym" | rgbfix "${FIXFLAGS[@]}" - >$3

# vim: ft=bash
