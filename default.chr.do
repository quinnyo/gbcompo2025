#!/usr/bin/env bash
set -euo pipefail

rgbgfx --version | redo-stamp

SRC="src/${2#*/}.png"
redo-ifchange "$SRC"

PARAMS="${SRC}.params"
ATFILE=""
if [[ -e "$PARAMS" ]]; then
	ATFILE="@$PARAMS"
	redo-ifchange "$PARAMS"
fi

mkdir -p "${2%/*}"
rgbgfx -p "${1}.pal" -a "${1}.atrb" -t "${1}.idx" -o "$3" $ATFILE -- "$SRC"

# vim: ft=bash
