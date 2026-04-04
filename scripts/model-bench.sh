#!/bin/bash

usage() {
	cat <<EOF
Usage: $0 [Options] [model-name]...
Options:
    --cpu: Run the benchmark on the CPU.
    --file: The file to run the benchmark on.
EOF
}

models=()
args=()
while [ $# -gt 0 ]; do
	case "$1" in
	--cpu)
		args+=("--cpu")
		shift
		;;
	--file)
		args+=("--file" "$2")
		shift 2
		;;
	-*)
		echo "Unknown option: $1"
		usage
		exit 1
		;;
	*)
		models+=("$1")
		shift
		;;
	esac
done

if [ "${#models[@]}" -eq 0 ]; then
	models=(
		qwen3.5:0.8b
		qwen3.5:2b
		deepseek-r1:1.5b
		lfm2.5:1.2b
		gemma4:2b
	)
fi

banner=$(printf '%*s' 60 '' | tr ' ' '=')

vexec() {
	printf "%q " "$@"
    echo
	"$@"
}

for model in "${models[@]}"; do
	echo "$banner"
	vexec cargo 2>/dev/null run --release -- \
		summary-bench --model "$model" \
		"${args[@]}"
	echo "$banner"
	echo
done
