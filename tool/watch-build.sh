#!/usr/bin/env bash
set -euo pipefail

## File events to monitor in order to trigger rebuild
EVENTS=(modify create delete delete_self move move_self)
## Files and directories to watch
MONITOR=(inc src tool '*.do' '*.sh')
## Pattern to match filenames to exclude from monitoring
MONITOR_EXCLUDE='.*([.](tiled-project|tiled-session|tmx|tsx)[.][^.]*|~$)'

type inotifywait || exit 1

function build() {
	printf "[%s] Building...\n" "$(date --iso-8601=seconds)"
	sleep 1
	redo -j 8
	printf "[%s] Done.\n" "$(date --iso-8601=seconds)"
}

build

# make comma separated list
printf -v events_list '%s,' ${EVENTS[@]}
events_list=${events_list%,}

while inotifywait --recursive --event ${events_list} --exclude "${MONITOR_EXCLUDE}" ${MONITOR[@]} ; do
	build
done

