#!/usr/bin/env bash
# Build inkly backend with the right Candle GPU backend for the host OS.
set -euo pipefail
cd "$(dirname "$0")/.."
case "$(uname -s)" in
Darwin)
  exec cargo build -p inkly --features metal "$@"
  ;;
Linux)
  exec cargo build -p inkly --features cuda "$@"
  ;;
MINGW* | MSYS* | CYGWIN*)
  exec cargo build -p inkly --features cuda "$@"
  ;;
*)
  echo "Unsupported OS for GPU build: $(uname -s)" >&2
  exit 1
  ;;
esac
