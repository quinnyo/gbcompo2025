#!/usr/bin/env bash
set -euo pipefail

# Generate a list of source files (TARGET.sources) for building a GB ROM.
find src -name '*.rgbasm' -type f |
sort >$3
redo-always
redo-stamp <$3

# vim: ft=bash
