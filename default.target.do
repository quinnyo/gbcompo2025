#!/usr/bin/env bash
set -euo pipefail


mkdir -p "$(dirname "${1}")"

printf "${1}" >$3

redo-ifchange "${2}"


# vim: ft=bash

