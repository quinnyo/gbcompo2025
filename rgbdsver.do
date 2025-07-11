#!/usr/bin/env bash
set -euo pipefail

redo-always
redo-stamp < <(rgbasm --version && rgblink --version && rgbfix --version)

# vim: ft=bash
