#!/usr/bin/env bash
set -euo pipefail

## Top-level assets target script.

redo-ifchange config.sh
. ./config.sh

## CHR assets
CHR_SRC_LIST="${OUTFILE}.chr.sources"
redo-ifchange "${CHR_SRC_LIST}"
mapfile -t CHR_SRCS < "${CHR_SRC_LIST}"

CHRS=()
for src in "${CHR_SRCS[@]}"; do
	if [[ $src == *.png ]]; then
		stub="${src#${SRCPREFIX%/}/}"
		CHRS+=("${BUILDPREFIX%/}/${stub%.*}.chr")
	fi
done

# CHRs
redo-ifchange ${CHRS[@]} ${CHR_SRCS[@]}

# vim: ft=bash

