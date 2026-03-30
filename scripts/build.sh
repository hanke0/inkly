#!/bin/bash

CHINA_MIRROR="${CHINA_MIRROR:-0}"

docker_args=()
if [ "${CHINA_MIRROR}" = "1" ]; then
	docker_args+=("--build-arg" "CHINA_MIRROR=1")
fi

case "${1:-}" in
mkl)
	exec docker build -f Dockerfile.mkl "${docker_args[@]}" -t inkly-builder:mkl .
	;;
cuda)
	exec docker build -f Dockerfile.cuda "${docker_args[@]}" -t inkly-builder:cuda .
	;;
*)
	echo "Usage: $0 [mkl|cuda|avx2]"
	exit 1
	;;
esac
