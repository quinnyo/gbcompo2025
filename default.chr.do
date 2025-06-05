#!/usr/bin/env bash
set -euo pipefail

SRC="src/${2#*/}.png"
redo-ifchange "$SRC"

mkdir -p "${2%/*}"
rgbgfx -o "$3" "$SRC"

# vim: ft=bash
