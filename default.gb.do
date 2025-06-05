#!/usr/bin/env bash
set -euo pipefail

redo-ifchange config.sh
. ./config.sh

# Ensure assets build first
redo-ifchange assets

# Gather sources
mapfile -d '' SOURCES < <(find src -name '*.rgbasm' -type f -print0 | sort --zero-terminated)

# Create target obj for each source
OBJDIR="${2}"
OBJS=()
for src in "${SOURCES[@]}"; do
	OBJS+=("${2}/${src#*/}.o")
done
redo-ifchange ${OBJS[@]}

rgblink "${LDFLAGS[@]}" "${OBJS[@]}" -o - -m "${1}.map" -n "${1}.sym" | rgbfix "${FIXFLAGS[@]}" - >$3

# vim: ft=bash
