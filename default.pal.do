#!/usr/bin/env bash
set -euo pipefail

## Generate palette from CHR source.

. ./config.sh

stub=${1#${BUILDPREFIX%/}}
SRC="src/${stub%.chr.pal}.png"
redo-ifchange config.sh "$SRC"

PARAMS="${SRC}.params"
ATFILE=""
if [[ -e "$PARAMS" ]]; then
	ATFILE="@$PARAMS"
	redo-ifchange "$PARAMS"
fi

mkdir -p "${2%/*}"
rgbgfx -p "$3" $ATFILE -- "$SRC"


# vim: ft=bash

