#!/usr/bin/env bash
set -euo pipefail

function logecho {
	printf "%s " "[$(date --iso-8601=seconds)]" "$@"
	printf "\n"
}

function errorecho {
	logecho "ERROR:" "$@" >&2
}

function errorexit {
	local retval
	retval=1
	if [ $# -gt 0 ]; then
		if [[ $1 =~ [0-9]+ ]]; then
			retval=$1
			shift
		fi
	fi
	if [ $# -gt 0 ]; then
		errorecho "$@"
	else
		errorecho "?"
	fi
	exit "$retval"
}

function requirecmd {
	local status
	status=0
	while [[ $# -gt 0 ]]; do
		if ! type "${1}"; then
			errorecho "command '${1}' not found."
			status=1
		fi
		shift
	done
	return $status
}

function projectdir_fingerprint_data {
	echo "watch-build projectdir fingerprint"
	(type git >/dev/null 2>&1) &&
		(git rev-parse --show-toplevel --absolute-git-dir --symbolic-full-name HEAD 2>/dev/null) ||
		echo "nogit ${PWD}"
}

function projectdir_fingerprint {
	projectdir_fingerprint_data | grep -o '^\S*' <(sha1sum -)
}

function build {
	logecho "Build starting..."
	if (redo -j 8) ; then
		logecho "Build complete."
	else
		logecho "Build failed."
	fi
}

requirecmd inotifywait redo || exit 1


## File events to monitor in order to trigger rebuild
EVENTS=(modify create delete delete_self move move_self)
## Files and directories to watch
MONITOR=(inc src tool *.do *.sh)
## Patterns to match filenames to exclude from monitoring
_excludes=(
	# files (including dirs) beginning with a dot
	'/[.]'
	# probable temporary/backup files
	'([.](orig|bak)|~)$'
	# Tiled temporary files
	'[.](tiled-project|tiled-session|tmx|tsx)[.][^.]+'
	# Rust build dir
	'/target/'
)
# join exclude patterns into one super pattern
_excljunc="$(printf "(%s)|" "${_excludes[@]}")"
_excljunc="${_excljunc%?}"
MONITOR_EXCLUDE="${_excljunc}"

# make comma separated list
printf -v events_list '%s,' "${EVENTS[@]}"
events_list=${events_list%,}

PROJECT_ID="$(projectdir_fingerprint)"
PROJECT_STATE="${XDG_STATE_HOME:-$HOME/.local/state}/watch-build/${PROJECT_ID}"
mkdir -p "${PROJECT_STATE}"
EVENT_FILE="$(mktemp -p "${PROJECT_STATE}/" eventsXXXX)"

inotifywait --monitor --outfile "${EVENT_FILE}" --recursive --event "${events_list}" --exclude "${MONITOR_EXCLUDE}" "${MONITOR[@]}" &
MONITOR_PID=$!

trap "logecho 'Stop signal received.' && exit" SIGINT SIGTERM
trap "logecho 'Cleaning up...' && rm '${EVENT_FILE}' && kill '${MONITOR_PID}'" EXIT

build

while inotifywait --event create,modify "${EVENT_FILE}" ; do
	sleep 0.5
	build
done

