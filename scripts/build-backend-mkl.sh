#!/bin/bash

set -eo pipefail

cd "$(dirname "$0")/.."

cmd="${1:-build}"
[ $# -gt 0 ] && shift

case "${cmd}" in
build) ;;
run) ;;
*)
	echo "Unsupported command: $cmd" >&2
	exit 1
	;;
esac

exec cargo $cmd -p inkly --features mkl --release "$@"
