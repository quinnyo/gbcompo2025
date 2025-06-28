#!/usr/bin/env bash
set -euo pipefail

CHRS=(out/assets/fonty8.chr out/assets/iconoglyphs.chr out/assets/character_test.chr)
redo-ifchange ${CHRS[@]}

# vim: ft=bash
