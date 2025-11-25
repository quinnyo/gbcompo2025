#!/usr/bin/env bash
set -euo pipefail

## Generate CHR, using RGBGFX to convert from PNG.

. ./config.sh

SRC="src/${2#${BUILDPREFIX%/}}.png"
redo-ifchange config.sh "$SRC" "${1}.pal"

PARAMS="${SRC}.params"
ATFILE=""
if [[ -e "$PARAMS" ]]; then
	ATFILE="@$PARAMS"
	redo-ifchange "$PARAMS"
fi

mkdir -p "${2%/*}"
rgbgfx -c "gbc:${1}.pal" -a "${1}.atrb" -t "${1}.idx" -o "$3" $ATFILE -- "$SRC"

# vim: ft=bash

