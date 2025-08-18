#!/usr/bin/env bash
set -euo pipefail

find src/assets -name '*.png' -type f -or -name '*.png.params' -type f |
	sort >$3
redo-always
redo-stamp <$3

# vim: ft=bash

