#!/bin/bash

set -eo pipefail

cd "$(dirname "$0")/.."
# shellcheck source=mkl-env.sh
source "$(dirname "$0")/mkl-env.sh"

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
	inkly_oneapi_mkl_env
	inkly_mkl_preflight_or_die
	exec cargo $cmd -p inkly --features mkl --release "$@"
	;;
MINGW* | MSYS* | CYGWIN*)
	exec cargo $cmd -p inkly --features mkl --release "$@"
	;;
*)
	echo "Unsupported OS for CPU build: $(uname -s)" >&2
	exit 1
	;;
esac
