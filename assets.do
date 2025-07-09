#!/usr/bin/env bash
set -euo pipefail

# find source images to convert
mapfile -t CHRSRCS < <(find src/assets -name '*.png' -type f | sort)

echo "${CHRSRCS[@]}" | redo-stamp

CHRS=()
for src in "${CHRSRCS[@]}"; do
	noext="${src%.png}"
	CHRS+=("out/${noext#*/}.chr")
done

redo-ifchange ${CHRS[@]} ${CHRSRCS[@]}

# vim: ft=bash
