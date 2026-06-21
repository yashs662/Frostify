#!/usr/bin/env bash
# Debug launcher (REMOVABLE — delete the `debug/` dir + `automation`
# feature to rip out). Kills any stale opal process, builds with the
# automation feature, runs against a JSON config/script.
#
#   ./debug/run.sh                  # uses debug/home.json
#   ./debug/run.sh debug/liked.json
set -e
CONFIG="${1:-debug/home.json}"

pkill -f 'target/debug/opal' 2>/dev/null || true
sleep 0.3

cargo build --features automation
exec ./target/debug/opal --config "$CONFIG"
