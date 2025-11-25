#!/usr/bin/env bash
set -euo pipefail

# Generate lists of source files automatically.
# Target name includes source type code, like: `*.SRCTYPE.sources`.
# Currently this script just has hardcoded behaviours for specific type codes.

SRCTYPE="${2##*.}"
case "$SRCTYPE" in
	asm) find src -name '*.rgbasm' -type f -or -name '*.asm' -type f
		;;
	chr) find src/assets -name '*.png' -type f -or -name '*.png.params' -type f
		;;
	*) exit 1
		;;
esac |
	sort --unique >$3
redo-always
redo-stamp <$3

# vim: ft=bash

