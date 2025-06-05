#!/usr/bin/env bash
set -euo pipefail

CHRS=(out/assets/fonty8.chr)
redo-ifchange ${CHRS[@]}

# vim: ft=bash
