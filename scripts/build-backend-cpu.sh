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

case "$(uname -s)" in
Darwin)
	exec cargo $cmd -p inkly --features accelerate --release "$@"
	;;
Linux)
	exec cargo $cmd -p inkly --features cuda --release "$@"
	;;
MINGW* | MSYS* | CYGWIN*)
	exec cargo $cmd -p inkly --features mkl --release "$@"
	;;
*)
	echo "Unsupported OS for CPU build: $(uname -s)" >&2
	exit 1
	;;
esac
