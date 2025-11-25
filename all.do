#!/usr/bin/env bash
set -euo pipefail

redo-ifchange config.sh
. ./config.sh

redo-ifchange "${OUTFILE}.target"

# vim: ft=bash

