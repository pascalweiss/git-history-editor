#!/usr/bin/env bash
#
# Start Git History Editor development server
#
# Usage:
#   ./run/start.sh          # Start Tauri dev mode with devtools
#   ./run/start.sh --build  # Start with full production build
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_DIR"

if [ "${1:-}" = "--build" ]; then
    echo "Starting Git History Editor (production build)..."
    npm run tauri:build
    open "src-tauri/target/release/bundle/macos/Git History Editor.app"
else
    echo "Starting Git History Editor (dev mode)..."
    echo "App will open at http://localhost:1420"
    npm run tauri:dev -- --features devtools
fi
