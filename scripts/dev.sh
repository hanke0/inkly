#!/bin/bash

set -e

backend_pid=
front_pid=

cd "$(dirname "$0")/.."

kill_tree() {
    local pid=$1
    local sig=${2:-TERM}
    for child in $(pgrep -P "$pid" 2>/dev/null || true); do
        kill_tree "$child" "$sig"
    done
    kill -"$sig" "$pid" 2>/dev/null || true
}

cleanup() {
    set +e
    trap '' EXIT INT TERM HUP

    [ -n "$backend_pid" ] && kill_tree "$backend_pid"
    [ -n "$front_pid" ] && kill_tree "$front_pid"

    sleep 0.5

    [ -n "$backend_pid" ] && kill_tree "$backend_pid" KILL
    [ -n "$front_pid" ] && kill_tree "$front_pid" KILL

    wait 2>/dev/null
}

trap cleanup EXIT INT TERM HUP

cargo run --release -p inkly >backend.log 2>&1 &
backend_pid=$!

cd frontend
npm run dev >../frontend.log 2>&1 &
front_pid=$!
cd ..

while true; do
    if ! kill -0 "$backend_pid" 2>/dev/null; then
        echo "backend exited, shutting down..."
        break
    fi
    if ! kill -0 "$front_pid" 2>/dev/null; then
        echo "frontend exited, shutting down..."
        break
    fi
    sleep 1
done
