#!/usr/bin/env bash
set -euo pipefail

redo-ifchange all config.sh
. ./config.sh

${MESENEXE} "${MESENARGS[@]}" "${OUTFILE}" >&2

# vim: ft=bash

