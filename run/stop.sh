#!/usr/bin/env bash
#
# Stop Git History Editor (dev server and running app instances)
#
# Kills:
#   - Tauri dev processes
#   - Vite dev server
#   - Git History Editor app instances
#
# Usage:
#   ./run/stop.sh
#

set -euo pipefail

echo "Stopping Git History Editor..."

# Kill Tauri dev processes
pkill -f "cargo run" || true
pkill -f "tauri dev" || true

# Kill Vite dev server
pkill -f "vite" || true

# Kill any running Git History Editor app instances
pkill -f "Git History Editor" || true
killall "Git History Editor" 2>/dev/null || true

echo "Done. All Git History Editor processes stopped."
