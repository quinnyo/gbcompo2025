#!/usr/bin/env bash
set -euo pipefail

redo-ifchange config.sh
. ./config.sh
cat <<EOF > "$3"
	rgbasm ${ASFLAGS[@]} -o \$3 -M \$2.d src/\${2#*/*/} >&2
EOF
chmod +x "$3"

# vim: ft=bash
