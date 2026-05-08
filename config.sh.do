#!/usr/bin/env bash

SRCS=( 'config-core.sh' 'config-override.sh' )

echo '# config.sh (generated)' > "$3"
for src in "${SRCS[@]}"; do
	if [[ -e "${src}" ]]; then
		echo ". '${src}'" >> "$3"
		redo-ifchange "${src}"
	else
		redo-ifcreate "${src}"
	fi
done

# vim: ft=bash

