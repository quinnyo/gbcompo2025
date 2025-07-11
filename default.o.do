#!/usr/bin/env bash
set -euo pipefail

redo-ifchange rgbdsver

SRC="src/${2#*/*/}"

mkdir -p "${1%/*}"
redo-ifchange compile "$SRC"

./compile $1 $2 $3

# Tell redo about rgbasm 'make' dependencies -- essentially any `include`d files.
DEPS=$(cut -d : -f 2- <"$2.d")
redo-ifchange ${DEPS#*:}

# vim: ft=bash
